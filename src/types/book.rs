use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use derives::{Removeable, Id};
use derives::{DbTable, Queryable};
use std::fmt::Display;
use std::fmt::Write;

use derive_builder::Builder;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::traits::Id;
use crate::traits::Removeable;
use crate::{
    traits::{CreateByPrompt, CreateTable, DbTable, DisplayTerminal, Insertable, Queryable},
    types::{edition::Edition, genre::Genre, review::Review, timestamp::Timestamp, uuid::Uuid},
};

use super::author::Author;

#[derive(Default, Builder, Debug, Clone, PartialEq, Eq, DbTable, Queryable, Id, Removeable)]
#[builder(setter(into))]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    #[builder(setter(into, strip_option), default = "None")]
    pub author_id: Option<Uuid>,
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
                    self.title, release_date, self.id.0
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
            write!(f, "[released by {}]", release_date)?;
            write!(f, " ")?; // TODO see above
        }
        write!(f, "({})", self.id)?;
        /* match (&self.release_date.0, Book::author(&self, conn).await?) {
            (None, None) => Ok(format!("{} ({})", self.title, self.id.0)),
            (None, Some(author)) => Ok(format!(
                "{} [written by {}] ({})",
                self.title,
                DisplayTerminal::fmt(&author, conn).await?,
                self.id.0
            )),
            (Some(release_date), None) => Ok(format!(
                "{} [released {}] ({})",
                self.title, release_date, self.id.0
            )),
            (Some(release_date), Some(author)) => Ok(format!(
                "{} [released {}] [written by {}] ({})",
                self.title,
                release_date,
                DisplayTerminal::fmt(&author, conn).await?,
                self.id.0
            )),
        } */
        Ok(())
    }
}
impl CreateByPrompt for Book {
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let title = inquire::Text::new("What title of the book?").prompt()?;
        if title.is_empty() {
            anyhow::bail!("Title is required");
        }
        let author = Author::query_by_prompt(conn).await?;
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
            author_id: author.map(|x| x.id),
            release_date,
            editions: vec![],
            reviews: vec![],
            genres: vec![],
            deleted: false,
        })
    }
}
impl Insertable for Book {
    async fn insert(
        self,
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
