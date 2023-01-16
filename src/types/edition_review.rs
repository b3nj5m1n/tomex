use anyhow::Result;
use inquire::{Confirm};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{book::Book, text::Text, timestamp::Timestamp, uuid::Uuid},
};
use derives::*;

use super::{edition::Edition, rating::Rating};

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
pub struct EditionReview {
    pub id: Uuid,
    pub edition_id: Uuid,
    pub rating: Option<u32>,
    pub recommend: Option<bool>,
    pub content: Option<Text>,
    pub cover_rating: Option<u32>,
    pub cover_text: Option<Text>,
    pub typesetting_rating: Option<u32>,
    pub typesetting_text: Option<Text>,
    pub material_rating: Option<u32>,
    pub material_text: Option<Text>,
    pub price_rating: Option<u32>,
    pub price_text: Option<Text>,
    pub timestamp_created: Timestamp,
    pub timestamp_updated: Timestamp,
    pub deleted: bool,
    pub book_title: Text,
}

impl EditionReview {
    pub async fn hydrate(&mut self, _conn: &sqlx::SqlitePool) -> Result<()> {
        Ok(())
    }
}

impl PromptType for EditionReview {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let edition = Edition::query_by_prompt(conn).await?;
        let edition_id = edition.id;
        let rating: Option<Rating> = PromptType::create_by_prompt_skippable(
            "What rating would you give this edition? (0-100)",
            None::<&Rating>,
            conn,
        )
        .await?;
        let recommend = Confirm::new("Would you recommend this edition?")
            .with_default(true)
            .prompt_skippable()?;

        Ok(Self {
            id,
            edition_id,
            rating,
            recommend,
            content: None,
            timestamp_created: Timestamp(chrono::Utc::now()),
            timestamp_updated: Timestamp(chrono::Utc::now()),
            book_title: edition.book_title,
            deleted: false,
            cover_rating: None,
            cover_text: None,
            typesetting_rating: None,
            typesetting_text: None,
            material_rating: None,
            material_text: None,
            price_rating: None,
            price_text: None,
        })
    }
    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let edition = Edition::get_by_id(conn, &self.edition_id).await?;
        let rating: Option<Rating> = PromptType::update_by_prompt_skippable(
            &self.rating,
            "What rating would you give this edition? (0-100)",
            conn,
        )
        .await?;
        let recommend = Confirm::new("Would you recommend this edition?")
            .with_default(if let Some(recommend) = &self.recommend {
                *recommend
            } else {
                true
            })
            .prompt_skippable()?;
        let content = inquire::Editor::new("Write a detailed a review:")
            .with_file_extension(".md")
            .with_predefined_text(if let Some(content) = &self.content.clone() {
                &content.0
            } else {
                ""
            })
            .prompt_skippable()?
            .map(Text);

        // Cover
        let cover_rating: Option<Rating> = PromptType::update_by_prompt_skippable(
            &self.cover_rating,
            "What rating would you give this edition's cover? (0-100)",
            conn,
        )
        .await?;
        let cover_text =
            inquire::Editor::new("Write a detailed a review for this edition's cover:")
                .with_file_extension(".md")
                .with_predefined_text(if let Some(content) = &self.cover_text.clone() {
                    &content.0
                } else {
                    ""
                })
                .prompt_skippable()?
                .map(Text);
        // Typesetting
        let typesetting_rating: Option<Rating> = PromptType::update_by_prompt_skippable(
            &self.typesetting_rating,
            "What rating would you give this edition's typesetting? (0-100)",
            conn,
        )
        .await?;
        let typesetting_text =
            inquire::Editor::new("Write a detailed a review for this edition's typesetting:")
                .with_file_extension(".md")
                .with_predefined_text(if let Some(content) = &self.typesetting_text.clone() {
                    &content.0
                } else {
                    ""
                })
                .prompt_skippable()?
                .map(Text);
        // Material
        let material_rating: Option<Rating> = PromptType::update_by_prompt_skippable(
            &self.material_rating,
            "What rating would you give this edition's material? (0-100)",
            conn,
        )
        .await?;
        let material_text =
            inquire::Editor::new("Write a detailed a review for this edition's material:")
                .with_file_extension(".md")
                .with_predefined_text(if let Some(content) = &self.material_text.clone() {
                    &content.0
                } else {
                    ""
                })
                .prompt_skippable()?
                .map(Text);
        // Price
        let price_rating: Option<Rating> = PromptType::update_by_prompt_skippable(
            &self.price_rating,
            "What rating would you give this edition's price? (0-100)",
            conn,
        )
        .await?;
        let price_text =
            inquire::Editor::new("Write a detailed a review for this edition's price:")
                .with_file_extension(".md")
                .with_predefined_text(if let Some(content) = &self.price_text.clone() {
                    &content.0
                } else {
                    ""
                })
                .prompt_skippable()?
                .map(Text);

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
            book_title: edition.book_title,
            cover_rating,
            cover_text,
            typesetting_rating,
            typesetting_text,
            material_rating,
            material_text,
            price_rating,
            price_text,
            ..self.clone()
        };
        Ok(new)
    }

    async fn create_by_prompt_skippable(
        _prompt: &str,
        _initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> Result<Option<Self>> {
        unreachable!("Can't skip creation of this type")
    }

    async fn update_by_prompt_skippable(
        _s: &Option<Self>,
        _prompt: &str,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>>
    where
        Self: Display,
    {
        unreachable!("Can't skip updating this type")
    }
}

impl Display for EditionReview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        write!(
            f,
            "{} ({})",
            self.book_title
                .style(&config.output_edition_review.style_content),
            self.id
        )
    }
}
impl DisplayTerminal for EditionReview {
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let edition = Edition::get_by_id(conn, &s.edition_id).await?;
        let book = Book::get_by_id(conn, &edition.book_id).await?;
        // Book title
        write!(f, "{edition} ")?;
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
                .format_str(s.timestamp_updated, conn, config)
                .await?
        )?;
        // ID
        write!(f, "({})", s.id)?;
        Ok(())
    }
}

