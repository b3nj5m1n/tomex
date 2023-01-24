use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub name:       String,
    pub birth_date: Option<String>,
    pub death_date: Option<String>,
    // pub bio:        Option<Bio>,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct Bio {
//     pub value: String,
// }
