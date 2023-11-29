use sqlx::{Connection, FromRow};

pub mod server {}

#[derive(FromRow)]
pub struct Landing {
    pub github_pr: u64,
    pub channel: String,
}

pub async fn migrate(connection: impl Connection) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(connection).await
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub async fn insert(self, connection: impl Connection) -> sqlx::Result<()> {
        // TODO: this isn't gonna compile until we have a running database for sqlx to talk to at
        // build time.
        sqlx::query!(
            "INSERT INTO landings(github_pr, channel) VALUES ($1, $2)",
            self.github_pr,
            self.channel
        )
        .execute(connection)
        .await;
        todo!()
        //sqlx::query!("insert")
    }
}
