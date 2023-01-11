use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, FromRow};
use std::fmt::{Display, Write};

use crate::{
    traits::*,
    types::{
        text::Text,
        timestamp::{OptionalTimestamp, Timestamp},
        uuid::Uuid,
    },
};
use derives::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Names, CRUD, Queryable, Removeable, Id)]
pub struct Author {
    pub id: Uuid,
    pub name_first: Option<Text>,
    pub name_last: Option<Text>,
    pub date_born: OptionalTimestamp,
    pub date_died: OptionalTimestamp,
    pub deleted: bool,
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.name_first, &self.name_last) {
            (None, None) => write!(f, "{}", self.id),
            (None, Some(name_last)) => {
                write!(f, "{}, (First name unknown) ({})", name_last, self.id)
            }
            (Some(name_first), None) => {
                write!(f, "(Last name unknown), {} ({})", name_first, self.id)
            }
            (Some(name_first), Some(name_last)) => {
                write!(f, "{}, {} ({})", name_last, name_first, self.id)
            }
        }
    }
}
impl DisplayTerminal for Author {
    async fn fmt(&self, f: &mut String, _conn: &sqlx::SqlitePool) -> Result<()> {
        match (&self.name_first, &self.name_last) {
            (None, None) => write!(f, "{}", self.id)?,
            (None, Some(name_last)) => {
                write!(f, "{}, (First name unknown) ({})", name_last, self.id)?
            }
            (Some(name_first), None) => {
                write!(f, "(Last name unknown), {} ({})", name_first, self.id)?
            }
            (Some(name_first), Some(name_last)) => {
                write!(f, "{}, {} ({})", name_last, name_first, self.id)?
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
    async fn create_by_prompt(_conn: &sqlx::SqlitePool) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name_first = Text::create_by_prompt_skippable("What is the authors first name?", None)?;
        let name_last = Text::create_by_prompt_skippable("What is the authors last name?", None)?;
        let date_born = OptionalTimestamp(Timestamp::create_by_prompt_skippable(
            "When was the author born?",
            None,
        )?);
        let date_died = OptionalTimestamp(Timestamp::create_by_prompt_skippable(
            "When did the author die?",
            None,
        )?);

        Ok(Self {
            id,
            name_first,
            name_last,
            date_born,
            date_died,
            deleted: false,
        })
    }
}
impl Updateable for Author {
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
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
        let name_first = match &self.name_first {
            Some(name_first) => name_first.update_by_prompt_skippable_deleteable(
                "Do you want to delete the authors first name?",
                "Change authors first name to:",
            )?,
            None => Text::create_by_prompt_skippable("What is the authors first name?", None)?,
        };
        let name_last = match &self.name_last {
            Some(name_last) => name_last.update_by_prompt_skippable_deleteable(
                "Do you want to delete the authors last name?",
                "Change authors last name to:",
            )?,
            None => Text::create_by_prompt_skippable("What is the authors last name?", None)?,
        };
        let date_born = match &self.date_born {
            OptionalTimestamp(Some(ts)) => {
                OptionalTimestamp(ts.update_by_prompt_skippable_deleteable(
                    "Delete date of birth?",
                    "When was the author born?",
                )?)
            }
            OptionalTimestamp(None) => OptionalTimestamp(Timestamp::create_by_prompt_skippable(
                "When was the author born?",
                None,
            )?),
        };
        let date_died = match &self.date_died {
            OptionalTimestamp(Some(ts)) => {
                OptionalTimestamp(ts.update_by_prompt_skippable_deleteable(
                    "Delete date of death?",
                    "When did the author die?",
                )?)
            }
            OptionalTimestamp(None) => OptionalTimestamp(Timestamp::create_by_prompt_skippable(
                "When did the author die?",
                None,
            )?),
        };

        if !inquire::Confirm::new("Update author?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };

        let new = Self {
            id: Uuid(uuid::Uuid::nil()),
            name_first,
            name_last,
            date_born,
            date_died,
            deleted: self.deleted,
        };
        Self::update(self, conn, new).await
    }
}
