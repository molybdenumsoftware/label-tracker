#![warn(clippy::pedantic)]

use std::{collections::BTreeSet, num::TryFromIntError};

use futures::FutureExt;
use sqlx::{Connection, FromRow, PgConnection, Postgres, Transaction};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Channel(String);

impl Channel {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub enum ForPrError {
    Sqlx(sqlx::Error),
    PrNotFound,
}

impl From<sqlx::Error> for ForPrError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl Landing {
    pub const TABLE: &str = "landings";

    /// Retrieves all [`Channel`]s this PR has landed in.
    ///
    /// # Errors
    ///
    /// See error type for details.
    ///
    /// # Panics
    ///
    /// See [`sqlx::query!`].
    pub async fn for_pr(
        connection: &mut PgConnection,
        pr_num: PrNumber,
    ) -> Result<BTreeSet<Channel>, ForPrError> {
        async fn transaction(
            txn: &mut Transaction<'_, Postgres>,
            pr_num: PrNumber,
        ) -> Result<BTreeSet<Channel>, ForPrError> {
            let pr_num: i32 = pr_num.into();

            let exists = sqlx::query!("SELECT 1 as pr from github_prs where number = $1", pr_num)
                .fetch_optional(&mut **txn)
                .await?
                .is_some();

            if !exists {
                return Err(ForPrError::PrNotFound);
            }

            let records = sqlx::query!(
                "SELECT channel from landings where github_pr_number = $1",
                pr_num,
            )
            .fetch_all(&mut **txn)
            .await?;

            let channels = records
                .into_iter()
                .map(|record| Channel::new(record.channel))
                .collect();

            Ok(channels)
        }

        let channels = connection
            .transaction(|txn| transaction(txn, pr_num).boxed())
            .await?;

        Ok(channels)
    }

    /// Inserts provided value into the database.
    ///
    /// # Errors
    ///
    /// See error type for details.
    pub async fn insert(self, connection: &mut PgConnection) -> sqlx::Result<()> {
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
    }
}
