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

pub trait Queryable
where
    Self: Sized,
    Self: Display,
{
    async fn query(conn: sqlx::SqlitePool) -> Result<Option<Self>>;
}
