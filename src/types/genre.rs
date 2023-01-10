use anyhow::Result;
use crossterm::style::Stylize;
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    traits::*,
    types::{text::Text, uuid::Uuid},
};
use derives::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Id, Names, CRUD, Queryable, Removeable)]
pub struct Genre {
    pub id: Uuid,
    pub name: Text,
    pub deleted: bool,
}

impl Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{}", self.name);
        let name = name.with(crossterm::style::Color::Rgb {
            r: 240,
            g: 198,
            b: 198,
        });
        write!(f, "{} ({})", name, self.id)
    }
}
impl DisplayTerminal for Genre {
    async fn fmt(&self, f: &mut String, _conn: &sqlx::SqlitePool) -> Result<()> {
        let name = format!("{}", self.name);
        let name = name.with(crossterm::style::Color::Rgb {
            r: 240,
            g: 198,
            b: 198,
        });
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
        // TODO: Possibly hardcode the uuid's as well
        let default_genres = vec![
            "Fantasy",
            "Science Fiction",
            "Dystopian",
            "Action & Adventure",
            "Mystery",
            "Horror",
            "Thriller",
            "Historical Fiction",
            "Romance",
            "Graphic Novel",
            "Short Story",
            "Young Adult",
            "Children",
            "Autobiography",
            "Biography",
            "Food & Drink",
            "Art & Photography",
            "Self-help",
            "History",
            "Travel",
            "True Crime",
            "Humor",
            "Essays",
            "Religion & Spirituality",
        ];
        for genre in default_genres {
            Self::insert(
                &Self {
                    id: Uuid(uuid::Uuid::new_v4()),
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
        .bind(&self.deleted)
        .execute(conn)
        .await?)
    }
    async fn create_by_prompt(_conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let name = Text::create_by_prompt("What is the name of the genre?", None)?;
        Ok(Self {
            id,
            name,
            deleted: false,
        })
    }
}
impl Updateable for Genre {
    async fn update(
        &self,
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
        &self,
        conn: &sqlx::SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Queryable,
    {
        let name = self
            .name
            .update_by_prompt_skippable("Change genre name to:")?;
        let new = Self {
            id: Uuid(uuid::Uuid::nil()),
            name,
            deleted: self.deleted,
        };
        Self::update(&self, conn, new).await
    }
}
