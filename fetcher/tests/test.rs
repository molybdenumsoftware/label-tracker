// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

#[test]
fn insert_prs() {
    let config = fetcher::Config {github_repo: "".parse().unwrap()};
    util::DatabaseContext::with(|context| 
        {
            
    fetcher::run(config);
        })
    // Db should contain PRs listed above
}

#[test]
fn update_pr() {
    // Config {repo: test_repo}
    // DbContext {#1}
    fetcher::run();
    // Db should contain PRs listed above
}
