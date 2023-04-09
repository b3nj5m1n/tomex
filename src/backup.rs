use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    config::Styleable,
    traits::*,
    types::{
        author::Author, binding::Binding, book::Book, book_author::BookAuthor,
        book_genre::BookGenre, edition::Edition, edition_language::EditionLanguage,
        edition_publisher::EditionPublisher, edition_review::EditionReview, format::EditionFormat,
        genre::Genre, language::Language, mood::Mood, pace::Pace, progress::Progress,
        publisher::Publisher, review::Review, review_mood::ReviewMood, series::Series, uuid::Uuid,
    },
};

/// Contains the entire state of the database
#[derive(Serialize, Deserialize, Default, PartialEq)]
pub struct State {
    moods:              Vec<Mood>,
    paces:              Vec<Pace>,
    genres:             Vec<Genre>,
    languages:          Vec<Language>,
    publishers:         Vec<Publisher>,
    books:              Vec<Book>,
    editions:           Vec<Edition>,
    authors:            Vec<Author>,
    reviews:            Vec<Review>,
    edition_reviews:    Vec<EditionReview>,
    progress:           Vec<Progress>,
    series:             Vec<Series>,
    bindings:           Vec<Binding>,
    edition_formats:    Vec<EditionFormat>,
    book_authors:       Vec<BookAuthor>,
    book_genres:        Vec<BookGenre>,
    edition_languages:  Vec<EditionLanguage>,
    edition_publishers: Vec<EditionPublisher>,
    review_moods:       Vec<ReviewMood>,
}

impl State {
    /// Generate [State] struct from database
    pub async fn load(conn: &sqlx::SqlitePool) -> Result<Self> {
        Ok(Self {
            moods:              Mood::get_all(conn).await?,
            paces:              Pace::get_all(conn).await?,
            genres:             Genre::get_all(conn).await?,
            languages:          Language::get_all(conn).await?,
            publishers:         Publisher::get_all(conn).await?,
            books:              Book::get_all(conn).await?,
            editions:           Edition::get_all(conn).await?,
            authors:            Author::get_all(conn).await?,
            reviews:            Review::get_all(conn).await?,
            edition_reviews:    EditionReview::get_all(conn).await?,
            progress:           Progress::get_all(conn).await?,
            series:             Series::get_all(conn).await?,
            bindings:           Binding::get_all(conn).await?,
            edition_formats:    EditionFormat::get_all(conn).await?,
            book_authors:       BookAuthor::get_all(conn).await?,
            book_genres:        BookGenre::get_all(conn).await?,
            edition_languages:  EditionLanguage::get_all(conn).await?,
            edition_publishers: EditionPublisher::get_all(conn).await?,
            review_moods:       ReviewMood::get_all(conn).await?,
        })
    }

    /// Sort all fields on [State]
    pub fn sort(&mut self) {
        self.moods.sort_by_key(|x| x.id.clone());
        self.paces.sort_by_key(|x| x.id.clone());
        self.genres.sort_by_key(|x| x.id.clone());
        self.languages.sort_by_key(|x| x.id.clone());
        self.publishers.sort_by_key(|x| x.id.clone());
        self.books.sort_by_key(|x| x.id.clone());
        self.editions.sort_by_key(|x| x.id.clone());
        self.authors.sort_by_key(|x| x.id.clone());
        self.reviews.sort_by_key(|x| x.id.clone());
        self.edition_reviews.sort_by_key(|x| x.id.clone());
        self.progress.sort_by_key(|x| x.id.clone());
        self.series.sort_by_key(|x| x.id.clone());
        self.bindings.sort_by_key(|x| x.id.clone());
        self.edition_formats.sort_by_key(|x| x.id.clone());
        self.book_authors.sort_by_key(|x| x.book_id.clone());
        self.book_genres.sort_by_key(|x| x.book_id.clone());
        self.edition_languages.sort_by_key(|x| x.edition_id.clone());
        self.edition_publishers
            .sort_by_key(|x| x.edition_id.clone());
        self.review_moods.sort_by_key(|x| x.review_id.clone());
    }

    /// Return true if the database is in default state
    /// Currently mostly just guesses so use with caution
    pub async fn is_fresh(conn: &sqlx::SqlitePool) -> Result<bool> {
        let current = State::load(conn).await?;
        // This solution doesn't work since we're not inserting the default
        // data into the tables using the Default trait. This solution would
        // be ideal but for now a workaround should be sufficient
        // return Ok(State::default() == current);

        Ok([
            (current.books.len(), 0),
            (current.editions.len(), 0),
            (current.authors.len(), 1), // The authors table has one entry in it by default
            (current.reviews.len(), 0),
            (current.edition_reviews.len(), 0),
            (current.progress.len(), 0),
        ]
        .into_iter()
        .filter(|(a, b)| a != b)
        .next()
        .is_none())
    }

