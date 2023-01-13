use anyhow::Result;
use sqlx::FromRow;
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{text::Text, uuid::Uuid},
};
use derives::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Id, Names, CRUD, Queryable, Removeable)]
pub struct Mood {
    pub id: Uuid,
    pub name: Text,
    pub deleted: bool,
}

impl Display for Mood {
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
                .style(&config.output_mood.style_content),
            self.id
        )
    }
}
impl DisplayTerminal for Mood {
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
                .style(&config.output_mood.style_content),
            self.id
        )?;
        Ok(())
    }
}

impl CreateTable for Mood {
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

        let default_moods = vec![
            (
                "Adventurous",
                uuid::uuid!("e7291183-ba90-48a3-b102-b21e732fd2c0"),
            ),
            (
                "Challenging",
                uuid::uuid!("95c5140e-f62d-4982-8858-c5336bd9df70"),
            ),
            ("Dark", uuid::uuid!("37f3beee-ba35-4957-ae9c-fb4f19827e4c")),
            (
                "Emotional",
                uuid::uuid!("b4b06dd1-be29-4914-8a6e-a11d7e12849c"),
            ),
            ("Funny", uuid::uuid!("6665ef19-ebdc-4bdb-bcb4-845f0d04f896")),
            (
                "Hopeful",
                uuid::uuid!("9c2d0812-6c25-4294-a917-6e7faa826ae8"),
            ),
            (
                "Informative",
                uuid::uuid!("4ba07184-92f3-41d0-b733-d3e403a7f533"),
            ),
            (
                "Inspiring",
                uuid::uuid!("07532c14-9bf5-442b-bd63-6038a40aaad0"),
            ),
            (
                "Lighthearted",
                uuid::uuid!("5447082a-bef4-4b27-8906-fc7b3124ecd6"),
            ),
            (
                "Mysterious",
                uuid::uuid!("0c86213f-64f4-47ab-ac29-e3fc4c0666b2"),
            ),
            (
                "Reflective",
                uuid::uuid!("12ff33e3-3b65-4821-afd6-5c2bdb1d9a60"),
            ),
            (
                "Relaxing",
                uuid::uuid!("3516a18c-a3f4-408a-9388-1790efddb538"),
            ),
            ("Sad", uuid::uuid!("bb2c5921-eee5-4a62-aa83-cb7834e558c2")),
            ("Tense", uuid::uuid!("7f584f2d-35f1-4fec-aeba-e62c7212398f")),
        ];
        for (mood, uuid) in default_moods {
            Self::insert(
                &Self {
                    id: Uuid(uuid),
                    name: Text(mood.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for Mood {
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
        .bind(&self.deleted)
        .execute(conn)
        .await?)
    }
    async fn create_by_prompt(_conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the mood?", None)?;
        Ok(Self {
            id,
            name,
            deleted: false,
        })
    }
}
impl Updateable for Mood {
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
        .bind(&new.deleted)
        .execute(conn)
        .await?)
    }

    async fn update_by_prompt(
        &mut self,
        conn: &sqlx::SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Queryable,
    {
        let name = self
            .name
            .update_by_prompt_skippable("Change mood name to:")?;
        let new = Self {
            id: Uuid(uuid::Uuid::nil()),
            name,
            deleted: self.deleted,
        };
        Self::update(self, conn, new).await
    }
}
