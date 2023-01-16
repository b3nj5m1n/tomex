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
pub struct Binding {
    pub id:      Uuid,
    pub name:    Text,
    pub deleted: bool,
}

impl PromptType for Binding {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the binding?", None, conn).await?;
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
            .update_by_prompt("Change binding name to:", conn)
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

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let name = self
            .name
            .to_string()
            .style(&config.output_binding.style_content);
        if config.output_binding.display_uuid {
            write!(f, "{} ({})", name, self.id)
        } else {
            write!(f, "{}", name)
        }
    }
}
impl DisplayTerminal for Binding {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let name = self
            .name
            .to_string()
            .style(&config.output_binding.style_content);
        if config.output_binding.display_uuid {
            write!(f, "{} ({})", name, self.id)?;
        } else {
            write!(f, "{}", name)?;
        }
        Ok(())
    }
}

impl CreateTable for Binding {
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

        let default_bindings = vec![
            (
                "Perfect binding",
                uuid::uuid!("11a8d073-879f-4970-871c-d1618a776784"),
            ),
            (
                "Case binding",
                uuid::uuid!("6ff10b06-bf48-49c7-8fa9-d3ef247a6858"),
            ),
            (
                "Saddle-stitching",
                uuid::uuid!("519f4975-7a4c-4a17-8927-cafc51f0d827"),
            ),
            (
                "Spiral binding",
                uuid::uuid!("f2b62dd1-26d9-4e5c-a2c6-11c7e7fabb8d"),
            ),
            (
                "Spiral wire binding",
                uuid::uuid!("9fbb9b81-185b-4ab4-a07a-166142337e9e"),
            ),
            (
                "Comb binding",
                uuid::uuid!("feaced93-58df-48c1-be9d-50f1a94e6404"),
            ),
            (
                "Tape binding",
                uuid::uuid!("d1b9b408-f446-4c2f-be1c-5d3f323e41f0"),
            ),
            (
                "Perfect binding with sewn signatures",
                uuid::uuid!("ac1b9213-31c7-452d-b995-8bc01fd367e1"),
            ),
            (
                "Japanese stab binding",
                uuid::uuid!("ea812bd7-df7f-4cf0-b8d8-31d39bfe18d9"),
            ),
            (
                "Hand-stitched binding",
                uuid::uuid!("ec5ba23c-4c1b-4950-b2d5-fad8ef85d855"),
            ),
        ];
        for (binding, uuid) in default_bindings {
            Self::insert(
                &Self {
                    id:      Uuid(uuid),
                    name:    Text(binding.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for Binding {
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
impl Updateable for Binding {
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
