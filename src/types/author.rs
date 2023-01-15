use anyhow::Result;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};
use std::fmt::{Display, Write};

use crate::{
    config,
    config::Styleable,
    traits::*,
    types::{
        text::Text,
        timestamp::{OptionalTimestamp, Timestamp},
        uuid::Uuid,
    },
};
use derives::*;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Names, CRUD, Queryable, Id, Serialize, Deserialize,
)]
pub struct Author {
    pub id: Uuid,
    pub name_first: Option<Text>,
    pub name_last: Option<Text>,
    pub date_born: OptionalTimestamp,
    pub date_died: OptionalTimestamp,
    pub deleted: bool,
    pub special: bool,
}

const UUID_UNKOWN: Uuid = Uuid(uuid::uuid!("00000000-0000-0000-0000-000000000000"));

impl PromptType for Author {
    async fn create_by_prompt(
        prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name_first =
            Text::create_by_prompt_skippable("What is the authors first name?", None, conn).await?;
        let name_last =
            Text::create_by_prompt_skippable("What is the authors last name?", None, conn).await?;
        Ok(Self {
            id,
            name_first,
            name_last,
            date_born: OptionalTimestamp(None),
            date_died: OptionalTimestamp(None),
            deleted: false,
            special: false,
        })
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        if self.special {
            match self.id {
                UUID_UNKOWN => write!(f, "{}", "UNKOWN AUTHOR".bold()),
                _ => Err(std::fmt::Error),
            }
        } else {
            match (&self.name_first, &self.name_last) {
                (None, None) => write!(f, "{}", self.id),
                (None, Some(name_last)) => {
                    write!(
                        f,
                        "{}, (First name unknown) ({})",
                        name_last.style(&config.output_author.style_content),
                        self.id
                    )
                }
                (Some(name_first), None) => {
                    write!(
                        f,
                        "(Last name unknown), {} ({})",
                        name_first.style(&config.output_author.style_content),
                        self.id
                    )
                }
                (Some(name_first), Some(name_last)) => {
                    write!(
                        f,
                        "{}, {} ({})",
                        name_last.style(&config.output_author.style_content),
                        name_first.style(&config.output_author.style_content),
                        self.id
                    )
                }
            }
        }
    }
}
impl DisplayTerminal for Author {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        if self.special {
            match self.id {
                UUID_UNKOWN => write!(f, "{}", "UNKOWN AUTHOR".bold())?,
                _ => anyhow::bail!("Unkown special author"),
            }
        } else {
            match (&self.name_first, &self.name_last) {
                (None, None) => write!(f, "{}", self.id)?,
                (None, Some(name_last)) => write!(
                    f,
                    "{}, (First name unknown) ({})",
                    name_last.style(&config.output_author.style_content),
                    self.id
                )?,
                (Some(name_first), None) => write!(
                    f,
                    "(Last name unknown), {} ({})",
                    name_first.style(&config.output_author.style_content),
                    self.id
                )?,
                (Some(name_first), Some(name_last)) => write!(
                    f,
                    "{}, {} ({})",
                    name_last.style(&config.output_author.style_content),
                    name_first.style(&config.output_author.style_content),
                    self.id
                )?,
            }
        }
        Ok(())
    }
}

impl CreateTable for Author {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
                name_first TEXT,
                name_last TEXT,
                date_born INTEGER,
                date_died INTEGER,
                deleted BOOL DEFAULT FALSE
            );"#,
            Self::TABLE_NAME
        ))
        .execute(conn)
        .await?;
        Self::insert(
            &Self {
                id: UUID_UNKOWN,
                name_first: None,
                name_last: None,
                date_born: OptionalTimestamp(None),
                date_died: OptionalTimestamp(None),
                deleted: false,
                special: true,
            },
            conn,
        )
        .await?;
        Ok(())
    }
}

