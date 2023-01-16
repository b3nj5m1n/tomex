use anyhow::Result;
use inquire::validator::Validation;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{edition::Edition, timestamp::Timestamp, uuid::Uuid},
};
use derives::*;

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    FromRow,
    Id,
    Names,
    CRUD,
    Queryable,
    Removeable,
    Serialize,
    Deserialize,
)]
pub struct Progress {
    pub id:             Uuid,
    pub edition_id:     Uuid,
    pub timestamp:      Timestamp,
    pub pages_progress: u32,
    pub deleted:        bool,
}

impl PromptType for Progress {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let edition = Edition::query_by_prompt(conn).await?;
        let timestamp =
            Timestamp::create_by_prompt("For when is this progress update?", None, conn).await?;
        let max_pages = edition.pages;
        let validator = move |input: &str| match input.parse::<u32>() {
            Ok(n) => {
                if let Some(max_pages) = max_pages {
                    if n <= max_pages {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(
                                "Input has to be lower than number of pages in edition".to_string(),
                            ),
                        ))
                    }
                } else {
                    Ok(Validation::Valid)
                }
            }
            Err(_) => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
            )),
        };
        let pages_progress = inquire::Text::new("At which page are you?")
            .with_validator(validator)
            .prompt()?
            .parse::<u32>()
            .expect("Unreachable");
        Ok(Self {
            id,
            edition_id: edition.id,
            timestamp,
            pages_progress,
            deleted: false,
        })
    }

    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let edition = Edition::get_by_id(conn, &self.edition_id).await?;
        let timestamp = PromptType::update_by_prompt(
            &self.timestamp,
            "For when is this progress update?",
            conn,
        )
        .await?;
        let max_pages = edition.pages;
        let validator = move |input: &str| match input.parse::<u32>() {
            Ok(n) => {
                if let Some(max_pages) = max_pages {
                    if n <= max_pages {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(
                                "Input has to be lower than number of pages in edition".to_string(),
                            ),
                        ))
                    }
                } else {
                    Ok(Validation::Valid)
                }
            }
            Err(_) => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
            )),
        };
        let pages_progress = inquire::Text::new("At which page are you?")
            .with_validator(validator)
            .prompt()?
            .parse::<u32>()
            .expect("Unreachable");
        let new = Self {
            timestamp,
            pages_progress,
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

impl Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        write!(
            f,
            "{}: {} pages",
            self.timestamp,
            self.pages_progress
                .to_string()
                .style(&config.output_progress.style_content)
        )?;
        if config.output_progress.display_uuid {
            write!(f, " ({})", self.id)
        } else {
            Ok(())
        }
    }
}
impl DisplayTerminal for Progress {
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let edition = Edition::get_by_id(conn, &self.edition_id).await?;
        let title = edition.to_string();
        write!(
            f,
            "{} {}: {} pages",
            title,
            self.timestamp,
            self.pages_progress
                .to_string()
                .style(&config.output_progress.style_content)
        )?;
        if config.output_progress.display_uuid {
            write!(f, " ({})", self.id)?;
        }
        Ok(())
    }
}

impl CreateTable for Progress {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
            	edition_id	TEXT	NOT NULL,
            	timestamp   INTEGER	NOT NULL,
            	pages_progress	INT	NOT NULL,
                deleted BOOL DEFAULT FALSE,
            	FOREIGN KEY (edition_id) REFERENCES {} (id)
            );

            "#,
            Self::TABLE_NAME,
            Edition::TABLE_NAME
        ))
        .execute(conn)
        .await?;

        Ok(())
    }
}

impl Insertable for Progress {
    async fn insert(
        &self,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized,
    {
        Ok(sqlx::query(&format!(
            r#"
                    INSERT INTO {} ( id, edition_id, timestamp, pages_progress, deleted )
                    VALUES ( ?1, ?2, ?3, ?4, ?5 )
                    "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&self.edition_id)
        .bind(&self.timestamp)
        .bind(self.pages_progress)
        .bind(self.deleted)
        .execute(conn)
        .await?)
    }
}
impl Updateable for Progress {
    async fn update(
        &mut self,
        conn: &sqlx::SqlitePool,
        new: Self,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                editon_id = ?2,
                timestamp = ?3,
                pages_progress = ?4,
                deleted = ?5
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.edition_id)
        .bind(&new.timestamp)
        .bind(new.pages_progress)
        .bind(new.deleted)
        .execute(conn)
        .await?)
    }
}
