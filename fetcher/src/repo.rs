use std::sync::atomic::AtomicBool;

use gix::prelude::*;
pub async fn fetch_or_clone(
    repo_path: &camino::Utf8Path,
    github_repo: &super::GitHubRepo,
) -> anyhow::Result<gix::Repository> {
    if repo_path.exists() {
        // gix::open(...)
        // fetch
        todo!()
    } else {
        let mut fetcher = gix::clone::PrepareFetch::new(
            github_repo.url(),
            repo_path,
            gix::create::Kind::Bare,
            gix::create::Options::default(),
            gix::open::Options::default(),
        )?;

        let repo = fetcher
            .fetch_only(gix::progress::Discard, &AtomicBool::new(false))
            .await?
            .0;

        Ok(repo)
    }
}