impl Insertable for Author {
    async fn insert(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult> {
        Ok(sqlx::query(
            r#"
                    INSERT INTO authors ( id, name_first, name_last, date_born, date_died, deleted )
                    VALUES ( ?1, ?2, ?3, ?4, ?5, ?6 )
                    "#,
        )
        .bind(&self.id)
        .bind(&self.name_first)
        .bind(&self.name_last)
        .bind(&self.date_born)
        .bind(&self.date_died)
        .bind(&self.deleted)
        .execute(conn)
        .await?)
    }
}
impl Updateable for Author {
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
        if self.special {
            anyhow::bail!("Can't update special author");
        }
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                name_first = ?2,
                name_last = ?3,
                date_born = ?4,
                date_died = ?5,
                deleted = ?6
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.name_first)
        .bind(&new.name_last)
        .bind(&new.date_born)
        .bind(&new.date_died)
        .bind(&new.deleted)
        .execute(conn)
        .await?)
    }

    async fn update_by_prompt(&mut self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable,
    {
        if self.special {
            anyhow::bail!("Can't update special author");
        }
        let name_first = match &self.name_first {
            Some(name_first) => {
                name_first
                    .update_by_prompt_skippable_deleteable(
                        "Do you want to delete the authors first name?",
                        "Change authors first name to:",
                        conn,
                    )
                    .await?
            }
            None => {
                Text::create_by_prompt_skippable("What is the authors first name?", None, conn)
                    .await?
            }
        };
        let name_last = match &self.name_last {
            Some(name_last) => {
                name_last
                    .update_by_prompt_skippable_deleteable(
                        "Do you want to delete the authors last name?",
                        "Change authors last name to:",
                        conn,
                    )
                    .await?
            }
            None => {
                Text::create_by_prompt_skippable("What is the authors last name?", None, conn)
                    .await?
            }
        };
        let date_born = match &self.date_born {
            OptionalTimestamp(Some(ts)) => OptionalTimestamp(
                ts.update_by_prompt_skippable_deleteable(
                    "Delete date of birth?",
                    "When was the author born?",
                    conn,
                )
                .await?,
            ),
            OptionalTimestamp(None) => OptionalTimestamp(
                Timestamp::create_by_prompt_skippable("When was the author born?", None, conn)
                    .await?,
            ),
        };
        let date_died = match &self.date_died {
            OptionalTimestamp(Some(ts)) => OptionalTimestamp(
                ts.update_by_prompt_skippable_deleteable(
                    "Delete date of death?",
                    "When did the author die?",
                    conn,
                )
                .await?,
            ),
            OptionalTimestamp(None) => OptionalTimestamp(
                Timestamp::create_by_prompt_skippable("When did the author die?", None, conn)
                    .await?,
            ),
        };

        if !inquire::Confirm::new("Update author?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };

        let new = Self {
            name_first,
            name_last,
            date_born,
            date_died,
            ..self.clone()
        };
        Self::update(self, conn, new).await
    }
}

impl Removeable for Author {
    async fn remove_by_prompt(conn: &sqlx::SqlitePool) -> Result<()>
    where
        Self: Queryable,
    {
        let x = Self::query_by_prompt_skippable(conn).await?;
        if let Some(x) = x.as_ref() {
            if x.special {
                anyhow::bail!("Can't remove special author manually");
            }
        }
        match x {
            Some(x) => {
                if !inquire::Confirm::new(&format!("Are you sure you want to remove {}?", x))
                    .with_default(false)
                    .prompt()?
                {
                    anyhow::bail!("Aborted");
                };
                Self::remove(&x, conn).await?;
                println!("Deleted");
            }
            None => println!("Nothing selected, doing nothing"),
        }
        Ok(())
    }
}

impl FromRow<'_, SqliteRow> for Author {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let s = Self {
            id: row.try_get("id")?,
            deleted: row.try_get("deleted")?,
            name_first: row.try_get("name_first")?,
            name_last: row.try_get("name_last")?,
            date_born: row.try_get("date_born")?,
            date_died: row.try_get("date_died")?,
            special: false,
        };
        if s.id == UUID_UNKOWN {
            return Ok(Self {
                id: s.id,
                special: true,
                ..Self::default()
            });
        }
        Ok(s)
    }
}
