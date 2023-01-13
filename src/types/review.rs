use anyhow::Result;
use crossterm::style::Stylize;
use inquire::{validator::Validation, Confirm};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};
use std::fmt::{Display, Write};

use crate::{
    config,
    traits::*,
    types::{book::Book, pace::Pace, text::Text, timestamp::Timestamp, uuid::Uuid},
};
use derives::*;

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Names,
    CRUD,
    Queryable,
    Removeable,
    Id,
    Serialize,
    Deserialize,
)]
pub struct Review {
    pub id: Uuid,
    pub book_id: Uuid,
    pub rating: Option<u32>,
    pub recommend: Option<bool>,
    pub content: Option<Text>,
    pub timestamp_created: Timestamp,
    pub timestamp_updated: Timestamp,
    pub pace_id: Option<Uuid>,
    pub pace: Option<Pace>,
    pub deleted: bool,
    pub book_title: Text,
}

impl Review {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_pace(conn).await?;
        Ok(())
    }
    pub async fn get_pace(&self, conn: &sqlx::SqlitePool) -> Result<Option<Pace>> {
        match &self.pace_id {
            Some(pace_id) => Ok(Some(Pace::get_by_id(conn, pace_id).await?)),
            None => Ok(None),
        }
    }
    pub async fn hydrate_pace(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.pace = self.get_pace(conn).await?;
        Ok(())
    }
}

impl Display for Review {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.book_title, self.id)
    }
}
impl DisplayTerminal for Review {
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        _config: &config::Config,
    ) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let book = Book::get_by_id(conn, &s.book_id).await?;
        // Book title
        write!(f, "{} ", book.title.to_string())?;
        // Rating
        if let Some(rating) = s.rating {
            let str = rating
                .to_string()
                .with(crossterm::style::Color::Rgb {
                    r: 198,
                    g: 160,
                    b: 246,
                })
                .bold();
            write!(f, "[Rating: {}] ", str)?;
        }
        // Recommended
        if let Some(recommended) = s.recommend {
            let str = match recommended {
                true => "Recommended"
                    .with(crossterm::style::Color::Rgb {
                        r: 166,
                        g: 218,
                        b: 149,
                    })
                    .bold(),
                false => "Not Recommended"
                    .with(crossterm::style::Color::Rgb {
                        r: 237,
                        g: 135,
                        b: 150,
                    })
                    .bold(),
            };
            write!(f, "[{}] ", str)?;
        }
        // Pace
        if let Some(pace) = s.pace {
            let str = "Pace".italic();
            write!(f, "[{}: {}]", str, pace)?;
            write!(f, " ")?;
        }
        // Author
        if let Some(authors) = book.get_authors(conn).await? {
            let str = "book written by".italic();
            write!(f, "[{}: ", str)?;
            write!(
                f,
                "{}",
                authors
                    .into_iter()
                    .map(|author| author.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            write!(f, "]",)?;
            write!(f, " ")?;
        }
        // Last updated
        write!(f, "[{} {}]", "Last updated".italic(), s.timestamp_updated)?;
        write!(f, " ")?;
        // ID
        write!(f, "({})", s.id)?;
        Ok(())
    }
}

