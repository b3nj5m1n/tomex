use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub title:       String,
    pub authors:     Option<Vec<Author>>,
    pub description: Option<String>,
    pub subjects:    Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub author: AuthorName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorName {
    pub key: String,
}
