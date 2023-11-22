#![warn(clippy::pedantic)]
// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]
#[macro_use]
extern crate rocket;

use rocket::{
    http::Status,
    serde::{json::Json, Serialize, Deserialize},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Channel(String);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct LandedIn {
    channels: Vec<Channel>,
}

#[get("/landed/github/<pr>")]
fn landed(pr: &str) -> Result<Json<LandedIn>, (Status, &'static str)> {
    Err((Status::NotFound, "PR not found"))
    // Ok(Json(LandedIn {
    //     channels: vec![Channel("master".to_owned())],
    // }))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![landed])
}

#[cfg(test)]
mod test {
    use rocket::{http::Status, local::blocking::Client};

    use crate::{LandedIn, Channel};

    #[test]
    fn pr_not_found() {
        let client = Client::tracked(super::rocket()).unwrap();
        let response = client.get("/landed/github/2134").dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.into_string(), Some("PR not found".into()));
    }

    #[test]
    fn pr_landed_in_master() {
        // <<< TODO: set up some state so 2124 has landed in master >>>
        let client = Client::tracked(super::rocket()).unwrap();
        let response = client.get("/landed/github/2134").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json::<LandedIn>().unwrap(), LandedIn{channels: vec![Channel("master".to_owned())]});
    }
}
