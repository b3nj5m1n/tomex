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
pub struct Publisher {
    pub id:      Uuid,
    pub name:    Text,
    pub deleted: bool,
}

impl UpdateVec for Publisher {
}

impl PromptType for Publisher {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the publisher?", None, conn).await?;
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
        let name =
            PromptType::update_by_prompt(&self.name, "Change publisher name to:", conn).await?;
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

impl Display for Publisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        write!(
            f,
            "{}",
            self.name
                .to_string()
                .style(&config.output_publisher.style_content),
        )?;
        if config.output_publisher.display_uuid {
            write!(f, " ({})", self.id)
        } else {
            Ok(())
        }
    }
}
impl DisplayTerminal for Publisher {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        write!(
            f,
            "{}",
            self.name
                .to_string()
                .style(&config.output_publisher.style_content),
        )?;
        if config.output_publisher.display_uuid {
            write!(f, " ({})", self.id)?;
        }
        Ok(())
    }
}

impl CreateTable for Publisher {
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

        let default_publishers = vec![
            (
                "Penguin Random House",
                uuid::uuid!("2334916b-e46c-4acf-ba6c-c2145f8e4be8"),
            ),
            (
                "Hachette Livre",
                uuid::uuid!("103c44fe-337a-46c9-8cfe-769d31af7557"),
            ),
            (
                "HarperCollins",
                uuid::uuid!("9f7ba146-adde-46a8-bacc-e2b0cdd76279"),
            ),
            (
                "Pan Macmillan",
                uuid::uuid!("f11b4ba2-e7f6-40a3-b48c-16d4113a1754"),
            ),
            (
                "Pearson Education",
                uuid::uuid!("5fee5de1-34e7-4ce6-b77f-b372024c517d"),
            ),
            (
                "Oxford University Press",
                uuid::uuid!("7cb9511d-c1c9-416f-8fc6-b5146eb22d3e"),
            ),
            (
                "Bloomsbury",
                uuid::uuid!("5f478846-4b3a-4dc2-9613-81545a313b1b"),
            ),
            (
                "Simon & Schuster",
                uuid::uuid!("0a2ae995-4657-4814-86ca-df96e1b6ec0b"),
            ),
            (
                "John Wiley & Sons",
                uuid::uuid!("f524b405-45d0-4709-a7bd-73714239e05b"),
            ),
        ];
        for (publisher, uuid) in default_publishers {
            Self::insert(
                &Self {
                    id:      Uuid(uuid),
                    name:    Text(publisher.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for Publisher {
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
impl Updateable for Publisher {
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
