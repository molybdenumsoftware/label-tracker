use sqlx::{Connection, FromRow, Result};

#[derive(FromRow)]
pub struct Landing {
    pub github_pr: u64,
    pub channel: String,
}

impl Landing {
    pub const TABLE: &str = "landings";

    pub fn insert(self, db: impl Connection) -> Result<()> {
        todo!()
        //sqlx::query!("insert")
    }
}
