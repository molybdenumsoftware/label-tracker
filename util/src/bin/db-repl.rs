use std::process::{self, Command};
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    let code = DatabaseContext::with(|database_ctx| {
        let db_url = database_ctx.db_url();
        let status = Command::new("psql").arg(db_url).status().unwrap();
        status.code().unwrap()
    })
    .await;

    process::exit(code)
}
