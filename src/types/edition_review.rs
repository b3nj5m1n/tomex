use crate::{
    traits::{CreateByPrompt, Insertable},
    types::timestamp::Timestamp,
};

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
