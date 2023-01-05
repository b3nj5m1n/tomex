use crate::{
    traits::{CreateByPrompt, Insertable},
    types::{timestamp::Timestamp, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub id: i32,
    pub edition_id: i32,
    pub timestamp: Timestamp,
    pub pages_progress: i32,
}
