use std::ops::Deref;

use sqlx::{migrate::Migrate, Acquire, Connection, Database, FromRow, Postgres, Transaction};

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

    pub async fn insert(
        self,
        mut connection: impl sqlx::PgExecutor<'_> + Connection,
    ) -> sqlx::Result<()> {
        async fn transaction(
            txn: &mut Transaction<'_, Postgres>,
            landing: Landing,
        ) -> Result<_, _> {
            sqlx::query!(
                "INSERT INTO github_prs(number) VALUES ($1)",
                landing.github_pr_number
            )
            .execute(transaction)
            .await?;
            sqlx::query!(
                "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
                landing.github_pr_number,
                landing.channel
            )
            .execute(connection)
            .await?;
        }

        connection
            .transaction(|txn| async move { transaction(self) })
            .await?;
        Ok(())
        // .unwrap();
        //sqlx::query!("insert")
    }
}
