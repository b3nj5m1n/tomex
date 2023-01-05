use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use derive_builder::Builder;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::{
    traits::{CreateByPrompt, Insertable, Queryable},
    types::{edition::Edition, genre::Genre, review::Review, timestamp::Timestamp, uuid::Uuid},
};

use super::author::Author;

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
    async fn create_by_prompt(conn: sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let title = inquire::Text::new("What title of the book?").prompt()?;
        if title.is_empty() {
            anyhow::bail!("Title is required");
        }
        let author = Author::query(conn).await?;
        let release_date = Timestamp(
            inquire::DateSelect::new("What was the book released?")
                .prompt_skippable()?
                .map(|x| {
                    DateTime::from_utc(
                        NaiveDateTime::new(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        Utc,
                    )
                }),
        );
        if !inquire::Confirm::new("Add book?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };
        Ok(Self {
            id,
            title,
            author: author.map(|x| x.id),
            release_date,
            editions: vec![],
            reviews: vec![],
            genres: vec![],
            deleted: false,
        })
    }
}
impl Insertable for Book {
    async fn insert(self, conn: sqlx::SqlitePool) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized,
    {
        Ok(sqlx::query(
            r#"
                    INSERT INTO books ( id, title, author, release_date, deleted )
                    VALUES ( ?1, ?2, ?3, ?4, ?5 )
                    "#,
        )
        .bind(&self.id)
        .bind(&self.title)
        .bind(&self.author)
        .bind(&self.release_date)
        .bind(&self.deleted)
        .execute(&conn)
        .await?)
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
