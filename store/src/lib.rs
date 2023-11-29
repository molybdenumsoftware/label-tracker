use std::ops::Deref;

use futures::FutureExt;
use sqlx::{migrate::Migrate, Acquire, Connection, FromRow, PgConnection, Postgres, Transaction};

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
        connection: &mut PgConnection,
        //<<< mut connection: impl sqlx::PgExecutor<'_> + Connection,
    ) -> sqlx::Result<()> {
        async fn transaction(
            txn: &mut Transaction<'_, Postgres>,
            landing: Landing,
        ) -> sqlx::Result<()> {
            sqlx::query!(
                "INSERT INTO github_prs(number) VALUES ($1)",
                landing.github_pr_number
            )
            .execute(&mut **txn)
            .await?;

            sqlx::query!(
                "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
                landing.github_pr_number,
                landing.channel
            )
            .execute(&mut **txn)
            .await?;

            Ok(())
        }

        connection
            .transaction(|txn| transaction(txn, self).boxed())
            .await?;
        Ok(())
        // .unwrap();
        //sqlx::query!("insert")
    }
}
