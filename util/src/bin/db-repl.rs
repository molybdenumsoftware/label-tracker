use std::process::{self, Command};
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    let code = {
        let database_ctx = DatabaseContext::init().await;
        let db_url = database_ctx.db_url();
        let status = Command::new("psql").arg(db_url).status().unwrap();
        drop(database_ctx);
        status.code().unwrap()
    };

    process::exit(code)
}
