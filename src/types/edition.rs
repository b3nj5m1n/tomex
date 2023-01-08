use crate::types::{
    edition_review::EditionReview, language::Language, mood::Mood, pace::Pace, progress::Progress,
    publisher::Publisher, timestamp::Timestamp,
};

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
