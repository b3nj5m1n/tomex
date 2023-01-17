use anyhow::Result;
use crossterm::style::Stylize;
use inquire::validator::Validation;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    default_colors::COLOR_DIMMED,
    traits::*,
    types::{
        book::Book, edition_language::EditionLanguage, edition_publisher::EditionPublisher,
        edition_review::EditionReview, isbn::Isbn, language::Language, progress::Progress,
        publisher::Publisher, text::Text, timestamp::OptionalTimestamp, uuid::Uuid,
    },
};
use derives::*;

use super::{binding::Binding, format::EditionFormat};

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Names,
    Queryable,
    Id,
    Removeable,
    CRUD,
    Serialize,
    Deserialize,
)]
pub struct Edition {
    pub id:                  Uuid,
    pub book_id:             Uuid,
    pub edition_title:       Option<Text>,
    pub edition_description: Option<Text>,
    pub isbn:                Option<Text>,
    pub pages:               Option<u32>,
    pub languages:           Option<Vec<Language>>,
    pub release_date:        OptionalTimestamp,
    pub format_id:           Option<Uuid>,
    pub format:              Option<EditionFormat>,
    pub height:              Option<u32>,
    pub width:               Option<u32>,
    pub thickness:           Option<u32>,
    pub weight:              Option<u32>,
    pub binding_id:          Option<Uuid>,
    pub binding:             Option<Binding>,
    pub publishers:          Option<Vec<Publisher>>,
    pub cover:               Option<String>,
    pub reviews:             Option<Vec<EditionReview>>,
    pub progress:            Option<Vec<Progress>>,
    pub deleted:             bool,
    pub book_title:          Text,
}

