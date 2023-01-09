use anyhow::Result;

use derives::{Names, Queryable};
use derives::{Id, Removeable};
use std::fmt::Display;
use std::fmt::Write;

use derive_builder::Builder;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::traits::Removeable;
use crate::traits::{Id, PromptType};
use crate::{
    traits::{CreateTable, Names, DisplayTerminal, Insertable, Queryable},
    types::{edition::Edition, genre::Genre, review::Review, timestamp::Timestamp, uuid::Uuid},
};

use super::author::Author;
use super::text::Text;
use super::timestamp::OptionalTimestamp;

#[derive(Default, Builder, Debug, Clone, PartialEq, Eq, Names, Queryable, Id, Removeable)]
#[builder(setter(into))]
pub struct Book {
    pub id: Uuid,
    pub title: Text,
    #[builder(setter(into, strip_option), default = "None")]
    pub author_id: Option<Uuid>,
    #[builder(setter(into, strip_option), default = "OptionalTimestamp(None)")]
    pub release_date: OptionalTimestamp,
    #[builder(default = "vec![]")]
    pub editions: Vec<Edition>,
    #[builder(default = "vec![]")]
    pub reviews: Vec<Review>,
    #[builder(default = "vec![]")]
    pub genres: Vec<Genre>,
    #[builder(default = "false")]
    pub deleted: bool,
}

impl Book {
    pub async fn author(&self, conn: &sqlx::SqlitePool) -> Result<Option<Author>> {
        match &self.author_id {
            Some(id) => {
                let author = Author::get_by_id(conn, id.clone()).await?;
                Ok(Some(author))
            }
            None => Ok(None),
        }
    }
}

impl CreateTable for Book {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                author TEXT,
                release_date INTEGER,
                deleted BOOL DEFAULT FALSE,
                FOREIGN KEY (author) REFERENCES authors (id)
            );"#,
            Self::TABLE_NAME
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}
impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.release_date.0 {
            None => write!(f, "{} ({})", self.title, self.id.0),
            Some(release_date) => {
                write!(
                    f,
                    "{}, released {} ({})",
                    self.title, release_date.0, self.id.0
                )
            }
        }
    }
}
impl DisplayTerminal for Book {
    async fn fmt(&self, f: &mut String, conn: &sqlx::SqlitePool) -> Result<()> {
        write!(f, "{}", self.title)?;
        write!(f, " ")?; // TODO firgure out how to use Formatter to avoid this
        if let Some(author) = Book::author(&self, conn).await? {
            write!(f, "[written by",)?;
            write!(f, " ")?; // TODO see above
            DisplayTerminal::fmt(&author, f, conn).await?;
            write!(f, "]",)?;
            write!(f, " ")?; // TODO see above
        }
        if let Some(release_date) = &self.release_date.0 {
            write!(f, "[released by {}]", release_date.0)?;
            write!(f, " ")?; // TODO see above
        }
        write!(f, "({})", self.id)?;
        Ok(())
    }
}
impl Insertable for Book {
    async fn insert(
        &self,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
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
        .bind(&self.author_id)
        .bind(&self.release_date)
        .bind(&self.deleted)
        .execute(conn)
        .await?)
    }
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let title = Text::create_by_prompt("What is the title of the book?", None)?;
        let author = Author::query_or_create_by_prompt_skippable(conn).await?;
        let release_date = OptionalTimestamp(Timestamp::create_by_prompt_skippable(
            "When was the book released?",
            None,
        )?);
        Ok(Self {
            id,
            title,
            author_id: author.map(|x| x.id),
            release_date,
            editions: vec![],
            reviews: vec![],
            genres: vec![],
            deleted: false,
        })
    }
}
impl FromRow<'_, SqliteRow> for Book {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            author_id: row.try_get("author")?,
            release_date: row.try_get("release_date")?,
            editions: vec![], // TODO
            reviews: vec![],
            genres: vec![],
            deleted: row.try_get("deleted")?,
        })
    }
}
