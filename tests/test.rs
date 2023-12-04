#[test]
fn insert_prs() {
    // Config {repo: test_repo}
    // Pr #1 landed in `master`, `channel1`, #2 landed in `master`, #3
    // DbContext {}
    fetcher::run();
    // Db should contain PRs listed above
}

#[test]
fn update_pr() {
    // Config {repo: test_repo}
    // DbContext {#1}
    // Pr #1 landed in `master`, `channel1`, #2 landed in `master`, #3
    fetcher::run();
    // Db should contain PRs listed above
}
