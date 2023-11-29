use std::ops::Deref;

use sqlx::{migrate::Migrate, Acquire, FromRow};

pub mod server {}

#[derive(FromRow)]
pub struct Landing {
    pub github_pr: i32,
    pub channel: String,
}

pub async fn migrate<'a, A>(connection: A) -> Result<(), sqlx::migrate::MigrateError>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations").run(connection).await
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub async fn insert(self, connection: impl sqlx::PgExecutor<'_>) -> sqlx::Result<()> {
        // TODO: this isn't gonna compile until we have a running database for sqlx to talk to at
        // build time.
        sqlx::query!(
            "INSERT INTO landings(github_pr, channel) VALUES ($1, $2)",
            self.github_pr,
            self.channel
        )
        .execute(connection)
        .await?;
        Ok(())
        // .unwrap();
        //sqlx::query!("insert")
    }
}
