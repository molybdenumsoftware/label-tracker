use std::{os::unix::process::CommandExt, process::Command};
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    let database_ctx = DatabaseContext::init();

    Command::new("psql").arg(database_ctx.await.db_url()).exec();
    println!("this ain't gonna happen");
}