impl CreateTable for Review {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE {} (
                id TEXT PRIMARY KEY NOT NULL,
                book_id TEXT NOT NULL,
            	rating INT,
            	recommend BOOL,
            	content	TEXT,
            	timestamp_created INTEGER,
            	timestamp_updated INTEGER,
            	pace_id INT,
            	deleted BOOL DEFAULT FALSE,
                book_title TEXT,
            	FOREIGN KEY (book_id) REFERENCES {} (id),
            	FOREIGN KEY (pace_id) REFERENCES {} (id)
            );"#,
            Self::TABLE_NAME,
            Book::TABLE_NAME,
            Pace::TABLE_NAME
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl Insertable for Review {
    async fn insert(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult> {
        Ok(sqlx::query(
            r#"
            INSERT INTO reviews ( id, book_id, rating, recommend, content, timestamp_created, timestamp_updated, pace_id, deleted, book_title )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10 )
            "#,
        )
        .bind(&self.id)
        .bind(&self.book_id)
        .bind(&self.rating)
        .bind(&self.recommend)
        .bind(&self.content)
        .bind(&self.timestamp_created)
        .bind(&self.timestamp_updated)
        .bind(&self.pace_id)
        .bind(&self.deleted)
        .bind(&self.book_title)
        .execute(conn)
        .await?)
    }
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let book = Book::query_by_prompt(conn).await?;
        let book_id = book.id;
        let validator = |input: &str| match input.parse::<u32>() {
            Ok(n) => {
                if n <= 100 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(
                            "Rating has to be between 0-100".to_string(),
                        ),
                    ))
                }
            }
            Err(_) => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
            )),
        };
        let rating = inquire::Text::new("What rating would you give this book? (0-100)")
            .with_validator(validator)
            .prompt_skippable()?
            .map(|x| x.parse::<u32>().expect("Unreachable"));
        let recommend = Confirm::new("Would you recommend this book?")
            .with_default(true)
            .prompt_skippable()?;
        let pace = Pace::query_by_prompt_skippable(conn).await?;
        let pace_id = pace.clone().map(|x| x.id);

        Ok(Self {
            id,
            book_id,
            rating,
            recommend,
            content: None,
            timestamp_created: Timestamp(chrono::Utc::now()),
            timestamp_updated: Timestamp(chrono::Utc::now()),
            pace_id,
            pace,
            book_title: book.title,
            deleted: false,
        })
    }
}
impl Updateable for Review {
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                book_id = ?2,
                rating = ?3,
                recommend = ?4,
                content = ?5,
                timestamp_created = ?6,
                timestamp_updated = ?7,
                pace_id = ?8,
                deleted = ?9,
                book_title = ?10
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.book_id)
        .bind(&new.rating)
        .bind(&new.recommend)
        .bind(&new.content)
        .bind(&new.timestamp_created)
        .bind(&new.timestamp_updated)
        .bind(&new.pace_id)
        .bind(&new.deleted)
        .bind(&new.book_title)
        .execute(conn)
        .await?)
    }

    async fn update_by_prompt(&mut self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable,
    {
        let book = Book::get_by_id(conn, &self.book_id).await?;
        let validator = |input: &str| match input.parse::<u32>() {
            Ok(n) => {
                if n <= 100 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(
                            "Rating has to be between 0-100".to_string(),
                        ),
                    ))
                }
            }
            Err(_) => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
            )),
        };
        let rating = inquire::Text::new("What rating would you give this book? (0-100)")
            .with_validator(validator)
            .with_initial_value(
                if let Some(rating) = &self.rating.clone().map(|x| x.to_string()) {
                    &rating
                } else {
                    ""
                },
            )
            .prompt_skippable()?
            .map(|x| x.parse::<u32>().expect("Unreachable"));
        let recommend = Confirm::new("Would you recommend this book?")
            .with_default(if let Some(recommend) = &self.recommend {
                *recommend
            } else {
                true
            })
            .prompt_skippable()?;
        let pace = Pace::query_by_prompt_skippable(conn).await?;
        let pace_id = pace.clone().map(|x| x.id);

        let content = inquire::Editor::new("Write a detailed a review:")
            .with_file_extension(".md")
            .with_predefined_text(if let Some(content) = &self.content.clone() {
                &content.0
            } else {
                ""
            })
            .prompt_skippable()?
            .map(|x| Text(x));

        if !inquire::Confirm::new("Update review?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };

        let new = Self {
            rating,
            recommend,
            content,
            timestamp_updated: Timestamp(chrono::Utc::now()),
            pace_id,
            pace,
            book_title: book.title,
            ..self.clone()
        };
        Self::update(self, conn, new).await
    }
}

impl FromRow<'_, SqliteRow> for Review {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            deleted: row.try_get("deleted")?,
            book_id: row.try_get("book_id")?,
            rating: row.try_get("rating")?,
            recommend: row.try_get("recommend")?,
            content: row.try_get("content")?,
            timestamp_created: row.try_get("timestamp_created")?,
            timestamp_updated: row.try_get("timestamp_updated")?,
            pace_id: row.try_get("pace_id")?,
            pace: None,
            book_title: row.try_get("book_title")?,
        })
    }
}
