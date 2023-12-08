use std::sync::atomic::AtomicBool;

use gix::prelude::*;
pub async fn fetch_or_clone(
    repo_path: &camino::Utf8Path,
    github_repo: &super::GitHubRepo,
) -> anyhow::Result<()> {
    // TODO: figure out how to do all this with gix. See
    // https://github.com/Byron/gitoxide/issues/1165.

    if repo_path.exists() {
        let status = tokio::process::Command::new("git")
            .args(["-C", repo_path.as_str()])
            .arg("fetch")
            // Run git ignoring any existing config files.
            // https://github.com/git/git/blob/v2.43.0/Documentation/git.txt#L708-L716
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .await?;
        anyhow::ensure!(status.success(), "git fetch failed");
        Ok(())
    } else {
        let status = tokio::process::Command::new("git")
            .args(["-C", repo_path.as_str()])
            .arg("clone")
            // Run git ignoring any existing config files.
            // https://github.com/git/git/blob/v2.43.0/Documentation/git.txt#L708-L716
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .await?;
        anyhow::ensure!(status.success(), "git fetch failed");
        todo!()
    }
}
