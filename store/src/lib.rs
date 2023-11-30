use std::{num::TryFromIntError, ops::Deref};

use futures::FutureExt;
use sqlx::{migrate::Migrate, Acquire, Connection, FromRow, PgConnection, Postgres, Transaction};

#[derive(Debug)]
pub struct PrNumber(i32);

#[derive(Debug)]
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
    pub channel: Channel,
}

pub struct Channel(String);

impl Channel {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
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

impl From<sqlx::Error> for ForPrError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub async fn for_pr(
        connection: &mut PgConnection,
        pr: PrNumber,
    ) -> Result<Vec<Channel>, ForPrError> {
        let pr_num: i32 = pr.into();

        let records = sqlx::query!(
            "SELECT channel from landings where github_pr_number = $1",
            pr_num,
        )
        .fetch_all(connection)
        .await?;

        let landings = records.into_iter().map(|record| record.channel).collect();

        Ok(records)
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
            let pr_num: i32 = landing.github_pr_number.into();

            sqlx::query!("INSERT INTO github_prs(number) VALUES ($1)", pr_num)
                .execute(&mut **txn)
                .await?;

            sqlx::query!(
                "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
                pr_num,
                landing.channel.as_str(),
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
