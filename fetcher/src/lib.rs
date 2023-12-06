#![warn(clippy::pedantic)]

mod github;

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
        let (ower, repo) = s
            .split_once('/')
            .ok_or_else(|| String::from("GitHub repo must contain `/`"))?;
        if repo.contains('/') {
            return Err(String::from("GitHub repo must only contain one `/`"));
        }
        Ok(Self {
            owner: ower.to_owned(),
            repo: repo.to_owned(),
        })
    }
}

pub fn run(github_repo: &GitHubRepo, db_context: &mut store::PgConnection) {
  todo!()
}
