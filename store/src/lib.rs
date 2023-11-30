use std::{
    num::TryFromIntError,
    ops::{Deref, DerefMut},
};

use futures::FutureExt;
use sqlx::{
    migrate::Migrate, postgres::PgTypeInfo, Acquire, Connection, FromRow, PgConnection, Postgres,
    Transaction,
};

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
    pub channel: Channel,
}

pub struct Channel(String);

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
            let pr_num: i32 = landing.github_pr_number.into();

            sqlx::query!("INSERT INTO github_prs(number) VALUES ($1)", pr_num)
                .execute(&mut **txn)
                .await?;

            let channel: String = landing.channel.into();
            sqlx::query!(
                "INSERT INTO landings(github_pr_number, channel) VALUES ($1, $2)",
                pr_num,
                channel,
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
