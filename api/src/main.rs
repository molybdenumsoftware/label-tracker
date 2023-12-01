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

#[derive(Database, Debug)]
#[database("data")]
struct Data(sqlx::Pool<sqlx::Postgres>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Channel(String);

impl Channel {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().to_string())
    }
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
    fn from(_value: PrNumberTooLarge) -> Self {
        Self::PrNumberTooLarge
    }
}

impl From<ForPrError> for LandedError {
    fn from(value: ForPrError) -> Self {
        Self::ForPr(value)
    }
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for LandedError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            LandedError::PrNumberTooLarge => {
                BadRequest(content::RawText("Pull request number too large.")).respond_to(request)
            }
            LandedError::ForPr(for_pr_error) => match for_pr_error {
                ForPrError::Sqlx(_sqlx_error) => {
                    let status = Status::from_code(500).unwrap();
                    Custom(status, content::RawText("Error. Sorry.")).respond_to(request)
                }
                ForPrError::PrNotFound(_) => {
                    NotFound(content::RawText("Pull request not found.")).respond_to(request)
                }
            },
        }
    }
}

#[rocket::get("/landed/github/<pr>")]
async fn landed(mut db: Connection<Data>, pr: u32) -> Result<Json<LandedIn>, LandedError> {
    let landings = Landing::for_pr(&mut db, pr.try_into()?).await?;

    let channels = landings
        .into_iter()
        .map(|channel| Channel::new(channel.as_str()))
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
    use rocket::{
        figment::Figment, futures::FutureExt, http::Status, local::asynchronous::Client, Rocket,
    };
    use sqlx::{Connection, PgConnection};
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
                assert_eq!(response.into_string().await, Some("PR not found".into()));
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
