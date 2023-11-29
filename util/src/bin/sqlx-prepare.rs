use std::process::Command;
use util::DatabaseContext;

fn main() {
    let database_ctx = DatabaseContext::init();

    let status = Command::new("cargo")
        .args(["sqlx", "prepare", "--database-url"])
        .arg(database_ctx.db_url())
        .current_dir("store")
        .status()
        .unwrap();

    assert!(status.success());
    println!("hello, world")
}