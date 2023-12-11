#![warn(clippy::pedantic)]

use crate::github::GitHub;
use itertools::Itertools;

mod github;
mod repo;

pub struct Config {
    pub github_repo: GitHubRepo,
}

pub struct GitHubRepo {
    owner: String,
    name: String,
}
impl GitHubRepo {
    fn url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }
}

impl std::str::FromStr for GitHubRepo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ower, repo) = s
            .split_once('/')
            .ok_or_else(|| String::from("GitHub repo must contain `/`"))?;
        if repo.contains('/') {
            return Err(String::from("GitHub repo must only contain one `/`"));
        }
        Ok(Self {
            owner: ower.to_owned(),
            name: repo.to_owned(),
        })
    }
}

// master, nixpkgs-unstable, nixpkgs-stable, release-23.11, nixos-23.11

/// Run the darn thing.
///
/// # Errors
///
/// IO that can fail includes database communication, GraphQL requests and git operations.
pub async fn run(
    github_repo: &GitHubRepo,
    db_connection: &mut store::PgConnection,
    github_api_token: &str,
    temp_dir: &camino::Utf8Path,
    branch_patterns: &Vec<wildmatch::WildMatch>,
) -> anyhow::Result<()> {
    let github_client = GitHub::new(github_api_token)?;
    let pulls = github_client.get_pulls(github_repo).await?;
    store::Pr::bulk_insert(db_connection, pulls).await?;
    repo::fetch_or_clone(temp_dir, github_repo).await?;
    repo::write_commit_graph(temp_dir).await?;
    let repo = gix::open(temp_dir)?;
    let references = repo.references()?;
    let branches = find_tracked_branches(&references, branch_patterns)?;
    for (branch_name, head) in branches {
        update_landings(db_connection, &repo, branch_name, head).await?;
    }

    Ok(())
}

async fn update_landings(
    db_connection: &mut store::PgConnection,

    repo: &gix::Repository,
    branch: String,
    head: gix::Id<'_>,
) -> anyhow::Result<()> {
    //<<< let commit = commit_graph
    //<<<     .commit_by_id(head)
    //<<<     .context("commit not found")?;
    let branch = store::Branch::get_or_insert(db_connection, branch).await?;

    for commit in repo.rev_walk([head]).all()? {
        let commit = commit?;

        if let Some(pr) = store::Pr::for_commit(db_connection, commit.id.to_string()).await? {
            store::Landing {
                github_pr: pr.number,
                branch_id: branch.id(),
            }
            .insert(db_connection)
            .await?;
        }
    }

    //<<< commit.iter_parents().for_each(|commit| {
    //<<<     dbg!(&commit);
    //<<< });
    // todo!("{branch}")
    Ok(())
}

// TODO filter these according to a configuration option
fn find_tracked_branches<'a>(
    references: &'a gix::reference::iter::Platform<'_>,
    matchers: &[wildmatch::WildMatch],
) -> anyhow::Result<Vec<(String, gix::Id<'a>)>> {
    references
        // Calling local_branches in a bare repo results in remote branches
        .local_branches()?
        .map(|r| r.map_err(|e| anyhow::anyhow!(e)))
        .map_ok(|branch| (branch.name().shorten().to_string(), branch.id()))
        .filter_ok(|(branch_name, _id)| matchers.iter().any(|matcher| matcher.matches(branch_name)))
        .collect()
}
