// PRs in fixture repository:
// #1: master branch1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

use futures::FutureExt;

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
    util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();

            let repo_tempdir = tempfile::tempdir().unwrap();
            let tempdir_path: &camino::Utf8Path = repo_tempdir.path().try_into().unwrap();
            let repo_path = tempdir_path.join("repo");

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &repo_path,
                &vec![wildmatch::WildMatch::new("*")],
            )
            .await
            .unwrap();

            drop(repo_tempdir);

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
    util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();
            store::Pr {
                number: 1.into(),
                commit: Some("73da20569ac857daf6ed4eed70f2f691626b6df3".into()),
            }
            .insert(&mut connection)
            .await
            .unwrap();

            let repo_tempdir = tempfile::tempdir().unwrap();
            let repo_path: camino::Utf8PathBuf =
                repo_tempdir.path().join("repo").try_into().unwrap();

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &repo_path,
                &vec![
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

const NAME: &'static str = "asdf";

fn f(t: impl std::fmt::Debug) {
    drop(t);
}

//<<< struct TestContext<'a> {
//<<<     db_context: &'a util::DatabaseContext,
//<<< }
//<<<
//<<< impl<'a> TestContext<'a> {
//<<<     pub async fn with(
//<<<         test: impl FnOnce(&Self) -> futures::future::LocalBoxFuture<'_, ()> + 'static,
//<<<     ) {
//<<<         let do_with_db = |db_context| {
//<<<             async move {
//<<<                 let test_context = Self { db_context };
//<<<                 test(&test_context).await;
//<<<             }
//<<<             .boxed_local()
//<<<         };
//<<<
//<<<         util::DatabaseContext::with(do_with_db).await;
//<<<     }
//<<< }

#[tokio::test]
async fn branch_patterns() {
    util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();

            // TODO: DRY repo_tempdir
            let repo_tempdir = tempfile::tempdir().unwrap();
            let repo_path: camino::Utf8PathBuf =
                repo_tempdir.path().join("repo").try_into().unwrap();

            fetcher::run(
                &github_repo(),
                &mut connection,
                &github_token(),
                &repo_path,
                &vec![wildmatch::WildMatch::new("master")],
            )
            .await
            .unwrap();

            assert(&mut connection, &[(1, "master"), (2, "master")]).await;
        }
        .boxed_local()
    })
    .await;
}
