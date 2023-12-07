#![warn(clippy::pedantic)]

use crate::github::GitHub;

mod github;
mod repo;

pub struct Config {
    pub github_repo: GitHubRepo,
}

pub struct GitHubRepo {
    owner: String,
    name: String,
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

/// Run the darn thing.
///
/// # Errors
///
/// IO that can fail includes database communication, GraphQL requests and git operations.
pub async fn run(
    github_repo: &GitHubRepo,
    db_connection: &mut store::PgConnection,
    github_api_token: &str,
    repo_path: &camino::Utf8Path,
) -> anyhow::Result<()> {
    let github_client = GitHub::new(github_api_token)?;
    let pulls = github_client.get_pulls(github_repo).await?;
    store::Pr::bulk_insert(db_connection, pulls).await?;
    repo::fetch_or_clone(repo_path, github_repo).await?;
    Ok(())
}
