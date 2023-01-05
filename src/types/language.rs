use crate::traits::{CreateByPrompt, Insertable};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Language {
    pub id: i32,
    pub name: String,
}
