use std::fmt::Display;
use std::fmt::Write;

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use derives::Id;
use derives::Queryable;
use derives::Removeable;
use derives::{DbTable, CRUD};
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};

use crate::traits::QueryType;
use crate::traits::Updateable;
use crate::{
    traits::{
        CreateByPrompt, CreateTable, DbTable, DisplayTerminal, Id, Insertable, Queryable,
        Removeable, CRUD,
    },
    types::{timestamp::Timestamp, uuid::Uuid},
};

use super::timestamp::OptionalTimestamp;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, FromRow, DbTable, CRUD, Queryable, Removeable, Id,
)]
pub struct Author {
    pub id: Uuid,
    pub name_first: Option<String>,
    pub name_last: Option<String>,
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
        let old = match Self::query_by_prompt(conn).await? {
            Some(author) => author,
            None => anyhow::bail!("No author selected"),
        };
        let mut name_first = inquire::Text::new("Change authors first name to:")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        if let None = name_first {
            if !inquire::Confirm::new("Do you want to delete this field?")
                .with_default(false)
                .prompt()?
            {
                name_first = old.name_first.clone();
            };
        }
        let mut name_last = inquire::Text::new("Change authors last name to:")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        if let None = name_last {
            if !inquire::Confirm::new("Do you want to delete this field?")
                .with_default(false)
                .prompt()?
            {
                name_last = old.name_last.clone();
            };
        }
        todo!()
        /* let mut date_born = Timestamp(
            inquire::DateSelect::new("When was the author born?")
                .prompt_skippable()?
                .map(|x| {
                    DateTime::from_utc(
                        NaiveDateTime::new(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        Utc,
                    )
                }),
        );
        if let None = date_born.0 {
            if !inquire::Confirm::new("Do you want to delete this field?")
                .with_default(false)
                .prompt()?
            {
                date_born = old.date_born.clone();
            };
        }
        let mut date_died = Timestamp(
            inquire::DateSelect::new("When did the author die?")
                .prompt_skippable()?
                .map(|x| {
                    DateTime::from_utc(
                        NaiveDateTime::new(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        Utc,
                    )
                }),
        );
        if let None = date_died.0 {
            if !inquire::Confirm::new("Do you want to delete this field?")
                .with_default(false)
                .prompt()?
            {
                date_died = old.date_died.clone();
            };
        }
        let delete = inquire::Confirm::new("Do you want to delete this author?")
            .with_default(false)
            .prompt()?;
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
            deleted: delete,
        };
        Self::update(&old, conn, new).await */
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
        let name_first = inquire::Text::new("What is the authors first name?")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        let name_last = inquire::Text::new("What is the authors last name?")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        let date_born = OptionalTimestamp(Timestamp::create_by_prompt_skippable("When was the author born?")?);
        let date_died = OptionalTimestamp(Timestamp::create_by_prompt_skippable("When did the author die?")?);
        if !inquire::Confirm::new("Add author?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };

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
