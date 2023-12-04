#![warn(clippy::pedantic)]

use futures::FutureExt;
use sqlx::Connection;

pub use sqlx::PgConnection;

#[derive(Debug, derive_more::From, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrNumber(pub i32);

#[derive(sqlx::FromRow, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Pr {
    pub number: PrNumber,
}

impl Pr {
    /// Inserts provided value into the database.
    ///
    /// # Errors
    ///
    /// See error type for details.
    ///
    /// # Panics
    ///
    /// See [`sqlx::query!`].
    pub async fn insert(self, connection: &mut sqlx::PgConnection) -> sqlx::Result<()> {
        let pr_num: i32 = self.number.into();

        sqlx::query!("INSERT INTO github_prs(number) VALUES ($1)", pr_num)
            .execute(&mut *connection)
            .await?;

        Ok(())
    }

    /// Retrieves all [`Landings`]s.
    ///
    /// # Errors
    ///
    /// See error type for details.
    ///
    /// # Panics
    ///
    /// See [`sqlx::query!`].
    pub async fn all(connection: &mut sqlx::PgConnection) -> Result<Vec<Pr>, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * from github_prs")
            .fetch_all(connection)
            .await
    }
}

#[derive(Debug)]
pub struct PrNumberTooLarge(std::num::TryFromIntError);

impl From<std::num::TryFromIntError> for PrNumberTooLarge {
    fn from(value: std::num::TryFromIntError) -> Self {
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

#[derive(sqlx::FromRow, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Landing {
    pub github_pr_number: PrNumber,
    pub channel: Channel,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, derive_more::From)]
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
        connection: &mut sqlx::PgConnection,
        pr_num: PrNumber,
    ) -> Result<Vec<Channel>, ForPrError> {
        async fn transaction(
            txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
            pr_num: PrNumber,
        ) -> Result<Vec<Channel>, ForPrError> {
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

    /// Retrieves all [`Landings`]s.
    ///
    /// # Errors
    ///
    /// See error type for details.
    ///
    /// # Panics
    ///
    /// See [`sqlx::query!`].
    pub async fn all(connection: &mut sqlx::PgConnection) -> Result<Vec<Landing>, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * from landings")
            .fetch_all(connection)
            .await
    }

    /// Inserts provided value into the database.
    ///
    /// # Errors
    ///
    /// See error type for details.
    pub async fn insert(self, connection: &mut sqlx::PgConnection) -> sqlx::Result<()> {
        async fn transaction(
            txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
            landing: Landing,
        ) -> sqlx::Result<()> {
            let pr_num: i32 = landing.github_pr_number.into();

            let pr = Pr {
                number: pr_num.into(),
            };

            pr.insert(txn).await?;

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
