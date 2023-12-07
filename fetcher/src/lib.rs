#![warn(clippy::pedantic)]

use crate::github::GitHub;
use anyhow::Result;

mod github;

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

pub async fn run(
    github_repo: &GitHubRepo,
    db_context: &mut store::PgConnection,
    github_api_token: &str,
) -> Result<()> {
    let github_client = GitHub::new(github_api_token)?;
    let pulls = github_client.get_pulls(github_repo).await?;
    Ok(())
}