impl CreateTable for EditionReview {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE {} (
            	id TEXT PRIMARY KEY NOT NULL,
                edition_id TEXT NOT NULL,
            	rating INT,
            	recommend BOOL,
            	content	TEXT,
            	cover_rating INT,
            	cover_text TEXT,
            	typesetting_rating INT,
            	typesetting_text TEXT,
            	material_rating INT,
            	material_text TEXT,
            	price_rating INT,
            	price_text TEXT,
            	timestamp_created INTEGER,
            	timestamp_updated INTEGER,
            	deleted BOOL DEFAULT FALSE,
                book_title TEXT,
            	FOREIGN KEY (edition_id) REFERENCES {} (id)
            );"#,
            Self::TABLE_NAME,
            Edition::TABLE_NAME,
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl Insertable for EditionReview {
    async fn insert(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult> {
        Ok(sqlx::query(&format!(
            r#"
            INSERT INTO {} ( 
                id, edition_id, rating, recommend, content,
                cover_rating, cover_text, typesetting_rating, typesetting_text,
                material_rating, material_text, price_rating, price_text,
                timestamp_created, timestamp_updated, deleted, book_title )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17 )
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&self.edition_id)
        .bind(self.rating)
        .bind(self.recommend)
        .bind(&self.content)
        .bind(self.cover_rating)
        .bind(&self.cover_text)
        .bind(self.typesetting_rating)
        .bind(&self.typesetting_text)
        .bind(self.material_rating)
        .bind(&self.material_text)
        .bind(self.price_rating)
        .bind(&self.price_text)
        .bind(&self.timestamp_created)
        .bind(&self.timestamp_updated)
        .bind(self.deleted)
        .bind(&self.book_title)
        .execute(conn)
        .await?)
    }
}
impl Updateable for EditionReview {
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                edition_id = ?2,
                rating = ?3,
                recommend = ?4,
                content = ?5,
            	cover_rating = ?6,
            	cover_text = ?7,
            	typesetting_rating = ?8,
            	typesetting_text = ?9,
            	material_rating = ?10,
            	material_text = ?11,
            	price_rating = ?12,
            	price_text = ?13,
                timestamp_created = ?14,
                timestamp_updated = ?15,
                deleted = ?16,
                book_title = ?17
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.edition_id)
        .bind(new.rating)
        .bind(new.recommend)
        .bind(&new.content)
        .bind(new.cover_rating)
        .bind(&new.cover_text)
        .bind(new.typesetting_rating)
        .bind(&new.typesetting_text)
        .bind(new.material_rating)
        .bind(&new.material_text)
        .bind(new.price_rating)
        .bind(&new.price_text)
        .bind(&new.timestamp_created)
        .bind(&new.timestamp_updated)
        .bind(new.deleted)
        .bind(&new.book_title)
        .execute(conn)
        .await?)
    }
}

impl FromRow<'_, SqliteRow> for EditionReview {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            deleted: row.try_get("deleted")?,
            edition_id: row.try_get("edition_id")?,
            rating: row.try_get("rating")?,
            recommend: row.try_get("recommend")?,
            content: row.try_get("content")?,
            timestamp_created: row.try_get("timestamp_created")?,
            timestamp_updated: row.try_get("timestamp_updated")?,
            book_title: row.try_get("book_title")?,
            cover_rating: row.try_get("cover_rating")?,
            cover_text: row.try_get("cover_text")?,
            typesetting_rating: row.try_get("typesetting_rating")?,
            typesetting_text: row.try_get("typesetting_text")?,
            material_rating: row.try_get("material_rating")?,
            material_text: row.try_get("material_text")?,
            price_rating: row.try_get("price_rating")?,
            price_text: row.try_get("price_text")?,
        })
    }
}
