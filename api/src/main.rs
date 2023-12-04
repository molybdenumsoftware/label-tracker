#![warn(clippy::pedantic)]

// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]

use rocket::{
    fairing::AdHoc,
    http::Status,
    launch,
    response::{
        content,
        status::{BadRequest, Custom, NotFound},
    },
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_db_pools::{
    sqlx::{self},
    Connection, Database,
};
use store::{ForPrError, Landing, PrNumberTooLarge};

#[launch]
fn rocket() -> _ {
    rocket::build().attach(app())
}

#[cfg(test)]
mod test {
    use rocket::{
        figment::Figment, futures::FutureExt, http::Status, local::asynchronous::Client, Rocket,
    };
    use store::Landing;
    use util::DatabaseContext;

    use crate::{Channel, LandedIn};

    trait Rocketable {
        fn rocket(&self) -> Rocket<rocket::Build>;
    }

    impl Rocketable for DatabaseContext {
        fn rocket(&self) -> Rocket<rocket::Build> {
            rocket::custom(
                Figment::from(rocket::Config::default())
                    .merge(("databases.data.url", self.db_url()))
                    .merge(("log_level", rocket::config::LogLevel::Debug)),
            )
            .attach(super::app())
        }
    }

    #[tokio::test]
    async fn pr_not_found() {
        DatabaseContext::with(|ctx: &DatabaseContext| {
            async {
                let client = Client::tracked(ctx.rocket()).await.unwrap();
                let response = client.get("/landed/github/2134").dispatch().await;
                assert_eq!(response.status(), Status::NotFound);
                assert_eq!(response.into_string().await, Some("Pull request not found.".into()));
            }
            .boxed()
        })
        .await;
    }

    #[tokio::test]
    async fn pr_landed_in_master() {
        DatabaseContext::with(|ctx| {
            async {
                let mut connection = ctx.connection().await.unwrap();

                let landing = Landing {
                    github_pr_number: 2134.try_into().unwrap(),
                    channel: store::Channel::new("nixos-unstable"),
                };

                landing.insert(&mut connection).await.unwrap();

                let client = Client::tracked(ctx.rocket()).await.unwrap();
                let response = client.get("/landed/github/2134").dispatch().await;
                assert_eq!(response.status(), Status::Ok);
                assert_eq!(
                    response.into_json::<LandedIn>().await.unwrap(),
                    LandedIn {
                        channels: vec![Channel("nixos-unstable".to_owned())]
                    }
                );
            }
            .boxed()
        })
        .await;
    }
}