    /// Serialize the state to a string
    pub fn serialize(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deseriablize from a string to state
    pub fn deserialize(s: String) -> Result<State> {
        Ok(serde_json::from_str(&s)?)
    }

    /// Rebuild the database from state
    pub async fn rebuild(&self, conn: &sqlx::SqlitePool) -> Result<()> {
        if !State::is_fresh(conn).await? {
            anyhow::bail!("Database seems to hold data, refusing to overwrite.");
        }

        let all: Vec<Uuid> = Mood::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.moods {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Pace::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.paces {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Genre::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.genres {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Language::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.languages {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Publisher::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.publishers {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Book::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.books {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Edition::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.editions {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Author::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.authors {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Review::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.reviews {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = EditionReview::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.edition_reviews {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Progress::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.progress {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Series::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.series {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = Binding::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.bindings {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<Uuid> = EditionFormat::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect();
        for x in &self.edition_formats {
            if !all.contains(&x.id) {
                x.insert(&conn).await?;
            }
        }

        let all: Vec<(Uuid, Uuid)> = BookAuthor::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| (x.book_id, x.author_id))
            .collect();
        for x in &self.book_authors {
            if !all.contains(&(x.book_id.clone(), x.author_id.clone())) {
                let x1 = self
                    .books
                    .iter()
                    .filter(|y| y.id == x.book_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find book with id {}",
                        x.book_id
                    ))?;
                let x2 = self
                    .authors
                    .iter()
                    .filter(|y| y.id == x.author_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find author with id {}",
                        x.author_id
                    ))?;
                BookAuthor::insert(&conn, x1, x2).await?;
            }
        }

        let all: Vec<(Uuid, Uuid)> = BookGenre::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| (x.book_id, x.genre_id))
            .collect();
        for x in &self.book_genres {
            if !all.contains(&(x.book_id.clone(), x.genre_id.clone())) {
                let x1 = self
                    .books
                    .iter()
                    .filter(|y| y.id == x.book_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find book with id {}",
                        x.book_id
                    ))?;
                let x2 = self
                    .genres
                    .iter()
                    .filter(|y| y.id == x.genre_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find genre with id {}",
                        x.genre_id
                    ))?;
                BookGenre::insert(&conn, x1, x2).await?;
            }
        }

        let all: Vec<(Uuid, Uuid)> = EditionLanguage::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| (x.edition_id, x.language_id))
            .collect();
        for x in &self.edition_languages {
            if !all.contains(&(x.edition_id.clone(), x.language_id.clone())) {
                let x1 = self
                    .editions
                    .iter()
                    .filter(|y| y.id == x.edition_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find edition with id {}",
                        x.edition_id
                    ))?;
                let x2 = self
                    .languages
                    .iter()
                    .filter(|y| y.id == x.language_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find language with id {}",
                        x.language_id
                    ))?;
                EditionLanguage::insert(&conn, x1, x2).await?;
            }
        }

        let all: Vec<(Uuid, Uuid)> = EditionPublisher::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| (x.edition_id, x.publisher_id))
            .collect();
        for x in &self.edition_publishers {
            if !all.contains(&(x.edition_id.clone(), x.publisher_id.clone())) {
                let x1 = self
                    .editions
                    .iter()
                    .filter(|y| y.id == x.edition_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find edition with id {}",
                        x.edition_id
                    ))?;
                let x2 = self
                    .publishers
                    .iter()
                    .filter(|y| y.id == x.publisher_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find publisher with id {}",
                        x.publisher_id
                    ))?;
                EditionPublisher::insert(&conn, x1, x2).await?;
            }
        }

        let all: Vec<(Uuid, Uuid)> = ReviewMood::get_all(&conn)
            .await?
            .into_iter()
            .map(|x| (x.review_id, x.mood_id))
            .collect();
        for x in &self.review_moods {
            if !all.contains(&(x.review_id.clone(), x.mood_id.clone())) {
                let x1 = self
                    .reviews
                    .iter()
                    .filter(|y| y.id == x.review_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find review with id {}",
                        x.review_id
                    ))?;
                let x2 = self
                    .moods
                    .iter()
                    .filter(|y| y.id == x.mood_id)
                    .next()
                    .ok_or(anyhow::anyhow!(
                        "Inconsistency in database, couldn't find mood with id {}",
                        x.mood_id
                    ))?;
                ReviewMood::insert(&conn, x1, x2).await?;
            }
        }

        Ok(())
    }
}
