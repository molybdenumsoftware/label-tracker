// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

use futures::FutureExt;

async fn assert_landings(connection: &mut store::PgConnection) {
    let mut landings = store::Landing::all(connection)
        .await
        .unwrap();
    landings.sort();
    assert_eq!(landings, [
        store::Landing{github_pr_number: store::PrNumber::from(1), channel: store::Channel("channel1".into())}    ])
}

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
            assert_landings(&mut connection).await;
            Ok(())
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
