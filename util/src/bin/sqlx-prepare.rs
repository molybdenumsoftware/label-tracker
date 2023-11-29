use std::process::Command;
use util::DatabaseContext;

fn main() {
    let database_ctx = DatabaseContext::init();

    let status = Command::new("cargo")
        .args(["sqlx", "prepare", "--workspace", "--database-url"])
        .arg(database_ctx.db_url())
        .status()
        .unwrap();

    assert!(status.success());
    println!("hello, world")
}
