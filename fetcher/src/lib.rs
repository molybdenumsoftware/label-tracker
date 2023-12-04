#![warn(clippy::pedantic)]

pub struct Config {
    pub github_repo: GitHubRepo,
}

pub struct GitHubRepo {
    owner: String,
    repo: String,
}

impl std::str::FromStr for GitHubRepo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('/')
    }
}

pub fn run(config: Config, db_context: util::DatabaseContext) {}
