// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

use futures::FutureExt;

#[tokio::test]
async fn insert_prs() {
    let config = fetcher::Config {
        github_repo: "molybdenumsoftware/label-tracker-test-fixture"
            .parse()
            .unwrap(),
    };

    let result: Result<(), String> = util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();
            fetcher::run(config, &mut connection);
            let landings store::Landing::for_pr(&mut connection, pr_num)
        }
        .boxed()
    })
    .await;

    result.unwrap();
}

#[tokio::test]
async fn update_pr() {
    // Config {repo: test_repo}
    // DbContext {#1}
    fetcher::run(config, context);
    // Db should contain PRs listed above
    //
    let result: Result<(), String> = util::DatabaseContext::with(|context| {
        fetcher::run(config, context);
        todo!()
        // Db should contain PRs listed above
    })
    .await;
}
