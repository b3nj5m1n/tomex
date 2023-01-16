use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{text::Text, uuid::Uuid},
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
pub struct EditionFormat {
    pub id:      Uuid,
    pub name:    Text,
    pub deleted: bool,
}

impl PromptType for EditionFormat {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the format?", None, conn).await?;
        Ok(Self {
            id,
            name,
            deleted: false,
        })
    }

    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let name = self
            .name
            .update_by_prompt("Change format name to:", conn)
            .await?;
        let new = Self {
            id: Uuid(uuid::Uuid::nil()),
            name,
            deleted: self.deleted,
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

impl Display for EditionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let name = self
            .name
            .to_string()
            .style(&config.output_format.style_content);
        if config.output_format.display_uuid {
            write!(f, "{} ({})", name, self.id)
        } else {
            write!(f, "{}", name)
        }
    }
}
impl DisplayTerminal for EditionFormat {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let name = self
            .name
            .to_string()
            .style(&config.output_format.style_content);
        if config.output_format.display_uuid {
            write!(f, "{} ({})", name, self.id)?;
        } else {
            write!(f, "{}", name)?;
        }
        Ok(())
    }
}

impl CreateTable for EditionFormat {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                deleted BOOL DEFAULT FALSE
            );
            "#,
            Self::TABLE_NAME
        ))
        .execute(conn)
        .await?;
        let default_formats = vec![
            (
                "Paperback",
                uuid::uuid!("93b3f802-21df-486c-b15b-3da96f533c01"),
            ),
            (
                "Hardcover",
                uuid::uuid!("249f84dc-b704-4bb1-8f48-b78ad973c543"),
            ),
            ("Ebook", uuid::uuid!("5e6b39a9-6f6f-4cf0-a92c-55088c36202f")),
            (
                "Audiobook",
                uuid::uuid!("ed96d107-3ba7-4328-9e15-c9b583863a17"),
            ),
        ];
        for (format, uuid) in default_formats {
            Self::insert(
                &Self {
                    id:      Uuid(uuid),
                    name:    Text(format.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for EditionFormat {
    async fn insert(
        &self,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized,
    {
        Ok(sqlx::query(&format!(
            r#"
                    INSERT INTO {} ( id, name, deleted )
                    VALUES ( ?1, ?2, ?3 )
                    "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&self.name)
        .bind(self.deleted)
        .execute(conn)
        .await?)
    }
}
impl Updateable for EditionFormat {
    async fn update(
        &mut self,
        conn: &sqlx::SqlitePool,
        new: Self,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                name = ?2,
                deleted = ?3
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.name)
        .bind(new.deleted)
        .execute(conn)
        .await?)
    }
}
