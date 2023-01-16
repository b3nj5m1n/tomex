use anyhow::Result;
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
    types::{book::Book, mood::Mood, pace::Pace, text::Text, timestamp::Timestamp, uuid::Uuid},
};
use derives::*;

use super::review_mood::ReviewMood;

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
    pub moods: Option<Vec<Mood>>,
}

impl Review {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_pace(conn).await?;
        self.hydrate_moods(conn).await?;
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
    pub async fn get_moods(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Mood>>> {
        let result = ReviewMood::get_all_for_a(conn, self).await?;
        Ok(if !result.is_empty() { Some(result) } else { None })
    }
    pub async fn hydrate_moods(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.moods = self.get_moods(conn).await?;
        Ok(())
    }
}

impl PromptType for Review {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
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
            moods: None,
        })
    }
    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let book = Book::get_by_id(conn, &s.book_id).await?;
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
                if let Some(rating) = &s.rating.map(|x| x.to_string()) {
                    rating
                } else {
                    ""
                },
            )
            .prompt_skippable()?
            .map(|x| x.parse::<u32>().expect("Unreachable"));
        let recommend = Confirm::new("Would you recommend this book?")
            .with_default(if let Some(recommend) = &s.recommend {
                *recommend
            } else {
                true
            })
            .prompt_skippable()?;
        let pace = match Pace::query_by_prompt_skippable(conn).await? {
            Some(pace) => Some(pace),
            None => s.pace.clone(),
        };
        let pace_id = pace.clone().map(|x| x.id);

        let content = inquire::Editor::new("Write a detailed a review:")
            .with_file_extension(".md")
            .with_predefined_text(if let Some(content) = &s.content.clone() {
                &content.0
            } else {
                ""
            })
            .prompt_skippable()?
            .map(Text);

        let moods = Mood::update_vec(&s.moods, conn, "Select moods for this edition:").await?;

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
            moods,
            ..s.clone()
        };
        Ok(new)
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
        config: &config::Config,
    ) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let book = Book::get_by_id(conn, &s.book_id).await?;
        // Book title
        write!(f, "{book} ")?;
        // Rating
        if let Some(rating) = s.rating {
            write!(
                f,
                "{} ",
                config
                    .output_rating
                    .format_str(rating.to_string(), conn, config)
                    .await?
            )?;
        }
        // Recommended
        if let Some(recommended) = s.recommend {
            let str = match recommended {
                true => {
                    config
                        .output_recommended_true
                        .format_str("YES", conn, config)
                        .await?
                }
                false => {
                    config
                        .output_recommended_false
                        .format_str("NO", conn, config)
                        .await?
                }
            };
            write!(f, "{str} ")?;
        }
        // Pace
        if let Some(pace) = s.pace {
            write!(
                f,
                "{} ",
                config.output_pace.format(pace, conn, config).await?
            )?;
        }
        // Moods
        if let Some(moods) = s.moods {
            write!(
                f,
                "{} ",
                config.output_mood.format_vec(moods, conn, config).await?
            )?;
        }
        // Author
        if let Some(authors) = book.get_authors(conn).await? {
            write!(
                f,
                "{} ",
                config
                    .output_author
                    .format_vec(authors, conn, config)
                    .await?
            )?;
        }
        // Last updated
        write!(
            f,
            "{} ",
            config
                .output_last_updated
                .format_str(s.timestamp_updated.to_string(), conn, config)
                .await?
        )?;
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
        let result = sqlx::query(
            r#"
            INSERT INTO reviews ( id, book_id, rating, recommend, content, timestamp_created, timestamp_updated, pace_id, deleted, book_title )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10 )
            "#,
        )
        .bind(&self.id)
        .bind(&self.book_id)
        .bind(self.rating)
        .bind(self.recommend)
        .bind(&self.content)
        .bind(&self.timestamp_created)
        .bind(&self.timestamp_updated)
        .bind(&self.pace_id)
        .bind(self.deleted)
        .bind(&self.book_title)
        .execute(conn)
        .await?;

        ReviewMood::update(conn, self, &None, &self.moods).await?;

        Ok(result)
    }
}
impl Updateable for Review {
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
        ReviewMood::update(conn, self, &self.moods, &new.moods).await?;
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
        .bind(new.rating)
        .bind(new.recommend)
        .bind(&new.content)
        .bind(&new.timestamp_created)
        .bind(&new.timestamp_updated)
        .bind(&new.pace_id)
        .bind(new.deleted)
        .bind(&new.book_title)
        .execute(conn)
        .await?)
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
            moods: None,
        })
    }
}
