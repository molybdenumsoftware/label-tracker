use std::ops::Deref;

use futures::FutureExt;
use sqlx::{migrate::Migrate, Acquire, Connection, FromRow, PgConnection, Postgres, Transaction};

pub mod server {}

pub struct PrNumber(u32);

impl PrNumber {
    pub fn new(number: u32) -
}

impl From<PrNumber> for i32 {
    fn from(value: PrNumber) -> Self {
        value.0.try_into().unwrap()
    }
}

#[derive(FromRow)]
pub struct Landing {
    pub github_pr_number: PrNumber,
    pub channel: String,
}

pub async fn migrate<'a, A>(connection: A) -> Result<(), sqlx::migrate::MigrateError>
where
    A: Acquire<'a>,
    <A::Connection as Deref>::Target: Migrate,
{
    sqlx::migrate!("./migrations").run(connection).await
}

enum ForPrError {
    Sqlx(sqlx::Error),
    PrNotFound(u64),
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub async fn for_pr(
        connection: &mut PgConnection,
        pr: PrNumber,
    ) -> Result<Vec<Landing>, ForPrError> {
        let rows = sqlx::query!(
            "SELECT channel from landings where github_pr_number = $1",
            pr.into()
        )
        .fetch_all(connection)
        .await?;
        todo!()
    }

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
