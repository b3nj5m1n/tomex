use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use derive_builder::Builder;
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};
use std::error::Error;
// use uuid::Uuid;

pub trait CreateByPrompt {
    fn create_by_prompt() -> Result<Self>
    where
        Self: Sized;
}

pub trait Insertable {
    async fn insert(self, conn: sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Sized;
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Timestamp(pub Option<chrono::DateTime<Utc>>);

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Timestamp
where
    i64: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        let value = <i64 as sqlx::Decode<DB>>::decode(value)?;
        if value == 0_i64 {
            return Ok(Self(None));
        }
        let ts = chrono::NaiveDateTime::from_timestamp_millis(value)
            // .filter(|x| *x != chrono::NaiveDateTime::from_timestamp_millis(0).unwrap())
            .map(|x| chrono::DateTime::from_utc(x, Utc));
        Ok(Self(ts))
    }
}
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Uuid {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s: String = self.0.to_string().clone();
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(
            std::borrow::Cow::Owned(s),
        ));

        sqlx::encode::IsNull::No
    }
}
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Timestamp {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Int64(match self.0 {
            None => 0_i64,
            Some(ts) => ts.timestamp_millis(),
        }));

        sqlx::encode::IsNull::No
    }
}
impl sqlx::Type<sqlx::Sqlite> for Timestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&i8 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Uuid(pub uuid::Uuid);

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Uuid
where
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as sqlx::Decode<DB>>::decode(value)?;
        let id = uuid::Uuid::parse_str(value)?;
        Ok(Self(id))
    }
}
impl sqlx::Type<sqlx::Sqlite> for Uuid {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&str as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

#[derive(Default, Builder, Debug, Clone, PartialEq, Eq)]
#[builder(setter(into))]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    #[builder(setter(into, strip_option), default = "None")]
    pub author: Option<Uuid>,
    #[builder(setter(into, strip_option), default = "Timestamp(None)")]
    pub release_date: Timestamp,
    #[builder(default = "vec![]")]
    pub editions: Vec<Edition>,
    #[builder(default = "vec![]")]
    pub reviews: Vec<Review>,
    #[builder(default = "vec![]")]
    pub genres: Vec<Genre>,
    #[builder(default = "false")]
    pub deleted: bool,
}
impl FromRow<'_, SqliteRow> for Book {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            author: row.try_get("author")?,
            release_date: row.try_get("release_date")?,
            editions: vec![], // TODO
            reviews: vec![],
            genres: vec![],
            deleted: row.try_get("deleted")?,
        })
    }
}
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
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Edition {
    pub id: i32,
    pub book_id: i32,
    pub edition_title: Option<String>,
    pub isbn: Option<i64>,
    pub pages: Option<i32>,
    pub language: Option<Language>,
    pub release_date: Timestamp,
    pub publisher: Option<Publisher>,
    pub cover: Option<String>,
    pub moods: Vec<Mood>,
    pub pace: Option<Pace>,
    pub reviews: Vec<EditionReview>,
    pub progress: Vec<Progress>,
    pub deleted: bool,
}
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub id: i32,
    pub book_id: i32,
    pub rating: Option<i32>,
    pub recommend: Option<bool>,
    pub content: Option<String>,
    pub timestamp_created: Timestamp,
    pub timestamp_updated: Timestamp,
    pub pace: Option<Pace>,
    pub deleted: bool,
}
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EditionReview {
    pub id: i32,
    pub edition_id: i32,
    pub rating: Option<i32>,
    pub recommend: Option<bool>,
    pub content: Option<String>,
    pub cover_rating: Option<i32>,
    pub cover_text: Option<String>,
    pub typesetting_rating: Option<i32>,
    pub typesetting_text: Option<String>,
    pub material_rating: Option<i32>,
    pub material_text: Option<String>,
    pub price_rating: Option<i32>,
    pub price_text: Option<String>,
    pub timestamp_created: Timestamp,
    pub timestamp_updated: Timestamp,
    pub deleted: bool,
}
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub id: i32,
    pub edition_id: i32,
    pub timestamp: Timestamp,
    pub pages_progress: i32,
}
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Language {
    pub id: i32,
    pub name: String,
}
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Publisher {
    pub id: i32,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Genre {
    Fantasy,
    ScienceFiction,
    Dystopian,
    ActionAndAdventure,
    Mystery,
    Horror,
    Thriller,
    HistoricalFiction,
    Romance,
    GraphicNovel,
    ShortStory,
    YoungAdult,
    Children,
    Autobiography,
    Biography,
    FoodAndDrink,
    ArtAndPhotography,
    SelfHelp,
    History,
    Travel,
    TrueCrime,
    Humor,
    Essays,
    ReligionAndSpirituality,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pace {
    Slow,
    Medium,
    Fast,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mood {
    Adventurous,
    Challenging,
    Dark,
    Emotional,
    Funny,
    Hopeful,
    Informative,
    Inspiring,
    Lighthearted,
    Mysterious,
    Reflective,
    Relaxing,
    Sad,
    Tense,
}
