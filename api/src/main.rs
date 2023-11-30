#![warn(clippy::pedantic)]
// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]
#[macro_use]
extern crate rocket;

use rocket::{
    fairing::AdHoc,
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_db_pools::{
    sqlx::{self, Row},
    Connection, Database,
};
use sqlx::PgConnection;
use store::Landing;

#[derive(Database, Debug)]
#[database("data")]
struct Data(sqlx::Pool<sqlx::Postgres>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Channel(String);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct LandedIn {
    channels: Vec<Channel>,
}

#[derive(Responder)]
enum LandedError {
    #[response(status = 404)]
    PrNotFound(()),
    #[response(status = 500)]
    Db(()),
}

impl From<sqlx::Error> for LandedError {
    fn from(_value: sqlx::Error) -> Self {
        Self::Db(())
    }
}

#[get("/landed/github/<pr>")]
async fn landed(mut db: PgConnection, pr: u64) -> Result<Json<LandedIn>, LandedError> {
    let landings = Landing::for_pr(db, pr).await?;

    let channels = rows
        .into_iter()
        .map(|row| row.get::<String, _>("channel"))
        .map(Channel)
        .collect();

    Ok(Json(LandedIn { channels }))
}

fn app() -> AdHoc {
    AdHoc::on_ignite("main", |rocket| async {
        rocket.attach(Data::init()).mount("/", routes![landed])
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(app())
}

#[cfg(test)]
mod test {
    use rocket::{figment::Figment, http::Status, local::asynchronous::Client, Rocket};
    use sqlx::{Connection, PgConnection};
    use store::Landing;
    use util::DatabaseContext;

    use crate::{Channel, LandedIn};

    struct TestContext {
        database_ctx: DatabaseContext,
    }

    impl TestContext {
        async fn init() -> Self {
            let database_ctx = DatabaseContext::init().await;
            Self { database_ctx }
        }

        async fn connection(&self) -> Result<PgConnection, sqlx::Error> {
            let url = self.database_ctx.db_url();
            sqlx::PgConnection::connect(&url).await
        }

        fn rocket(&self) -> Rocket<rocket::Build> {
            rocket::custom(
                Figment::from(rocket::Config::default())
                    .merge(("databases.data.url", self.database_ctx.db_url()))
                    .merge(("log_level", rocket::config::LogLevel::Debug)),
            )
            .attach(super::app())
        }
    }

    #[tokio::test]
    async fn pr_not_found() {
        let ctx = TestContext::init().await;
        let client = Client::tracked(ctx.rocket()).await.unwrap();
        let response = client.get("/landed/github/2134").dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.into_string().await, Some("PR not found".into()));
    }

    #[tokio::test]
    async fn pr_landed_in_master() {
        let ctx = TestContext::init().await;
        let mut connection = ctx.connection().await.unwrap();

        let landing = Landing {
            github_pr_number: 2134,
            channel: "nixos-unstable".to_string(),
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
}
