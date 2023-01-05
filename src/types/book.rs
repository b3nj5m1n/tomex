use derive_builder::Builder;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::{
    traits::{CreateByPrompt, Insertable},
    types::{edition::Edition, genre::Genre, review::Review, timestamp::Timestamp, uuid::Uuid},
};

#[derive(Default, Builder, Debug, Clone, PartialEq, Eq)]
#[builder(setter(into))]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    #[builder(setter(into, strip_option), default = "None")]
    pub author: Option<Uuid>,
    #[builder(setter(into, strip_option), default = "Timestamp(None)")]
    pub release_date: Timestamp,
    #[builder(default = "vec![]")]
    pub editions: Vec<Edition>,
    #[builder(default = "vec![]")]
    pub reviews: Vec<Review>,
    #[builder(default = "vec![]")]
    pub genres: Vec<Genre>,
    #[builder(default = "false")]
    pub deleted: bool,
}
impl CreateByPrompt for Book {
    fn create_by_prompt() -> anyhow::Result<Self>
    where
        Self: Sized {
        todo!()
    }
}
impl Insertable for Book {
    async fn insert(self, conn: sqlx::SqlitePool) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized {
        todo!()
    }
}
impl FromRow<'_, SqliteRow> for Book {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            author: row.try_get("author")?,
            release_date: row.try_get("release_date")?,
            editions: vec![], // TODO
            reviews: vec![],
            genres: vec![],
            deleted: row.try_get("deleted")?,
        })
    }
}
