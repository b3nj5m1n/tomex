use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub title:       String,
    pub authors:     Option<Vec<Author>>,
    pub description: Option<Description>,
    // pub subjects:    Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Description {
    Simple(String),
    Complex(DescriptionComplex),
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescriptionComplex {
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub author: AuthorName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorName {
    pub key: String,
}
