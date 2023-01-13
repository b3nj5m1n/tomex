use anyhow::Result;
use crossterm::style::Stylize;
use inquire::{validator::Validation, MultiSelect};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    default_colors::COLOR_DIMMED,
    traits::*,
    types::{
        book::Book, edition_language::EditionLanguage, edition_publisher::EditionPublisher,
        edition_review::EditionReview, language::Language, progress::Progress,
        publisher::Publisher, text::Text, timestamp::OptionalTimestamp, uuid::Uuid,
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
    pub book_title: Text,
}

impl Edition {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_languages(conn).await?;
        self.hydrate_publishers(conn).await?;
        Ok(())
    }
    pub async fn get_languages(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Language>>> {
        let result = EditionLanguage::get_all_for_a(conn, self).await?;
        Ok(if result.len() > 0 { Some(result) } else { None })
    }
    pub async fn hydrate_languages(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.languages = self.get_languages(conn).await?;
        Ok(())
    }
    pub async fn get_publishers(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Publisher>>> {
        let result = EditionPublisher::get_all_for_a(conn, self).await?;
        Ok(if result.len() > 0 { Some(result) } else { None })
    }
    pub async fn hydrate_publishers(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.publishers = self.get_publishers(conn).await?;
        Ok(())
    }
}

impl Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        match (&self.isbn, &self.edition_title) {
            (None, None) => write!(
                f,
                "{} ({})",
                self.book_title.style(&config.output_edition.style_content),
                self.id
            ),
            (None, Some(title)) => write!(
                f,
                "{} ({})",
                title.style(&config.output_edition.style_content),
                self.id
            ),
            (Some(isbn), None) => {
                write!(
                    f,
                    "{} ({})",
                    self.book_title.style(&config.output_edition.style_content),
                    isbn.to_string().with(COLOR_DIMMED)
                )
            }
            (Some(isbn), Some(title)) => {
                write!(
                    f,
                    "{} ({})",
                    title.style(&config.output_edition.style_content),
                    isbn.to_string().with(COLOR_DIMMED)
                )
            }
        }
    }
}
impl DisplayTerminal for Edition {
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let book = Book::get_by_id(conn, &s.book_id).await?;
        // Edition/Book title
        let title = match s.edition_title {
            Some(t) => format!("{}", t),
            None => format!("{}", book.title),
        }
        .style(&config.output_edition.style_content);
        write!(f, "{} ", title)?;
        // Author
        if let Some(authors) = book.get_authors(conn).await? {
            let str = "written by".italic();
            write!(f, "[{}: ", str)?;
            write!(
                f,
                "{}",
                authors
                    .into_iter()
                    .map(|author| author.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            write!(f, "]",)?;
            write!(f, " ")?;
        }
        // Page count
        if let Some(pages) = s.pages {
            let str = pages.to_string().with(crossterm::style::Color::Rgb {
                r: 139,
                g: 213,
                b: 202,
            });
            write!(f, "[{} pages] ", str)?;
        }
        // Language
        if let Some(languages) = s.languages {
            let str = "written in".italic();
            write!(f, "[{}: ", str)?;
            write!(
                f,
                "{}",
                languages
                    .into_iter()
                    .map(|language| language.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            write!(f, "]",)?;
            write!(f, " ")?;
        }
        // Release date
        if let Some(release_date) = s.release_date.0 {
            let str = "released".italic();
            write!(f, "[{} {}]", str, release_date)?;
            write!(f, " ")?; // TODO see above
        }
        // Publishers
        if let Some(publishers) = s.publishers {
            let str = "published by".italic();
            write!(f, "[{}: ", str)?;
            write!(
                f,
                "{}",
                publishers
                    .into_iter()
                    .map(|publisher| publisher.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            write!(f, "]",)?;
            write!(f, " ")?;
        }
        // ISBN or ID
        if let Some(isbn) = s.isbn {
            let str = isbn.to_string().italic();
            write!(f, "({})", str)?;
        } else {
            write!(f, "({})", s.id)?;
        }
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
                book_title TEXT,
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
            INSERT INTO editions ( id, book_id, edition_title, isbn, pages, release_date, cover, deleted, book_title )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9 );
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
        .bind(&self.book_title)
        .execute(conn)
        .await?;

        EditionLanguage::update(conn, &self, &None, &self.languages).await?;
        EditionPublisher::update(conn, &self, &None, &self.publishers).await?;

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
            Ok(_) => Ok(Validation::Valid),
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
            book_title: book.title,
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
        EditionLanguage::update(conn, &self, &self.languages, &new.languages).await?;
        EditionPublisher::update(conn, &self, &self.publishers, &new.publishers).await?;
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                book_id = ?2,
                edition_title = ?3,
                isbn = ?4,
                pages = ?5,
                release_date = ?6,
                cover = ?7,
                deleted = ?8,
                book_title = ?9
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
        .bind(&new.book_title)
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
        let book = Book::get_by_id(conn, &self.book_id).await?;
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
        let indicies_selected = if let Some(current_languages) = &current_languages {
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
        } else {
            languages = current_languages;
        }
        // Publishers
        let all_publishers = Publisher::get_all(conn).await?;
        let current_publishers = self.get_publishers(conn).await?;
        let indicies_selected = if let Some(current_publishers) = &current_publishers {
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
        } else {
            publishers = current_publishers;
        }
        let new = Self {
            edition_title,
            isbn,
            languages,
            publishers,
            deleted: self.deleted,
            book_title: book.title,
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
            book_title: row.try_get("book_title")?,
            ..Self::default()
        })
    }
}
