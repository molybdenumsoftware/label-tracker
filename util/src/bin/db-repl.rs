use std::process::{self, Command};
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    let database_ctx = DatabaseContext::init();
    let db_url = database_ctx.await.db_url();
    let status = Command::new("psql").arg(db_url).status().unwrap();
    drop(database_ctx);
    process::exit(status.code().unwrap());
}
