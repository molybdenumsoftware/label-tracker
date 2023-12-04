// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

#[test]
fn insert_prs() {
    let config = fetcher::Config {
        github_repo: "molybdenumsoftware/label-tracker-test-fixture"
            .parse()
            .unwrap(),
    };

    let result: Result<(), String> = util::DatabaseContext::with(|context| {
        fetcher::run(config);
        todo!()
        // Db should contain PRs listed above
    }).await;
}

#[test]
fn update_pr() {
    // Config {repo: test_repo}
    // DbContext {#1}
    fetcher::run();
    // Db should contain PRs listed above
}
