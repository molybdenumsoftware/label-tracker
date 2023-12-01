use std::process::Command;
use util::DatabaseContext;

#[tokio::main]
async fn main() {
    DatabaseContext::with(|ctx| {
        //<<< let status = Command::new("cargo")
        //<<<     .args(["sqlx", "prepare", "--database-url"])
        //<<<     .arg(ctx.db_url())
        //<<<     .current_dir("store")
        //<<<     .status()
        //<<<     .unwrap();
        let status = Command::new("false").status().unwrap();
        assert!(status.success()); // <<< is this ok? >>>
    })
    .await;
}
