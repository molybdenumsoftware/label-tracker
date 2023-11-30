#![warn(clippy::pedantic)]
// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]

use std::ops::DerefMut;

use rocket::{
    fairing::AdHoc,
    http::{ContentType, Status},
    launch,
    response::{content, status::BadRequest},
    serde::{json::Json, Deserialize, Serialize},
    Response,
};

use rocket_db_pools::{
    sqlx::{self, Row},
    Connection, Database,
};
use sqlx::PgConnection;
use store::{ForPrError, Landing, PrNumberTooLarge};

#[derive(Database, Debug)]
#[database("data")]
struct Data(sqlx::Pool<sqlx::Postgres>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Channel(String);

impl Channel {

}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct LandedIn {
    channels: Vec<Channel>,
}

enum LandedError {
    PrNumberTooLarge,
    ForPr(ForPrError),
}

impl From<PrNumberTooLarge> for LandedError {
    fn from(value: PrNumberTooLarge) -> Self {
        Self::PrNumberTooLarge
    }
}

impl From<ForPrError> for LandedError {
    fn from(value: ForPrError) -> Self {
        Self::ForPr(value)
    }
}

fn foo<'a, 'b>(foo: &'a str, bar: &'b str) -> &'a str {
    &bar[0..1]
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for LandedError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            LandedError::PrNumberTooLarge => {
                BadRequest(content::RawText("Pull request number too large")).respond_to(request)
            }
            LandedError::ForPr(for_pr_error) => match for_pr_error {
                ForPrError::Sqlx(sqlx_error) => todo!(),
                ForPrError::PrNotFound(_) => todo!(),
            },
        }
    }
}

#[rocket::get("/landed/github/<pr>")]
async fn landed(mut db: Connection<Data>, pr: u32) -> Result<Json<LandedIn>, LandedError> {
    let landings = Landing::for_pr(&mut *db, pr.try_into()?).await?;

    let channels = landings
        .into_iter()
        .map(|landing| Channel::new(landing.channel.as_str()))
        .collect();

    Ok(Json(LandedIn { channels }))
}

fn app() -> AdHoc {
    AdHoc::on_ignite("main", |rocket| async {
        rocket
            .attach(Data::init())
            .mount("/", rocket::routes![landed])
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
}
