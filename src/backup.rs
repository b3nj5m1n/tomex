use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    traits::*,
    types::{
        author::Author, book::Book, book_author::BookAuthor, book_genre::BookGenre,
        edition::Edition, edition_language::EditionLanguage, edition_publisher::EditionPublisher,
        edition_review::EditionReview, genre::Genre, language::Language, mood::Mood, pace::Pace,
        progress::Progress, publisher::Publisher, review::Review,
    },
};

/// Contains the entire state of the database
#[derive(Serialize, Deserialize)]
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
    book_authors:       Vec<BookAuthor>,
    book_genres:        Vec<BookGenre>,
    edition_languages:  Vec<EditionLanguage>,
    edition_publishers: Vec<EditionPublisher>,
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
            book_authors:       BookAuthor::get_all(conn).await?,
            book_genres:        BookGenre::get_all(conn).await?,
            edition_languages:  EditionLanguage::get_all(conn).await?,
            edition_publishers: EditionPublisher::get_all(conn).await?,
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
        self.book_authors.sort_by_key(|x| x.book_id.clone());
        self.book_genres.sort_by_key(|x| x.book_id.clone());
        self.edition_languages.sort_by_key(|x| x.edition_id.clone());
        self.edition_publishers
            .sort_by_key(|x| x.edition_id.clone());
    }

    /// Serialize the state to a string
    pub fn serialize(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deseriablize from a string to state
    pub fn deserialize(s: String) -> Result<State> {
        Ok(serde_json::from_str(&s)?)
    }

    /// Rebuild the database from state
    pub async fn rebuild(&self) -> Result<()> {
        todo!();
    }
}
