use sqlx::{FromRow, Connection, Result};

#[derive(FromRow)]
pub struct GithubPr {
    pub number: u64,
}

impl GithubPr {
    pub fn insert(self, db: impl Connection) -> Result<()> {
        sqlx::query!("insert")
    }
}
