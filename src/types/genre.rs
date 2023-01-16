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
pub struct Genre {
    pub id: Uuid,
    pub name: Text,
    pub deleted: bool,
}

impl UpdateVec for Genre {}

impl PromptType for Genre {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the genre?", None, conn).await?;
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
            .update_by_prompt("Change genre name to:", conn)
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

impl Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let name = self
            .name
            .to_string()
            .style(&config.output_genre.style_content);
        write!(f, "{} ({})", name, self.id)
    }
}
impl DisplayTerminal for Genre {
    async fn fmt(
        &self,
        f: &mut String,
        _conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let name = self
            .name
            .to_string()
            .style(&config.output_genre.style_content);
        write!(f, "{} ({})", name, self.id)?;
        Ok(())
    }
}

impl CreateTable for Genre {
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
        let default_genres = vec![
            (
                "Fantasy",
                uuid::uuid!("26f223a0-879b-4581-9f43-393ff0bf1dbb"),
            ),
            (
                "Science Fiction",
                uuid::uuid!("25a88e29-86e5-4cf6-a035-5c0d932e49e1"),
            ),
            (
                "Dystopian",
                uuid::uuid!("c693ed78-35c1-4488-956a-63d4f8b028d3"),
            ),
            (
                "Action & Adventure",
                uuid::uuid!("d637b371-6c57-4ddf-83c2-22580b81d646"),
            ),
            (
                "Mystery",
                uuid::uuid!("2b8276ab-8a04-478f-b19b-1abab68ed9a7"),
            ),
            (
                "Horror",
                uuid::uuid!("151bc0c4-e2b3-4090-bf91-43ce62dd5d26"),
            ),
            (
                "Thriller",
                uuid::uuid!("7e6f2351-83c1-4a2c-9e4d-793d46301ad2"),
            ),
            (
                "Historical Fiction",
                uuid::uuid!("c12196b9-a845-4c79-a54c-9f9d42ae83db"),
            ),
            (
                "Romance",
                uuid::uuid!("e777568b-e417-4217-9315-4ef28b63807f"),
            ),
            (
                "Graphic Novel",
                uuid::uuid!("50e0cb16-5c2f-4cac-a1fc-6f22f2307859"),
            ),
            (
                "Short Story",
                uuid::uuid!("4186b4d5-80c7-4d7a-a8d4-595cc6be0d66"),
            ),
            (
                "Young Adult",
                uuid::uuid!("ab72338e-0934-4cbf-8f20-66ebcd5e01ce"),
            ),
            (
                "Children",
                uuid::uuid!("197c53e7-b5f3-42d5-8241-1941a2c94402"),
            ),
            (
                "Autobiography",
                uuid::uuid!("3346d4ee-51ac-4e98-ad81-c3703644041e"),
            ),
            (
                "Biography",
                uuid::uuid!("1edd2b50-65e3-4542-8162-ec9dc1332c2b"),
            ),
            (
                "Food & Drink",
                uuid::uuid!("26e9b484-844e-4a71-959a-fa053c340205"),
            ),
            (
                "Art & Photography",
                uuid::uuid!("ecd24fdb-cdcb-4d89-bc6e-3aa06d69ceeb"),
            ),
            (
                "Self-help",
                uuid::uuid!("6893afd1-ba69-4ca7-a71d-d86efe876c03"),
            ),
            (
                "History",
                uuid::uuid!("b4cb537a-f287-4e48-8e6a-e16d88416ab3"),
            ),
            (
                "Travel",
                uuid::uuid!("ca1cf171-1635-493b-a157-b08a92a20654"),
            ),
            (
                "True Crime",
                uuid::uuid!("04f2b840-baee-4af2-af1b-afe110ae1801"),
            ),
            ("Humor", uuid::uuid!("a86ff460-8e20-4176-8db9-29acaabacf99")),
            (
                "Essays",
                uuid::uuid!("1f67e35e-487d-4717-9c12-cca8ea224cdc"),
            ),
            (
                "Religion & Spirituality",
                uuid::uuid!("3f04f6f8-59b9-4afa-beb0-164a45afbbb5"),
            ),
        ];
        for (genre, uuid) in default_genres {
            Self::insert(
                &Self {
                    id: Uuid(uuid),
                    name: Text(genre.to_string()),
                    deleted: false,
                },
                conn,
            )
            .await?;
        }
        Ok(())
    }
}

impl Insertable for Genre {
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
impl Updateable for Genre {
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
