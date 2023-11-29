use sqlx::{Connection, FromRow, Result};

pub mod server {}

#[derive(FromRow)]
pub struct Landing {
    pub github_pr: u64,
    pub channel: String,
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub async fn insert(self, connection: impl Connection) -> Result<()> {
        // TODO: this isn't gonna compile until we have a running database for sqlx to talk to at
        // build time.
        //<<< sqlx::query!(
        //<<<     "INSERT INTO landings(github_pr, channel) VALUES (?, ?)",
        //<<<     self.github_pr,
        //<<<     self.channel
        //<<< )
        //<<< .execute(connection)
        //<<< .await;
        todo!()
        //sqlx::query!("insert")
    }
}
