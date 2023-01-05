use crate::{
    traits::{CreateByPrompt, Insertable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mood {
    Adventurous,
    Challenging,
    Dark,
    Emotional,
    Funny,
    Hopeful,
    Informative,
    Inspiring,
    Lighthearted,
    Mysterious,
    Reflective,
    Relaxing,
    Sad,
    Tense,
}
