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
    Removeable,
    Serialize,
    Deserialize,
)]
pub struct Mood {
    pub id:      Uuid,
    pub name:    Text,
    pub deleted: bool,
}

impl Queryable for Mood {
    async fn sort_for_display(x: Vec<Self>) -> Vec<Self> {
        let mut x = x.clone();
        x.sort_by(|a, b| a.name.0.partial_cmp(&b.name.0).unwrap());
        return x;
    }
}

impl UpdateVec for Mood {
}

impl PromptType for Mood {
    async fn create_by_prompt(
        prompt: &str,
        initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt(prompt, initial_value.map(|x| &x.name), conn).await?;
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
        let name = Text::update_by_prompt(&self.name, "Change mood name to:", conn).await?;
        Ok(Self {
            name,
            ..self.clone()
        })
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

impl Display for Mood {
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
                .style(&config.output_mood.style_content),
        )?;
        if config.output_mood.display_uuid {
            write!(f, " ({})", self.id)
        } else {
            Ok(())
        }
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
            "{}",
            self.name
                .to_string()
                .style(&config.output_mood.style_content),
        )?;
        if config.output_mood.display_uuid {
            write!(f, " ({})", self.id)?;
        }
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
                    id:      Uuid(uuid),
                    name:    Text(mood.to_string()),
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
        .bind(self.deleted)
        .execute(conn)
        .await?)
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
        .bind(new.deleted)
        .execute(conn)
        .await?)
    }
}
