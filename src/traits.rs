use anyhow::Result;
use sqlx::sqlite::SqliteQueryResult;

pub trait CreateByPrompt {
    fn create_by_prompt() -> Result<Self>
    where
        Self: Sized;
}

pub trait Insertable {
    async fn insert(self, conn: sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Sized;
}
