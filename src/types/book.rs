use anyhow::Result;
use inquire::MultiSelect;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{
        author::Author, edition::Edition, genre::Genre, review::Review, text::Text,
        timestamp::OptionalTimestamp, uuid::Uuid,
    },
};
use derives::*;

use super::{book_author::BookAuthor, book_genre::BookGenre, rating::Rating, series::Series};

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Names,
    Id,
    Removeable,
    CRUD,
    Serialize,
    Deserialize,
)]
pub struct Book {
    pub id:           Uuid,
    pub title:        Text,
    pub authors:      Option<Vec<Author>>,
    pub release_date: OptionalTimestamp,
    pub summary:      Option<Text>,
    pub series_id:    Option<Uuid>,
    pub series_index: Option<u32>,
    pub series:       Option<Series>,
    pub editions:     Option<Vec<Edition>>,
    pub reviews:      Option<Vec<Review>>,
    pub genres:       Option<Vec<Genre>>,
    pub deleted:      bool,
}

impl Queryable for Book {
    async fn sort_for_display(x: Vec<Self>) -> Vec<Self> {
        let mut x = x.clone();
        x.sort_by(|a, b| a.title.0.partial_cmp(&b.title.0).unwrap());
        return x;
    }
}

impl Book {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_authors(conn).await?;
        self.hydrate_genres(conn).await?;
        self.hydrate_series(conn).await?;
        Ok(())
    }

    pub async fn get_authors(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Author>>> {
        let result = BookAuthor::get_all_for_a(conn, self).await?;
        Ok(if !result.is_empty() {
            Some(result)
        } else {
            None
        })
    }

    pub async fn get_genres(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Genre>>> {
        let result = BookGenre::get_all_for_a(conn, self).await?;
        Ok(if !result.is_empty() {
            Some(result)
        } else {
            None
        })
    }

    pub async fn get_series(&self, conn: &sqlx::SqlitePool) -> Result<Option<Series>> {
        if let Some(id) = &self.series_id {
            Ok(Some(Series::get_by_id(conn, id).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn hydrate_authors(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.authors = self.get_authors(conn).await?;
        Ok(())
    }

    pub async fn hydrate_genres(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.genres = self.get_genres(conn).await?;
        Ok(())
    }

    pub async fn hydrate_series(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.series = self.get_series(conn).await?;
        Ok(())
    }

    pub async fn get_by_title(conn: &sqlx::SqlitePool, title: String) -> Result<Option<Self>> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE title = ?1 COLLATE NOCASE AND deleted = 0;",
            Self::TABLE_NAME
        ))
        .bind(title)
        .fetch_optional(conn)
        .await?)
    }
}

impl PromptType for Book {
    async fn create_by_prompt(
        _prompt: &str,
        _initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> Result<Self> {
        let id = Uuid(uuid::Uuid::new_v4());
        let title = Text::create_by_prompt("What is the title of the book?", None, conn).await?;
        let author = Author::query_or_create_by_prompt_skippable(conn).await?;
        let all_genres = Genre::get_all(conn).await?;
        let mut genres =
            MultiSelect::new("Select genres for this book:", all_genres).prompt_skippable()?;
        if let Some(genres_) = genres {
            genres = if !genres_.is_empty() {
                Some(genres_)
            } else {
                None
            };
        }
        let series = Series::query_or_create_by_prompt_skippable(conn).await?;
        let series_id = series.clone().map(|x| x.id);
        let series_index = match series_id {
            Some(_) => {
                PromptType::create_by_prompt_skippable(
                    "What is the books position in the series?",
                    None::<&Rating>,
                    conn,
                )
                .await?
            }
            None => None,
        };

        Ok(Self {
            id,
            title,
            authors: author.map(|x| vec![x]),
            release_date: OptionalTimestamp(None),
            summary: None,  // TODO
            editions: None, // TODO
            reviews: None,  // TODO
            genres,
            deleted: false,
            series_id,
            series_index,
            series,
        })
    }

    async fn update_by_prompt(&self, _prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        let title = self
            .title
            .update_by_prompt("Change title to:", conn)
            .await?;
        let release_date = PromptType::update_by_prompt_skippable(
            &self.release_date.0,
            "When was the book released?",
            conn,
        )
        .await?;
        let genres = Genre::update_vec(&self.genres, conn, "Select genres for this book:").await?;
        let series = match Series::query_or_create_by_prompt_skippable(conn).await? {
            Some(series) => Some(series),
            None => self.series.clone(),
        };
        let series_id = series.clone().map(|x| x.id);
        let series_index = match series_id {
            Some(_) => {
                PromptType::update_by_prompt_skippable(
                    &self.series_index,
                    "What is the books position in the series?",
                    conn,
                )
                .await?
            }
            None => None,
        };
        let new = Self {
            id: self.id.clone(),
            title,
            authors: self.authors.clone(), // TODO
            release_date: OptionalTimestamp(release_date),
            summary: self.summary.clone(), // TODO
            editions: self.editions.clone(),
            reviews: self.reviews.clone(),
            genres,
            deleted: self.deleted,
            series_id,
            series_index,
            series,
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

impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let title = self
            .title
            .to_string()
            .style(&config.output_book.style_content);
        match &self.release_date.0 {
            None => write!(f, "{}", title)?,
            Some(release_date) => write!(f, "{}, released {}", title, release_date)?,
        }
        if config.output_book.display_uuid {
            write!(f, " ({})", self.id)
        } else {
            Ok(())
        }
    }
}
impl DisplayTerminal for Book {
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        let mut s = self.clone();
        s.hydrate(conn).await?;
        let title = self
            .title
            .to_string()
            .style(&config.output_book.style_content);
        write!(f, "{title} ")?;
        if let Some(authors) = s.authors {
            write!(
                f,
                "{} ",
                config
                    .output_author
                    .format_vec(authors, conn, config)
                    .await?
            )?;
        }
        if let Some(series) = s.series {
            write!(f, "[")?;
            if let Some(idx) = s.series_index {
                write!(
                    f,
                    "{} in the ",
                    format!("#{}", idx).style(&config.output_series.style_content)
                )?;
            } else {
                write!(f, "Part of the ")?;
            }
            write!(f, "{} series", series.to_string())?;
            write!(f, "] ")?;
        }
        if let Some(release_date) = &s.release_date.0 {
            write!(
                f,
                "{} ",
                config
                    .output_release_date
                    .format_str(release_date, conn, config)
                    .await?
            )?;
        }
        if let Some(genres) = s.genres {
            write!(
                f,
                "{} ",
                config.output_genre.format_vec(genres, conn, config).await?
            )?;
        }
        if config.output_book.display_uuid {
            write!(f, "({})", s.id)?;
        }
        Ok(())
    }
}

impl CreateTable for Book {
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                release_date INTEGER,
                summary TEXT,
                series_id TEXT,
                series_index INTEGER,
                deleted BOOL DEFAULT FALSE
            );"#,
            Self::TABLE_NAME
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl Insertable for Book {
    async fn insert(
        &self,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
    where
        Self: Sized,
    {
        let result = sqlx::query(
            r#"
            INSERT INTO books ( id, title, release_date, series_id, series_index, deleted )
            VALUES ( ?1, ?2, ?3, ?4, ?5, ?6 );
            "#,
        )
        .bind(&self.id)
        .bind(&self.title)
        .bind(&self.release_date)
        .bind(&self.series_id)
        .bind(&self.series_index)
        .bind(self.deleted)
        .execute(conn)
        .await?;

        if let Some(authors) = &self.authors {
            for author in authors {
                BookAuthor::insert(conn, self, author).await?;
            }
        }
        if let Some(genres) = &self.genres {
            for genre in genres {
                BookGenre::insert(conn, self, genre).await?;
            }
        }

        Ok(result)
    }
}
impl Updateable for Book {
    async fn update(
        &mut self,
        conn: &sqlx::SqlitePool,
        new: Self,
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        self.hydrate(conn).await?;
        BookAuthor::update(conn, self, &self.authors, &new.authors).await?;
        BookGenre::update(conn, self, &self.genres, &new.genres).await?;
        Ok(sqlx::query(&format!(
            r#"
            UPDATE {}
            SET 
                title = ?2,
                release_date = ?3,
                series_id = ?4,
                series_index = ?5,
                deleted = ?6
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.title)
        .bind(&new.release_date)
        .bind(&new.series_id)
        .bind(&new.series_index)
        .bind(new.deleted)
        .execute(conn)
        .await?)
    }
}

impl FromRow<'_, SqliteRow> for Book {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id:           row.try_get("id")?,
            title:        row.try_get("title")?,
            authors:      None,
            release_date: row.try_get("release_date")?,
            summary:      row.try_get("summary")?,
            editions:     None, // TODO
            reviews:      None,
            genres:       None,
            deleted:      row.try_get("deleted")?,
            series_id:    row.try_get("series_id")?,
            series_index: row.try_get("series_index")?,
            series:       None,
        })
    }
}
