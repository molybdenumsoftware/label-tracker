use std::sync::atomic::AtomicBool;

use gix::prelude::*;
pub async fn fetch_or_clone(repo_path: &camino::Utf8Path, github_repo: &super::GitHubRepo) -> anyhow::Result<gix::Repository> {
    let mut fetcher = gix::clone::PrepareFetch::new(format!("https://github.com/{}/{}", github_repo.owner, github_repo.name), repo_path, gix::create::Kind::Bare, gix::create::Options::default(), gix::open::Options::default())?;
    Ok(fetcher.fetch_only(gix::progress::Discard, &AtomicBool::new(false)).await?.0)
}
