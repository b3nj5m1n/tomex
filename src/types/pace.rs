use crate::traits::{CreateByPrompt, Insertable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pace {
    Slow,
    Medium,
    Fast,
}
