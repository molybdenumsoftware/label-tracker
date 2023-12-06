use rocket::futures::FutureExt;

trait Rocketable {
    fn rocket(&self) -> rocket::Rocket<rocket::Build>;
}

impl Rocketable for util::DatabaseContext {
    fn rocket(&self) -> rocket::Rocket<rocket::Build> {
        rocket::custom(
            rocket::figment::Figment::from(rocket::Config::default())
                .merge(("databases.data.url", self.db_url()))
                .merge(("log_level", rocket::config::LogLevel::Debug)),
        )
        .attach(api::app())
    }
}

#[tokio::test]
async fn pr_not_found() {
    util::DatabaseContext::with(|ctx: &util::DatabaseContext| {
        async {
            let client = rocket::local::asynchronous::Client::tracked(ctx.rocket())
                .await
                .unwrap();
            let response = client.get("/landed/github/2134").dispatch().await;
            assert_eq!(response.status(), rocket::http::Status::NotFound);
            assert_eq!(
                response.into_string().await,
                Some("Pull request not found.".into())
            );
        }
        .boxed()
    })
    .await;
}

#[tokio::test]
async fn pr_not_landed() {
    util::DatabaseContext::with(|ctx| {
        async {
            let mut connection = ctx.connection().await.unwrap();

            store::Pr {
                number: 123.try_into().unwrap(),
                commit: "deadbeef".into(),
            }
            .insert(&mut connection)
            .await
            .unwrap();

            let client = rocket::local::asynchronous::Client::tracked(ctx.rocket())
                .await
                .unwrap();

            let response = client.get("/landed/github/123").dispatch().await;
            assert_eq!(response.status(), rocket::http::Status::Ok);

            assert_eq!(
                response.into_json::<api::LandedIn>().await.unwrap(),
                api::LandedIn { channels: vec![] }
            );
        }
        .boxed()
    })
    .await;
}

#[tokio::test]
async fn pr_landed() {
    util::DatabaseContext::with(|ctx| {
        async {
            let mut connection = ctx.connection().await.unwrap();

            store::Channel::new("nixos-unstable").insert(&mut connection).await.unwrap();

            let landing = store::Landing {
                github_pr: 2134.try_into().unwrap(),
                channel: store::ChannelNumber(1),
            };

            landing.insert(&mut connection).await.unwrap();

            let client = rocket::local::asynchronous::Client::tracked(ctx.rocket())
                .await
                .unwrap();
            let response = client.get("/landed/github/2134").dispatch().await;
            assert_eq!(response.status(), rocket::http::Status::Ok);
            assert_eq!(
                response.into_json::<api::LandedIn>().await.unwrap(),
                api::LandedIn {
                    channels: vec![api::Channel("nixos-unstable".to_owned())]
                }
            );
        }
        .boxed()
    })
    .await;
}