impl Edition {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_languages(conn).await?;
        self.hydrate_publishers(conn).await?;
        self.hydrate_format(conn).await?;
        self.hydrate_binding(conn).await?;
        Ok(())
    }

    pub async fn get_languages(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Language>>> {
        let result = EditionLanguage::get_all_for_a(conn, self).await?;
        Ok(if !result.is_empty() {
            Some(result)
        } else {
            None
        })
    }

    pub async fn get_format(&self, conn: &sqlx::SqlitePool) -> Result<Option<EditionFormat>> {
        Ok(match &self.format_id {
            Some(id) => Some(EditionFormat::get_by_id(conn, id).await?),
            None => None,
        })
    }

    pub async fn get_binding(&self, conn: &sqlx::SqlitePool) -> Result<Option<Binding>> {
        Ok(match &self.binding_id {
            Some(id) => Some(Binding::get_by_id(conn, id).await?),
            None => None,
        })
    }

    pub async fn hydrate_languages(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.languages = self.get_languages(conn).await?;
        Ok(())
    }

    pub async fn get_publishers(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Publisher>>> {
        let result = EditionPublisher::get_all_for_a(conn, self).await?;
        Ok(if !result.is_empty() {
            Some(result)
        } else {
            None
        })
    }

    pub async fn hydrate_publishers(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.publishers = self.get_publishers(conn).await?;
        Ok(())
    }

    pub async fn hydrate_format(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.format = self.get_format(conn).await?;
        Ok(())
    }

    pub async fn hydrate_binding(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.binding = self.get_binding(conn).await?;
        Ok(())
    }
}

impl PromptType for Edition {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let book = Book::query_or_create_by_prompt(conn).await?;
        let book_id = book.id;
        let edition_title =
            Text::create_by_prompt_skippable("What is the title of this edition?", None, conn)
                .await?;
        let edition_description = Text::create_by_prompt_skippable(
            "Describe this edition (for example 1st edition, special edition, etc):",
            None,
            conn,
        )
        .await?;
        let isbn = PromptType::create_by_prompt_skippable(
            "What is the isbn of this edition?",
            None::<&Isbn>,
            conn,
        )
        .await?;
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
        let format = match EditionFormat::query_by_prompt_skippable(conn).await? {
            Some(format) => Some(format),
            None => None,
        };
        let format_id = format.clone().map(|x| x.id);
        Ok(Self {
            id,
            book_id,
            edition_title,
            edition_description,
            isbn: isbn.map(|x| x.to_text()),
            pages,
            languages: None,
            release_date: OptionalTimestamp(None),
            publishers: None,
            cover: None,
            reviews: None,
            progress: None,
            deleted: false,
            book_title: book.title,
            format_id,
            format,
            height: None,     // TODO
            width: None,      // TODO
            thickness: None,  // TODO
            weight: None,     // TODO
            binding_id: None, // TODO
            binding: None,
        })
    }

    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let book = Book::get_by_id(conn, &s.book_id).await?;
        let edition_title = PromptType::update_by_prompt_skippable(
            &s.edition_title,
            "What is the edition title?",
            conn,
        )
        .await?;
        let edition_description = PromptType::update_by_prompt_skippable(
            &s.edition_description,
            "Describe this edition (for example 1st edition, special edition, etc)",
            conn,
        )
        .await?;
        let isbn = PromptType::update_by_prompt_skippable(
            &s.isbn,
            "What is the isbn of this edition?",
            conn,
        )
        .await?;
        // Languages
        let languages =
            Language::update_vec(&s.languages, conn, "Select languages for this edition:").await?;
        // Publishers
        let publishers =
            Publisher::update_vec(&s.publishers, conn, "Select publishers for this edition:")
                .await?;
        // Format
        let format = match EditionFormat::query_by_prompt_skippable(conn).await? {
            Some(format) => Some(format),
            None => s.format.clone(),
        };
        let format_id = format.clone().map(|x| x.id);
        // Binding
        let binding = match Binding::query_by_prompt_skippable(conn).await? {
            Some(binding) => Some(binding),
            None => s.binding.clone(),
        };
        let binding_id = binding.clone().map(|x| x.id);
        let new = Self {
            edition_title,
            edition_description,
            isbn,
            languages,
            publishers,
            deleted: self.deleted,
            book_title: book.title,
            format_id,
            format,
            binding,
            binding_id,
            ..self.clone()
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

impl Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        // TODO hide uuid if requested
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
            Some(t) => format!("{t}"),
            None => format!("{}", book.title),
        }
        .style(&config.output_edition.style_content);
        write!(f, "{title} ")?;
        // Author
        if let Some(authors) = book.get_authors(conn).await? {
            write!(
                f,
                "{} ",
                config
                    .output_author
                    .format_vec(authors, conn, config)
                    .await?
            )?;
        }
        // Page count
        if let Some(pages) = s.pages {
            write!(
                f,
                "{} ",
                config
                    .output_page_count
                    .format_str(pages, conn, config)
                    .await?
            )?;
        }
        // Format
        if let Some(format) = s.format {
            write!(
                f,
                "{} ",
                config.output_format.format(format, conn, config).await?
            )?;
        }
        // Binding
        if let Some(binding) = s.binding {
            write!(
                f,
                "{} ",
                config.output_binding.format(binding, conn, config).await?
            )?;
        }
        // Language
        if let Some(languages) = s.languages {
            write!(
                f,
                "{} ",
                config
                    .output_language
                    .format_vec(languages, conn, config)
                    .await?
            )?;
        }
        // Release date
        if let Some(release_date) = s.release_date.0 {
            write!(
                f,
                "{} ",
                config
                    .output_release_date
                    .format_str(release_date, conn, config)
                    .await?
            )?;
        }
        // Publishers
        if let Some(publishers) = s.publishers {
            write!(
                f,
                "{} ",
                config
                    .output_publisher
                    .format_vec(publishers, conn, config)
                    .await?
            )?;
        }
        // ISBN or ID
        if let Some(isbn) = s.isbn {
            let str = isbn.to_string().italic();
            write!(f, "({str})")?;
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
                edition_description   TEXT,
            	isbn	TEXT,
                pages   INT,
            	release_date	INTEGER,
                format_id TEXT,
                height  INT,
                width    INT,
                thickness INT,
                weight INT,
                binding_id TEXT,
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
            INSERT INTO editions ( id, book_id, edition_title, edition_description, isbn, pages, release_date, format_id, height, width, thickness, weight, binding_id, cover, deleted, book_title )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16 );
            "#,
        )
        .bind(&self.id)
        .bind(&self.book_id)
        .bind(&self.edition_title)
        .bind(&self.edition_description)
        .bind(&self.isbn)
        .bind(self.pages)
        .bind(&self.release_date)
        .bind(&self.format_id)
        .bind(&self.height)
        .bind(&self.width)
        .bind(&self.thickness)
        .bind(&self.weight)
        .bind(&self.binding_id)
        .bind(&self.cover)
        .bind(self.deleted)
        .bind(&self.book_title)
        .execute(conn)
        .await?;

        EditionLanguage::update(conn, self, &None, &self.languages).await?;
        EditionPublisher::update(conn, self, &None, &self.publishers).await?;

        Ok(result)
    }
}
impl Updateable for Edition {
    async fn update(
        &mut self,
        conn: &sqlx::SqlitePool,
        new: Self,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        self.hydrate(conn).await?;
        EditionLanguage::update(conn, self, &self.languages, &new.languages).await?;
        EditionPublisher::update(conn, self, &self.publishers, &new.publishers).await?;
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                book_id = ?2,
                edition_title = ?3,
                edition_description = ?4,
                isbn = ?5,
                pages = ?6,
                release_date = ?7,
                format_id = ?8,
                height = ?9,
                width = ?10,
                thickness = ?11,
                weight = ?12,
                binding_id = ?13,
                cover = ?14,
                deleted = ?15,
                book_title = ?16
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.book_id)
        .bind(&new.edition_title)
        .bind(&new.edition_description)
        .bind(&new.isbn)
        .bind(new.pages)
        .bind(&new.release_date)
        .bind(&new.format_id)
        .bind(&new.height)
        .bind(&new.width)
        .bind(&new.thickness)
        .bind(&new.weight)
        .bind(&new.binding_id)
        .bind(&new.cover)
        .bind(new.deleted)
        .bind(&new.book_title)
        .execute(conn)
        .await?)
    }
}

impl FromRow<'_, SqliteRow> for Edition {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id:                  row.try_get("id")?,
            book_id:             row.try_get("book_id")?,
            edition_title:       row.try_get("edition_title")?,
            edition_description: row.try_get("edition_description")?,
            isbn:                row.try_get("isbn")?,
            pages:               row.try_get("pages")?,
            release_date:        row.try_get("release_date")?,
            cover:               row.try_get("cover")?,
            deleted:             row.try_get("deleted")?,
            book_title:          row.try_get("book_title")?,
            format_id:           row.try_get("format_id")?,
            height:              row.try_get("height")?,
            width:               row.try_get("width")?,
            thickness:           row.try_get("thickness")?,
            weight:              row.try_get("weight")?,
            binding_id:          row.try_get("binding_id")?,
            languages:           Self::default().languages,
            format:              Self::default().format,
            binding:             Self::default().binding,
            publishers:          Self::default().publishers,
            reviews:             Self::default().reviews,
            progress:            Self::default().progress,
        })
    }
}
