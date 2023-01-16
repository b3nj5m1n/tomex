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
pub struct Language {
    pub id: Uuid,
    pub name: Text,
    pub deleted: bool,
}

impl UpdateVec for Language {}

impl PromptType for Language {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the language?", None, conn).await?;
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
            .update_by_prompt("Change language name to:", conn)
            .await?;
        let new = Self {
            id: Uuid(uuid::Uuid::nil()),
            name,
            deleted: self.deleted,
        };
        Ok(new)
    }

    async fn create_by_prompt_skippable(
        prompt: &str,
        initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Option<Self>> {
        unreachable!("Can't skip creation of this type")
    }

    async fn update_by_prompt_skippable(
        s: &Option<Self>,
        prompt: &str,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>>
    where
        Self: Display,
    {
        unreachable!("Can't skip updating this type")
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        write!(
            f,
            "{} ({})",
            self.name
                .to_string()
                .style(&config.output_language.style_content),
            self.id
        )
    }
}
impl DisplayTerminal for Language {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        write!(
            f,
            "{} ({})",
            self.name
                .to_string()
                .style(&config.output_language.style_content),
            self.id
        )?;
        Ok(())
    }
}

impl CreateTable for Language {
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

        let default_languages = vec![(
            "English",
            uuid::uuid!("a95f5f6e-8560-4b02-9443-14f7502d28fe"),
        )];
        for (language, uuid) in default_languages {
            Self::insert(
                &Self {
                    id: Uuid(uuid),
                    name: Text(language.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for Language {
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
impl Updateable for Language {
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
