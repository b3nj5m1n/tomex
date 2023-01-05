use std::fmt::Display;

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};

use crate::{
    traits::{CreateByPrompt, Insertable, Queryable},
    types::{timestamp::Timestamp, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Author {
    pub id: Uuid,
    pub name_first: Option<String>,
    pub name_last: Option<String>,
    pub date_born: Timestamp,
    pub date_died: Timestamp,
    pub deleted: bool,
}
impl Queryable for Author {
    async fn query(conn: sqlx::SqlitePool) -> Result<Option<Self>> {
        let authors: Vec<Author> = sqlx::query_as::<_, Author>("SELECT * FROM authors;")
            .fetch_all(&conn)
            .await?;
        let ans: Option<Author> =
            inquire::Select::new("Select author:", authors).prompt_skippable()?;
        Ok(ans)
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
impl CreateByPrompt for Author {
    async fn create_by_prompt(_conn: sqlx::SqlitePool) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let name_first = inquire::Text::new("What is the authors first name?")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        let name_last = inquire::Text::new("What is the authors last name?")
            .prompt_skippable()?
            .filter(|x| !x.is_empty());
        let date_born = Timestamp(
            inquire::DateSelect::new("What was the author born?")
                .prompt_skippable()?
                .map(|x| {
                    DateTime::from_utc(
                        NaiveDateTime::new(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        Utc,
                    )
                }),
        );
        let date_died = Timestamp(
            inquire::DateSelect::new("What did the author die?")
                .prompt_skippable()?
                .map(|x| {
                    DateTime::from_utc(
                        NaiveDateTime::new(x, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        Utc,
                    )
                }),
        );
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
    async fn insert(self, conn: sqlx::SqlitePool) -> Result<SqliteQueryResult> {
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
        .execute(&conn)
        .await?)
    }
}