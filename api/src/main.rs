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

use rocket_db_pools::{sqlx::{self, Row}, Database, Connection};

#[derive(Database)]
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

#[get("/landed/github/<pr>")]
async fn landed(mut db: Connection<Data>, pr: &str) -> Result<Json<LandedIn>, (Status, &'static str)> {
    sqlx::query(r#"SELECT "master" as channel"#)
        .fetch_one(&mut **db)
        .await
        .and_then(|r| r.try_column("channel"))
        .ok();
    Err((Status::NotFound, "PR not found"))
    // Ok(Json(LandedIn {
    //     channels: vec![Channel("master".to_owned())],
    // }))
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
    use camino::{Utf8Path, Utf8PathBuf};
    use rocket::{figment::Figment, http::Status, local::blocking::Client, Rocket};
    use std::{
        fs,
        process::{Child, Command},
    };

    use crate::{Channel, LandedIn};

    // IDEA: return a struct that implements drop and kills db when dropped, this means when test exits db is killd (even on panic)

    // If we want to share between tests -> RC
    struct TestContext {
        tmp_dir: tempfile::TempDir,
        postgres: Child,
        // have function that returns something
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            self.postgres.kill().unwrap();
            self.postgres.wait().unwrap();
        }
    }

    impl TestContext {
        // Note: postgres isn't actually going to listen on this port (see the empty
        // listen_addresses down below), this just determines the name of the socket it listens to.
        const PORT: &str = "1";

        fn socket(&self) -> Utf8PathBuf {
            TestContext::sockets_dir(self.tmp_dir.path().try_into().unwrap())
                .join(format!(".s.PGSQL.{}", Self::PORT))
        }

        fn sockets_dir(path: &Utf8Path) -> Utf8PathBuf {
            path.join("sockets")
        }

        fn init() -> Self {
            let tmp_dir = tempfile::tempdir().unwrap();
            let sockets_dir = Self::sockets_dir(tmp_dir.path().try_into().unwrap());
            let data_dir = tmp_dir.path().join("data");
            fs::create_dir(&sockets_dir).unwrap();

            assert!(Command::new("initdb")
                .arg(&data_dir)
                .status()
                .unwrap()
                .success());

            let postgres = Command::new("postgres")
                .arg("-D")
                .arg(data_dir)
                .arg("-p")
                .arg(Self::PORT)
                .arg("-c")
                .arg(format!("unix_socket_directories={sockets_dir}"))
                .arg("-c")
                .arg("listen_addresses=")
                .spawn()
                .unwrap();

            Self { tmp_dir, postgres }
        }

        fn rocket(&self) -> Rocket<rocket::Build> {
            rocket::custom(
                Figment::from(rocket::Config::default())
                    .merge(("databases.data.url", self.db_url())),
            )
            .attach(super::app())
        }

        fn db_url(&self) -> String {
            format!("pgsql://{}", self.socket())
        }
    }

    #[test]
    fn pr_not_found() {
        let ctx = TestContext::init();
        let client = Client::tracked(ctx.rocket()).unwrap();
        let response = client.get("/landed/github/2134").dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.into_string(), Some("PR not found".into()));
    }

    #[test]
    fn pr_landed_in_master() {
        let ctx = TestContext::init();
        // <<< TODO: set up some state so 2124 has landed in master >>>
        let client = Client::tracked(ctx.rocket()).unwrap();
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
