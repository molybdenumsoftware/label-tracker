use std::{num::TryFromIntError, ops::Deref};

use futures::FutureExt;
use sqlx::{migrate::Migrate, Acquire, Connection, FromRow, PgConnection, Postgres, Transaction};

pub mod server {}

pub struct PrNumber(i32);

pub struct PrNumberTooLarge(TryFromIntError);

impl From<TryFromIntError> for PrNumberTooLarge {
    fn from(value: TryFromIntError) -> Self {
        Self(value)
    }
}

impl TryFrom<u32> for PrNumber {
    type Error = PrNumberTooLarge;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl From<PrNumber> for i32 {
    fn from(value: PrNumber) -> Self {
        value.0
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

pub enum ForPrError {
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
            Some(32),
            //<<< pr.into()
        )
        .fetch_all(connection);
        //<<< .await?;
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
                83,
                //<<< landing.github_pr_number
            )
            .execute(&mut **txn)
            .await?;

            sqlx::query!(
                "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
                83,
                //<<< landing.github_pr_number,
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
