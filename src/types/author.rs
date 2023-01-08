use std::fmt::Display;
use std::fmt::Write;

use anyhow::Result;

use derives::Id;
use derives::Queryable;
use derives::Removeable;
use derives::{DbTable, CRUD};
use sqlx::{sqlite::SqliteQueryResult, FromRow};

use crate::traits::QueryType;
use crate::traits::Updateable;
use crate::{
    traits::{
        CreateByPrompt, CreateTable, DbTable, DisplayTerminal, Id, Insertable, Queryable,
        Removeable, CRUD,
    },
    types::{timestamp::Timestamp, uuid::Uuid},
};

use super::text::Text;
use super::timestamp::OptionalTimestamp;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, FromRow, DbTable, CRUD, Queryable, Removeable, Id,
)]
pub struct Author {
    pub id: Uuid,
    pub name_first: Option<Text>,
    pub name_last: Option<Text>,
    pub date_born: OptionalTimestamp,
    pub date_died: OptionalTimestamp,
    pub deleted: bool,
}
impl Updateable for Author {
    async fn update(&self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult> {
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

    async fn update_by_query(conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable,
    {
        let old = Self::query_by_prompt(conn).await?;

        let name_first = match &old.name_first {
            Some(name_first) => name_first.update_by_prompt_skippable_deleteable(
                "Do you want to delete the authors first name?",
                "Change authors first name to:",
            )?,
            None => Text::create_by_prompt_skippable("What is the authors first name?", None)?,
        };
        let name_last = match &old.name_last {
            Some(name_last) => name_last.update_by_prompt_skippable_deleteable(
                "Do you want to delete the authors last name?",
                "Change authors last name to:",
            )?,
            None => Text::create_by_prompt_skippable("What is the authors last name?", None)?,
        };
        let date_born = match &old.date_born {
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
        let date_died = match &old.date_died {
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
            deleted: old.deleted,
        };
        Self::update(&old, conn, new).await
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
impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.name_first, &self.name_last) {
            (None, None) => write!(f, "{}", self.id.0),
            (None, Some(name_last)) => {
                write!(f, "{}, (First name unknown) ({})", name_last, self.id.0)
            }
            (Some(name_first), None) => {
                write!(f, "(Last name unknown), {} ({})", name_first, self.id.0)
            }
            (Some(name_first), Some(name_last)) => {
                write!(f, "{}, {} ({})", name_last, name_first, self.id.0)
            }
        }
    }
}
impl DisplayTerminal for Author {
    async fn fmt(&self, f: &mut String, _conn: &sqlx::SqlitePool) -> Result<()> {
        match (&self.name_first, &self.name_last) {
            (None, None) => write!(f, "{}", self.id.0)?,
            (None, Some(name_last)) => {
                write!(f, "{}, (First name unknown) ({})", name_last, self.id.0)?
            }
            (Some(name_first), None) => {
                write!(f, "(Last name unknown), {} ({})", name_first, self.id.0)?
            }
            (Some(name_first), Some(name_last)) => {
                write!(f, "{}, {} ({})", name_last, name_first, self.id.0)?
            }
        }
        Ok(())
    }
}
impl CreateByPrompt for Author {
    async fn create_by_prompt(_conn: &sqlx::SqlitePool) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        /* let name_first = inquire::Text::new("What is the authors first name?")
        .prompt_skippable()?
        .filter(|x| !x.is_empty()); */
        let name_first = Text::create_by_prompt_skippable("What is the authors first name?", None)?;
        let name_last = Text::create_by_prompt_skippable("What is the authors last name?", None)?;
        /* let name_last = inquire::Text::new("What is the authors last name?")
        .prompt_skippable()?
        .filter(|x| !x.is_empty()); */
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
}
