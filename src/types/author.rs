use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use sqlx::{sqlite::{SqliteQueryResult, SqliteRow}, FromRow, Row};

use crate::{
    traits::{CreateByPrompt, Insertable},
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
impl CreateByPrompt for Author {
    fn create_by_prompt() -> Result<Self> {
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
