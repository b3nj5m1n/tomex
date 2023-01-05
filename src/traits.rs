use std::fmt::Display;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};

pub trait CreateByPrompt {
    async fn create_by_prompt(conn: sqlx::SqlitePool) -> Result<Self>
    where
        Self: Sized;
}

pub trait Insertable {
    async fn insert(self, conn: sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Sized;
}

pub trait DbTable
where
    Self: Sized,
{
    const NAME_SINGULAR: &'static str;
    const NAME_PLURAL: &'static str;
    const TABLE_NAME: &'static str = Self::NAME_PLURAL;
}

pub trait Queryable
where
    for<'r> Self: FromRow<'r, sqlx::sqlite::SqliteRow>,
    Self: DbTable,
    Self: Sized,
    Self: Display,
    Self: Send,
    Self: Unpin,
{
    async fn query(conn: sqlx::SqlitePool) -> Result<Option<Self>> {
        let x: Vec<Self> = sqlx::query_as::<_, Self>(&format!("SELECT * FROM {};", Self::TABLE_NAME))
            .fetch_all(&conn)
            .await?;
        let ans: Option<Self> = inquire::Select::new(&format!("Select {}:", Self::NAME_SINGULAR), x).prompt_skippable()?;
        Ok(ans)
    }
}
