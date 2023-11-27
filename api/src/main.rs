#![warn(clippy::pedantic)]
// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]
#[macro_use]
extern crate rocket;

use rocket::{
    fairing::AdHoc,
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
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

fn app() -> AdHoc {
    AdHoc::on_ignite("main", |rocket| async {
        rocket.mount("/", routes![landed])
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(app())
}

#[cfg(test)]
mod test {
    use once_cell::sync::Lazy;
    use rocket::{http::Status, local::blocking::Client};
    use std::{
        fs,
        process::{self, Child, Command},
    };

    use crate::{app, Channel, LandedIn};

    // IDEA: return a struct that implements drop and kills db when dropped, this means when test exits db is killd (even on panic)

    // If we want to share between tests -> RC
    struct TestContext {
        dir: tempfile::TempDir,
        postgres: Child,
        // have function that returns something
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            self.postgres.kill().unwrap()
        }
    }

    fn setup_database() -> TestContext {
        static POSTGRES_SERVER: Lazy<Child> = Lazy::new(|| {
            let tmp_dir = tempfile::tempdir().unwrap();
            let sockets_dir = tmp_dir.path().join("sockets");
            let data_dir = tmp_dir.path().join("data");
            std::boxed::Box::<tempfile::TempDir>::leak(Box::new(tmp_dir));
            fs::create_dir(&sockets_dir).unwrap();

            assert!(Command::new("initdb")
                .arg(&data_dir)
                .status()
                .unwrap()
                .success());

            Command::new("postgres")
                .arg("-D")
                .arg(data_dir)
                .arg("-c")
                .arg(format!(
                    "unix_socket_directories={}",
                    sockets_dir.to_str().unwrap()
                ))
                .spawn()
                .unwrap()
        });

        println!(
            "our pid is {} psql pid is {}",
            process::id(),
            POSTGRES_SERVER.id()
        );

        // static DB:
        // postgres -D /tmp/data -c unix_socket_directories=/tmp/psql.sockets
        //<<< rocket::custom(Figment::from(rocket::Config::default()).merge(("databases.data.url", db)))
        todo!()
    }

    #[test]
    fn pr_not_found() {
        //setup_database().attach(app());
        let client = Client::tracked(super::rocket()).unwrap();
        let response = client.get("/landed/github/2134").dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.into_string(), Some("PR not found".into()));
    }

    #[test]
    fn pr_landed_in_master() {
        setup_database();
        // <<< TODO: set up some state so 2124 has landed in master >>>
        let client = Client::tracked(super::rocket()).unwrap();
        let response = client.get("/landed/github/2134").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json::<LandedIn>().unwrap(),
            LandedIn {
                channels: vec![Channel("master".to_owned())]
            }
        );
    }
}
