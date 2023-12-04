// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

#[test]
fn insert_prs() {
    // Config {repo: test_repo}
    // DbContext {}
    fetcher::run();
    // Db should contain PRs listed above
}

#[test]
fn update_pr() {
    // Config {repo: test_repo}
    // DbContext {#1}
    fetcher::run();
    // Db should contain PRs listed above
}
