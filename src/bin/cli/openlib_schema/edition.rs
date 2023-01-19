use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edition {
    pub publishers:      Option<Vec<String>>,
    pub subtitle:        Option<String>,
    pub title:           Option<String>,
    pub physical_format: Option<String>,
    pub publish_date:    Option<String>,
    pub authors:         Option<Vec<Author>>,
    pub works:           Option<Vec<Work>>,
    pub number_of_pages: Option<u32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Work {
    pub key: String,
}
