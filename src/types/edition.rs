use anyhow::Result;
use crossterm::style::Stylize;
use inquire::{validator::Validation, MultiSelect};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    traits::*,
    types::{
        author::Author,
        book::Book,
        book_author::BookAuthor,
        book_genre::BookGenre,
        edition_review::EditionReview,
        genre::Genre,
        language::Language,
        mood::Mood,
        pace::Pace,
        progress::Progress,
        publisher::Publisher,
        text::Text,
        timestamp::{OptionalTimestamp, Timestamp},
        uuid::Uuid,
    },
};
use derives::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Names, Queryable, Id, Removeable, CRUD)]
pub struct Edition {
    pub id: Uuid,
    pub book_id: Uuid,
    pub edition_title: Option<Text>,
    pub isbn: Option<Text>,
    pub pages: Option<u32>,
    pub languages: Option<Vec<Language>>,
    pub release_date: OptionalTimestamp,
    pub publishers: Option<Vec<Publisher>>,
    pub cover: Option<String>,
    pub reviews: Option<Vec<EditionReview>>,
    pub progress: Option<Vec<Progress>>,
    pub deleted: bool,
}

impl Edition {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_languages(conn).await?;
        self.hydrate_publishers(conn).await?;
        Ok(())
    }
    pub async fn get_languages(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Language>>> {
        // let result = BookAuthor::get_all_for_a(conn, self).await?;
        // Ok(if result.len() > 0 { Some(result) } else { None })
        todo!()
    }
    pub async fn hydrate_languages(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        // self.authors = self.get_authors(conn).await?;
        // Ok(())
        todo!()
    }
    pub async fn get_publishers(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Publisher>>> {
        // let result = BookAuthor::get_all_for_a(conn, self).await?;
        // Ok(if result.len() > 0 { Some(result) } else { None })
        todo!()
    }
    pub async fn hydrate_publishers(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        // self.authors = self.get_authors(conn).await?;
        // Ok(())
        todo!()
    }
}

impl Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.isbn {
            None => write!(f, "{}", self.id),
            Some(isbn) => {
                write!(f, "{}", isbn)
            }
        }
    }
}
impl DisplayTerminal for Edition {
    async fn fmt(&self, f: &mut String, conn: &sqlx::SqlitePool) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        if let Some(edition_title) = s.edition_title {
            let title = format!("{}", edition_title);
            let title = title
                .with(crossterm::style::Color::Rgb {
                    r: 238,
                    g: 153,
                    b: 16,
                })
                .bold();
            write!(f, "{}", title)?;
            write!(f, " ")?; // TODO firgure out how to use Formatter to avoid this
        }
        if let Some(isbn) = s.isbn {
            let str = isbn.to_string().italic();
            write!(f, "[{}]", str)?;
            write!(f, " ")?;
        }
        write!(f, "({})", s.id)?;
        Ok(())
    }
}

