#![warn(clippy::pedantic)]
// required because rocket::launch, remove if clippy permits.
#![allow(clippy::no_effect_underscore_binding)]

use api::app;
use rocket::launch;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(app())
}
