use anyhow::Result;
use inquire::MultiSelect;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use std::fmt::{Display, Write};

use crate::{
    config::{self, Styleable},
    traits::*,
    types::{
        author::Author,
        edition::Edition,
        genre::Genre,
        review::Review,
        text::Text,
        timestamp::{OptionalTimestamp, Timestamp},
        uuid::Uuid,
    },
};
use derives::*;

use super::{book_author::BookAuthor, book_genre::BookGenre};

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
pub struct Book {
    pub id: Uuid,
    pub title: Text,
    pub authors: Option<Vec<Author>>,
    pub release_date: OptionalTimestamp,
    pub editions: Option<Vec<Edition>>,
    pub reviews: Option<Vec<Review>>,
    pub genres: Option<Vec<Genre>>,
    pub deleted: bool,
}

impl Book {
    pub async fn hydrate(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.hydrate_authors(conn).await?;
        self.hydrate_genres(conn).await?;
        Ok(())
    }
    pub async fn get_authors(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Author>>> {
        let result = BookAuthor::get_all_for_a(conn, self).await?;
        Ok(if result.len() > 0 { Some(result) } else { None })
    }
    pub async fn get_genres(&self, conn: &sqlx::SqlitePool) -> Result<Option<Vec<Genre>>> {
        let result = BookGenre::get_all_for_a(conn, self).await?;
        Ok(if result.len() > 0 { Some(result) } else { None })
    }
    pub async fn hydrate_authors(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.authors = self.get_authors(conn).await?;
        Ok(())
    }
    pub async fn hydrate_genres(&mut self, conn: &sqlx::SqlitePool) -> Result<()> {
        self.genres = self.get_genres(conn).await?;
        Ok(())
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
            None => write!(f, "{} ({})", title, self.id),
            Some(release_date) => {
                write!(f, "{}, released {} ({})", title, release_date, self.id)
            }
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
        write!(f, "{} ", title)?;
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
        write!(f, "({})", s.id)?;
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
            INSERT INTO books ( id, title, release_date, deleted )
            VALUES ( ?1, ?2, ?3, ?4 );
            "#,
        )
        .bind(&self.id)
        .bind(&self.title)
        .bind(&self.release_date)
        .bind(&self.deleted)
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
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let id = Uuid(uuid::Uuid::new_v4());
        let title = Text::create_by_prompt("What is the title of the book?", None)?;
        let author = Author::query_or_create_by_prompt_skippable(conn).await?;
        // let release_date = OptionalTimestamp(Timestamp::create_by_prompt_skippable(
        //     "When was the book released?",
        //     None,
        // )?);
        let all_genres = Genre::get_all(conn).await?;
        let mut genres =
            MultiSelect::new("Select genres for this book:", all_genres).prompt_skippable()?;
        if let Some(genres_) = genres {
            genres = if genres_.len() > 0 {
                Some(genres_)
            } else {
                None
            };
        }
        Ok(Self {
            id,
            title,
            authors: author.map(|x| vec![x]),
            release_date: OptionalTimestamp(None),
            editions: None, // TODO
            reviews: None,  // TODO
            genres,
            deleted: false,
        })
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
                release_date = ?4,
                deleted = ?5
            WHERE
                id = ?1;
            "#,
            Self::TABLE_NAME
        ))
        .bind(&self.id)
        .bind(&new.title)
        .bind(&new.release_date)
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
        let title = self.title.update_by_prompt_skippable("Change title to:")?;
        let release_date = match &self.release_date {
            OptionalTimestamp(Some(ts)) => {
                OptionalTimestamp(ts.update_by_prompt_skippable_deleteable(
                    "Delete release date?",
                    "When was the book released?",
                )?)
            }
            OptionalTimestamp(None) => OptionalTimestamp(Timestamp::create_by_prompt_skippable(
                "When was the book released?",
                None,
            )?),
        };
        let all_genres = Genre::get_all(conn).await?;
        let current_genres = self.get_genres(conn).await?;
        let indicies_selected = if let Some(current_genres) = current_genres {
            all_genres
                .iter()
                .enumerate()
                .filter(|(_, genre)| current_genres.contains(genre))
                .map(|(i, _)| i)
                .collect::<Vec<usize>>()
        } else {
            vec![]
        };
        let mut genres = MultiSelect::new("Select genres for this book:", all_genres)
            .with_default(&indicies_selected)
            .prompt_skippable()?;
        if let Some(genres_) = genres {
            genres = if genres_.len() > 0 {
                Some(genres_)
            } else {
                None
            };
        }
        let new = Self {
            id: self.id.clone(),
            title,
            authors: self.authors.clone(), // TODO
            release_date,
            editions: self.editions.clone(),
            reviews: self.reviews.clone(),
            genres,
            deleted: self.deleted,
        };
        Self::update(self, conn, new).await
    }
}

impl FromRow<'_, SqliteRow> for Book {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            authors: None,
            release_date: row.try_get("release_date")?,
            editions: None, // TODO
            reviews: None,
            genres: None,
            deleted: row.try_get("deleted")?,
        })
    }
}
