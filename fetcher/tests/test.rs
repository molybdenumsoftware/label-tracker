// PRs in fixture repository:
// #1: master branch1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

use futures::FutureExt;

impl TestContext<'_> {
    async fn with(
        test: impl for<'a> FnOnce(&'a TestContext<'a>) -> futures::future::LocalBoxFuture<()> + 'static,
    ) {
        util::DatabaseContext::with(|db_context| {
            async move {
                let tempdir = tempfile::tempdir().unwrap();

                let test_context = TestContext {
                    db_context,
                    tempdir,
                };

                test(&test_context).await;
                drop(test_context);
            }
            .boxed_local()
        })
        .await;
    }

    fn repo_dir(&self) -> camino::Utf8PathBuf {
        self.tempdir.path().join("repo").try_into().unwrap()
    }
}

async fn assert(connection: &mut store::PgConnection, expected_landings: &[(i32, &str)]) {
    let mut landings = store::Landing::all(connection).await.unwrap();
    landings.sort();
    let all_branches = store::Branch::all(connection).await.unwrap();

    let actual = landings
        .into_iter()
        .map(|landing| {
            (
                landing.github_pr.0,
                all_branches.get(&landing.branch_id).unwrap().name(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(actual, expected_landings);

    let mut prs = store::Pr::all(connection).await.unwrap();
    prs.sort();

    assert_eq!(
        prs,
        [
            store::Pr {
                number: 1.into(),
                commit: Some("73da20569ac857daf6ed4eed70f2f691626b6df3".into()),
            },
            store::Pr {
                number: 2.into(),
                commit: Some("ab909e9f7125283acdd8f6e490ad5b9750f89c81".into()),
            },
            store::Pr {
                number: 3.into(),
                commit: None,
            },
        ]
    );
}

fn github_repo() -> fetcher::GitHubRepo {
    "molybdenumsoftware/label-tracker-test-fixture"
        .parse()
        .unwrap()
}

fn github_token() -> String {
    std::env::var("GITHUB_TOKEN").expect("$GITHUB_TOKEN should be set")
}

#[tokio::test]
async fn first_run() {
    TestContext::with(|context| {
        async move {
            let mut connection = context.db_context.connection().await.unwrap();

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &context.repo_dir(),
                &[
                    wildmatch::WildMatch::new("master"),
                    wildmatch::WildMatch::new("channel*"),
                ],
            )
            .await
            .unwrap();

            assert(
                &mut connection,
                &[(1, "channel1"), (1, "master"), (2, "master")],
            )
            .await;
        }
        .boxed_local()
    })
    .await;
}

#[tokio::test]
async fn subsequent_run() {
    TestContext::with(|context| {
        async move {
            let mut connection = context.db_context.connection().await.unwrap();
            store::Pr {
                number: 1.into(),
                commit: Some("73da20569ac857daf6ed4eed70f2f691626b6df3".into()),
            }
            .insert(&mut connection)
            .await
            .unwrap();

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &context.repo_dir(),
                &[
                    wildmatch::WildMatch::new("master"),
                    wildmatch::WildMatch::new("channel*"),
                ],
            )
            .await
            .unwrap();

            assert(
                &mut connection,
                &[(1, "channel1"), (1, "master"), (2, "master")],
            )
            .await;
        }
        .boxed_local()
    })
    .await;
}

struct TestContext<'a> {
    db_context: &'a util::DatabaseContext,
    tempdir: tempfile::TempDir,
}

#[tokio::test]
async fn branch_patterns() {
    TestContext::with(|context| {
        async move {
            let mut connection = context.db_context.connection().await.unwrap();

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &context.repo_dir(),
                &[wildmatch::WildMatch::new("master")],
            )
            .await
            .unwrap();

            assert(&mut connection, &[(1, "master"), (2, "master")]).await;
        }
        .boxed_local()
    })
    .await;
}
