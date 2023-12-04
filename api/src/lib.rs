#![warn(clippy::pedantic)]

use rocket_db_pools::Database;

fn app() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("main", |rocket| async {
        rocket
            .attach(Data::init())
            .mount("/", rocket::routes![landed])
    })
}

#[derive(rocket_db_pools::Database, Debug)]
#[database("data")]
struct Data(sqlx::Pool<sqlx::Postgres>);

#[rocket::get("/landed/github/<pr>")]
async fn landed(mut db: Connection<Data>, pr: u32) -> Result<Json<LandedIn>, LandedError> {
    let landings = Landing::for_pr(&mut db, pr.try_into()?).await?;

    let channels = landings
        .into_iter()
        .map(|channel| Channel::new(channel.as_str()))
        .collect();

    Ok(Json(LandedIn { channels }))
}

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
                ForPrError::PrNotFound => {
                    NotFound(content::RawText("Pull request not found.")).respond_to(request)
                }
            },
        }
    }
}
