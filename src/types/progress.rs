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
    // Queryable,
    Removeable,
    Serialize,
    Deserialize,
)]
pub struct Progress {
    pub id:             Uuid,
    pub edition_id:     Uuid,
    pub timestamp:      Timestamp,
    pub pages_progress: PagesProgress,
    pub deleted:        bool,
}

impl Queryable for Progress {
    async fn sort_for_display(x: Vec<Self>) -> Vec<Self> {
        let mut x = x.clone();
        x.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        return x;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PagesProgress {
    #[default]
    Started,
    Finished,
    Pages(u32),
}
impl sqlx::Type<sqlx::Sqlite> for PagesProgress {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <i64 as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for PagesProgress {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Int64(match self {
            PagesProgress::Started => 0_i64,
            PagesProgress::Finished => -1_i64,
            PagesProgress::Pages(n) => i64::from(*n),
        }));

        sqlx::encode::IsNull::No
    }
}
impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for PagesProgress
where
    i64: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i64 as sqlx::Decode<DB>>::decode(value)?;
        match value {
            0 => Ok(Self::Started),
            -1 => Ok(Self::Finished),
            n if n > 0 && u32::try_from(n).is_ok() => {
                Ok(Self::Pages(u32::try_from(n).expect("Unreachable")))
            }
            _ => Err(Box::new(sqlx::Error::Protocol(
                "Invalid pages_progress value".to_string(),
            ))),
        }
    }
}
impl PromptType for PagesProgress {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        const OPTION_START: &'static str = "Start reading book";
        const OPTION_PAGES: &'static str = "Input current page number";
        const OPTION_FINISH: &'static str = "Finish reading book";
        let options: Vec<&str> = vec![OPTION_START, OPTION_PAGES, OPTION_FINISH];

        let ans: Result<&str, inquire::InquireError> = inquire::Select::new("", options).prompt();

        match ans {
            Ok(OPTION_START) => Ok(Self::Started),
            Ok(OPTION_PAGES) => {
                let edition = Edition::query_by_prompt(conn).await?;
                let max_pages = edition.pages;
                let validator = move |input: &str| match input.parse::<u32>() {
                    Ok(n) => {
                        if let Some(max_pages) = max_pages {
                            if n <= max_pages {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    inquire::validator::ErrorMessage::Custom(
                                        "Input has to be lower than number of pages in edition"
                                            .to_string(),
                                    ),
                                ))
                            }
                        } else {
                            Ok(Validation::Valid)
                        }
                    }
                    Err(_) => Ok(Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(
                            "Input isn't a valid number".to_string(),
                        ),
                    )),
                };
                let pages_progress = inquire::Text::new("At which page are you?")
                    .with_validator(validator)
                    .prompt()?
                    .parse::<u32>()
                    .expect("Unreachable");
                Ok(Self::Pages(pages_progress))
            }
            Ok(OPTION_FINISH) => Ok(Self::Finished),
            Ok(_) => unreachable!("Unexpected response"),
            Err(_) => Err(anyhow::anyhow!("Error getting progress")),
        }
    }

    async fn create_by_prompt_skippable(
        _prompt: &str,
        _initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> Result<Option<Self>> {
        unreachable!("Can't skip creation of this type")
    }

    async fn update_by_prompt(
        &self,
        _prompt: &str,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        unreachable!("Can't update this type")
    }

    async fn update_by_prompt_skippable(
        _s: &Option<Self>,
        _prompt: &str,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        unreachable!("Can't update this type")
    }
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
        let pages_progress = PagesProgress::create_by_prompt("", None, conn).await?;
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
        let timestamp = PromptType::update_by_prompt(
            &self.timestamp,
            "For when is this progress update?",
            conn,
        )
        .await?;
        let pages_progress = PagesProgress::create_by_prompt("", None, conn).await?;
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
        match self.pages_progress {
            PagesProgress::Started => write!(
                f,
                "{} {}",
                "Started book".style(&config.output_progress.style_content),
                self.timestamp,
            )?,
            PagesProgress::Finished => write!(
                f,
                "{} {}",
                "Finished book".style(&config.output_progress.style_content),
                self.timestamp,
            )?,
            PagesProgress::Pages(n) => write!(
                f,
                "{}: {} pages",
                self.timestamp,
                n.to_string().style(&config.output_progress.style_content)
            )?,
        };
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
        match self.pages_progress {
            PagesProgress::Started => write!(
                f,
                "{}: {} {}",
                title,
                "Started book".style(&config.output_progress.style_content),
                self.timestamp,
            )?,
            PagesProgress::Finished => write!(
                f,
                "{}: {} {}",
                title,
                "Finished book".style(&config.output_progress.style_content),
                self.timestamp,
            )?,
            PagesProgress::Pages(n) => write!(
                f,
                "{}: {} pages ({})",
                title,
                n.to_string().style(&config.output_progress.style_content),
                self.timestamp,
            )?,
        };
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
            	pages_progress	BIGINT	NOT NULL,
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
        .bind(self.pages_progress.clone())
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
