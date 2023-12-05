// PRs in fixture repository:
// #1: master channel1
// #2: master
// #3:

// TODO: consider asserting fixture repo state

use futures::FutureExt;
use store::Landing;

async fn assert_landings(connection: &mut store::PgConnection) {
    let mut landings = store::Landing::all(connection).await.unwrap();
    landings.sort();

    assert_eq!(
        landings,
        [
            store::Landing {
                github_pr_number: store::PrNumber::from(1),
                channel: store::Channel::new("channel1")
            },
            store::Landing {
                github_pr_number: store::PrNumber::from(1),
                channel: store::Channel::new("master")
            },
            store::Landing {
                github_pr_number: store::PrNumber::from(2),
                channel: store::Channel::new("master")
            },
        ]
    );

    let mut prs = store::Pr::all(connection).await.unwrap();
    prs.sort();

    assert_eq!(
        prs,
        [
            store::Pr {
                number: 1.into(),
                commit: "a".into()
            },
            store::Pr {
                number: 2.into(),
                commit: "b".into()
            },
            store::Pr {
                number: 3.into(),
                commit: "c".into()
            },
        ]
    );
}

fn github_repo() -> fetcher::GitHubRepo {
    "molybdenumsoftware/label-tracker-test-fixture"
        .parse()
        .unwrap()
}

#[tokio::test]
async fn insert_prs() {
    util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();

            fetcher::run(&github_repo(), &mut connection);

            assert_landings(&mut connection).await;
        }
        .boxed()
    })
    .await;
}

#[tokio::test]
async fn update_pr() {
    util::DatabaseContext::with(|context| {
        async move {
            let mut connection = context.connection().await.unwrap();
            store::Pr {
                number: 1.into(),
                commit: "a".into(),
            }
            .insert(&mut connection)
            .await
            .unwrap();

            fetcher::run(&github_repo(), &mut connection);

            assert_landings(&mut connection).await;
        }
        .boxed()
    })
    .await;
}
