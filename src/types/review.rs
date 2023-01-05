use crate::{
    traits::{CreateByPrompt, Insertable},
    types::{pace::Pace, timestamp::Timestamp, uuid::Uuid},
};

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
