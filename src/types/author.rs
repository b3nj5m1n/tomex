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
    types::{text::Text, timestamp::OptionalTimestamp, uuid::Uuid},
};
use derives::*;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, Names, CRUD, Queryable, Id, Serialize, Deserialize,
)]
pub struct Author {
    pub id:        Uuid,
    pub name:      Option<Text>,
    pub date_born: OptionalTimestamp,
    pub date_died: OptionalTimestamp,
    pub deleted:   bool,
    pub special:   bool,
}

const UUID_UNKOWN: Uuid = Uuid(uuid::uuid!("00000000-0000-0000-0000-000000000000"));

impl Author {
    pub async fn get_by_name(conn: &sqlx::SqlitePool, name: String) -> Result<Option<Self>> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE name = ?1 COLLATE NOCASE AND deleted = 0;",
            Self::TABLE_NAME
        ))
        .bind(name)
        .fetch_optional(conn)
        .await?)
    }
}

impl PromptType for Author {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name =
            Text::create_by_prompt_skippable("What is the authors name?", None, conn).await?;
        Ok(Self {
            id,
            name,
            date_born: OptionalTimestamp(None),
            date_died: OptionalTimestamp(None),
            deleted: false,
            special: false,
        })
    }

    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        if self.special {
            anyhow::bail!("Can't update special author");
        }
        let name =
            PromptType::update_by_prompt_skippable(&self.name, "What is the authors name?", conn)
                .await?;
        let date_born = PromptType::update_by_prompt_skippable(
            &self.date_born.0,
            "When was the author born?",
            conn,
        )
        .await?;
        let date_died = PromptType::update_by_prompt_skippable(
            &self.date_died.0,
            "When did the author die?",
            conn,
        )
        .await?;

        if !inquire::Confirm::new("Update author?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };

        let new = Self {
            name,
            date_born: OptionalTimestamp(date_born),
            date_died: OptionalTimestamp(date_died),
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
            match &self.name {
                None => (),
                Some(name) => write!(f, "{}", name.style(&config.output_author.style_content),)?,
            }
            if config.output_author.display_uuid {
                write!(f, " ({})", self.id)
            } else {
                Ok(())
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
            match &self.name {
                None => (),
                Some(name) => write!(f, "{}", name.style(&config.output_author.style_content),)?,
            }
        }
        if config.output_author.display_uuid {
            write!(f, " ({})", self.id)?;
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
                name TEXT,
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
                id:        UUID_UNKOWN,
                name:      None,
                date_born: OptionalTimestamp(None),
                date_died: OptionalTimestamp(None),
                deleted:   false,
                special:   true,
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
            INSERT INTO authors ( id, name, date_born, date_died, deleted )
            VALUES ( ?1, ?2, ?3, ?4, ?5 )
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .bind(&self.date_born)
        .bind(&self.date_died)
        .bind(self.deleted)
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
                name = ?2,
                date_born = ?3,
                date_died = ?4,
                deleted = ?5
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.name)
        .bind(&new.date_born)
        .bind(&new.date_died)
        .bind(new.deleted)
        .execute(conn)
        .await?)
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
                if !inquire::Confirm::new(&format!("Are you sure you want to remove {x}?"))
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
            id:        row.try_get("id")?,
            deleted:   row.try_get("deleted")?,
            name:      row.try_get("name")?,
            date_born: row.try_get("date_born")?,
            date_died: row.try_get("date_died")?,
            special:   false,
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
