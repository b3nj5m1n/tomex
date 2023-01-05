use crate::{
    traits::{CreateByPrompt, Insertable},
    types::uuid::Uuid,
};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Publisher {
    pub id: i32,
    pub name: String,
}
