use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    traits::*,
    types::{mood::Mood, review::Review, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct ReviewMood {
    pub review_id: Uuid,
    pub mood_id: Uuid,
}

impl JunctionTable<Review, Mood> for ReviewMood {
    const TABLE_NAME: &'static str = formatcp!("{}_{}", Review::NAME_SINGULAR, Mood::NAME_SINGULAR);

    async fn get_id_a(&self) -> &Uuid {
        &self.review_id
    }

    async fn get_id_b(&self) -> &Uuid {
        &self.mood_id
    }
}
