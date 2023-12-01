use std::process::Command;
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    let database_ctx = DatabaseContext::init();

    let status = Command::new("cargo")
        .args(["sqlx", "prepare", "--database-url"])
        .arg(database_ctx.await.db_url())
        .current_dir("store")
        .status()
        .unwrap();

    drop(database_ctx);
    assert!(status.success());
}
