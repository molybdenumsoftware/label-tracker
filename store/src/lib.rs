#![warn(clippy::pedantic)]

use futures::FutureExt;
use sqlx::Connection;

pub use sqlx::PgConnection;

#[derive(Debug, derive_more::From, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrNumber(pub i32);

#[derive(Debug, derive_more::From, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(pub i32);

#[derive(Debug, derive_more::From, PartialEq, Eq)]
#[from(forward)]
pub struct GitCommit(pub String);

#[derive(sqlx::FromRow, PartialEq, Eq, Debug)]
pub struct Pr {
    pub number: PrNumber,
    pub commit: Option<GitCommit>,
}

impl PartialOrd for Pr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.number.partial_cmp(&other.number)
    }
}

impl Ord for Pr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
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
        sqlx::query!(
            "
            INSERT INTO github_prs(number, commit) VALUES ($1, $2)
            ON CONFLICT DO UPDATE SET commit=$2
            ",
            self.number.0,
            self.commit.0,
        )
        .execute(&mut *connection)
        .await?;

        Ok(())
    }

    /// Bulk insert PRs
    ///
    /// # Errors
    ///
    /// See error type for details.
    pub async fn bulk_insert(
        connection: &mut sqlx::PgConnection,
        prs: Vec<Self>,
    ) -> sqlx::Result<()> {
        // TODO: look into doing a real bulk insert with sqlx
        for pr in prs {
            pr.insert(connection).await?;
        }
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
        sqlx::query!("SELECT * from github_prs")
            .map(|pr| Self {
                number: pr.number.into(),
                commit: pr.commit.map(Into::into),
            })
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
    pub github_pr: PrNumber,
    pub channel_id: ChannelId,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, derive_more::From)]
pub struct Channel {
    id: ChannelId,
    name: String,
}

impl Channel {
    /// Gets or inserts.
    ///
    /// # Errors
    ///
    /// See error type for details.
    pub async fn get_or_insert(
        connection: &mut sqlx::PgConnection,
        s: impl AsRef<str>,
    ) -> sqlx::Result<Self> {
        async fn transaction(
            s: String,
            txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> sqlx::Result<Channel> {
            let channel = sqlx::query_as!(Channel, "SELECT * from channels WHERE name = $1", s);

            todo!();
        }

        let s = s.as_ref().to_owned();
        connection
            .transaction(move |txn| transaction(s, txn).boxed())
            .await
    }

    pub async fn all(
        connection: &mut sqlx::PgConnection,
    ) -> sqlx::Result<std::collections::BTreeMap<ChannelId, Self>> {
        todo!()
    }

    pub fn name(&self) -> &str {
        &self.name
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

            let records = sqlx::query_as!(Channel,
                "SELECT channels.id, channels.name from landings, channels where landings.github_pr = $1 AND landings.channel_id = channels.id",
                pr_num,
            )
            .fetch_all(&mut **txn)
            .await?;

            let channels = records;

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
    pub async fn all(connection: &mut sqlx::PgConnection) -> Result<Vec<Self>, sqlx::Error> {
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
            sqlx::query!(
                "INSERT INTO landings(github_pr, channel_id) VALUES ($1, $2)",
                landing.github_pr.0,
                landing.channel_id.0,
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