impl CreateTable for Edition {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE {} (
                id TEXT PRIMARY KEY NOT NULL,
            	book_id	TEXT NOT NULL,
                edition_title   TEXT,
            	isbn	TEXT,
                pages   INT,
            	release_date	INTEGER,
            	cover	TEXT,
            	deleted BOOL DEFAULT FALSE,
            	FOREIGN KEY (book_id) REFERENCES {} (id)
            );"#,
            Self::TABLE_NAME,
            Book::TABLE_NAME,
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl Insertable for Edition {
    async fn insert(
        &self,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized,
    {
        let result = sqlx::query(
            r#"
            INSERT INTO editions ( id, book_id, edition_title, isbn, pages, release_date, cover, deleted )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8 );
            "#,
        )
        .bind(&self.id)
        .bind(&self.book_id)
        .bind(&self.edition_title)
        .bind(&self.isbn)
        .bind(&self.pages)
        .bind(&self.release_date)
        .bind(&self.cover)
        .bind(&self.deleted)
        .execute(conn)
        .await?;

        // TODO
        // EditionLanguage::update(conn, &self, &None, self.languages).await?;
        // EditionPublisher::update(conn, &self, &None, self.languages).await?;

        Ok(result)
    }
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let book = Book::query_by_prompt(conn).await?;
        let book_id = book.id;
        let edition_title =
            Text::create_by_prompt_skippable("What is the title of this edition?", None)?;
        let isbn = Text::create_by_prompt_skippable("What is the isbn of this edition?", None)?;
        let validator = |input: &str| match input.parse::<u32>() {
            Ok(n) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid(
                inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
            )),
        };
        let pages = inquire::Text::new("How many pages does this edition have?")
            .with_validator(validator)
            .prompt_skippable()?
            .map(|x| x.parse::<u32>().expect("Unreachable"));
        Ok(Self {
            id,
            book_id,
            edition_title,
            isbn,
            pages,
            languages: None,
            release_date: OptionalTimestamp(None),
            publishers: None,
            cover: None,
            reviews: None,
            progress: None,
            deleted: false,
        })
    }
}
impl Updateable for Edition {
    async fn update(
        &mut self,
        conn: &sqlx::SqlitePool,
        new: Self,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        self.hydrate(conn).await?;
        // TODO
        // EditionLanguage::update(conn, &self, &None, self.languages).await?;
        // EditionPublisher::update(conn, &self, &None, self.languages).await?;
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                book_id = ?2,
                edition_title = ?4,
                isbn = ?5,
                pages = ?6,
                release_date = ?7,
                cover = ?8,
                deleted = ?10,
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.book_id)
        .bind(&new.edition_title)
        .bind(&new.isbn)
        .bind(&new.pages)
        .bind(&new.release_date)
        .bind(&new.cover)
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
        let edition_title = match &self.edition_title {
            Some(s) => s.update_by_prompt_skippable_deleteable(
                "Delete edition_tittle date?",
                "What is the edition title?",
            )?,
            None => Text::create_by_prompt_skippable("What is the edition title?", None)?,
        };
        let isbn = match &self.isbn {
            Some(s) => s.update_by_prompt_skippable_deleteable(
                "Delete isbn?",
                "What is the isbn of this edition?",
            )?,
            None => Text::create_by_prompt_skippable("What is the isbn of this edition?", None)?,
        };
        // Languages
        let all_languages = Language::get_all(conn).await?;
        let current_languages = self.get_languages(conn).await?;
        let indicies_selected = if let Some(current_languages) = current_languages {
            all_languages
                .iter()
                .enumerate()
                .filter(|(_, language)| current_languages.contains(language))
                .map(|(i, _)| i)
                .collect::<Vec<usize>>()
        } else {
            vec![]
        };
        let mut languages = MultiSelect::new("Select languages for this edition:", all_languages)
            .with_default(&indicies_selected)
            .prompt_skippable()?;
        if let Some(languages_) = languages {
            languages = if languages_.len() > 0 {
                Some(languages_)
            } else {
                None
            };
        }
        // Publishers
        let all_publishers = Publisher::get_all(conn).await?;
        let current_publishers = self.get_publishers(conn).await?;
        let indicies_selected = if let Some(current_publishers) = current_publishers {
            all_publishers
                .iter()
                .enumerate()
                .filter(|(_, publisher)| current_publishers.contains(publisher))
                .map(|(i, _)| i)
                .collect::<Vec<usize>>()
        } else {
            vec![]
        };
        let mut publishers =
            MultiSelect::new("Select publishers for this edition:", all_publishers)
                .with_default(&indicies_selected)
                .prompt_skippable()?;
        if let Some(publishers_) = publishers {
            publishers = if publishers_.len() > 0 {
                Some(publishers_)
            } else {
                None
            };
        }
        let new = Self {
            edition_title,
            isbn,
            languages,
            publishers,
            deleted: self.deleted,
            ..self.clone()
        };
        Self::update(self, conn, new).await
    }
}

impl FromRow<'_, SqliteRow> for Edition {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            book_id: row.try_get("book_id")?,
            edition_title: row.try_get("edition_title")?,
            isbn: row.try_get("isbn")?,
            pages: row.try_get("pages")?,
            release_date: row.try_get("release_date")?,
            cover: row.try_get("cover")?,
            deleted: row.try_get("deleted")?,
            ..Self::default()
        })
    }
}
