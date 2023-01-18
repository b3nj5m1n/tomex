use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tomex::types::{text::Text, timestamp::OptionalTimestamp};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub url:             String,
    pub key:             String,
    pub title:           Option<String>,
    pub subtitle:        Option<String>,
    pub authors:         Option<Vec<Author>>,
    pub identifiers:     Option<Identifiers>,
    pub classifications: Option<Classifications>,
    pub publishers:      Option<Vec<Publisher>>,
    pub publish_date:    Option<String>,
    pub subjects:        Option<Vec<Subject>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub url:  String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identifiers {
    pub isbn_10:     Option<Vec<String>>,
    pub isbn_13:     Option<Vec<String>>,
    pub oclc:        Option<Vec<String>>,
    pub openlibrary: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Classifications {
    pub lc_classifications: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Publisher {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub url:  String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub title: String,
    pub url:   String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formats {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cover {
    pub small:  String,
    pub medium: String,
    pub large:  String,
}

pub async fn create_by_isbn(
    isbn: &str,
    _conn: &sqlx::SqlitePool,
) -> Result<tomex::types::book::Book> {
    let isbn_prefixed = format!("ISBN:{isbn}");
    let url = "https://openlibrary.org/api/books";
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("bibkeys", isbn_prefixed.as_str());
    params.insert("jscmd", "data");
    params.insert("format", "json");
    let resp = client.get(url).query(&params).send().await?.text().await?;
    let books: HashMap<String, Book> = serde_json::from_str(&resp)?;
    let book = books.get(&isbn_prefixed).ok_or(anyhow!(
        "Book not found in response, might not be in the OpenLibrary database"
    ))?;
    // TODO: Author
    // TODO: Edition
    Ok(tomex::types::book::Book {
        id:           tomex::types::uuid::Uuid(uuid::Uuid::new_v4()),
        title:        Text(book.title.clone().ok_or(anyhow!("Title not found"))?),
        authors:      None,                    // TODO
        release_date: OptionalTimestamp(None), // TODO
        summary:      None,                    // TODO
        series_id:    None,                    // TODO
        series_index: None,                    // TODO
        series:       None,                    // TODO
        editions:     None,                    // TODO
        reviews:      None,                    // TODO
        genres:       None,                    // TODO
        deleted:      false,
    })
}
