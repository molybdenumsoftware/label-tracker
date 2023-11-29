use std::ops::Deref;

use sqlx::{migrate::Migrate, Acquire, FromRow, Connection};

pub mod server {}

#[derive(FromRow)]
pub struct Landing {
    pub github_pr_number: i32,
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

    pub async fn insert(self, mut connection: impl sqlx::PgExecutor<'_> + Connection) -> sqlx::Result<()> {
        connection.transaction(|txn| async move {
        sqlx::query!("INSERT INTO github_prs(number) VALUES ($1)", self.github_pr_number)
            .execute(transaction)
            .await?;
        sqlx::query!(
            "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
            self.github_pr_number,
            self.channel
        )
        .execute(connection)
        .await?;
            
        }).await?;
        Ok(())
        // .unwrap();
        //sqlx::query!("insert")
    }
}
