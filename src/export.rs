use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::{
    traits::Queryable,
    types::{
        edition::Edition,
        progress::{PagesProgress, Progress},
    },
};

#[derive(Debug, Default, Serialize)]
pub struct Export {
    #[serde(rename = "Book Id")]
    book_id: Option<String>,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "Author l-f")]
    author_lf: Option<String>,
    #[serde(rename = "Additional Authors")]
    additional_authors: Option<String>,
    #[serde(rename = "ISBN")]
    isbn: Option<String>,
    #[serde(rename = "ISBN13")]
    isbn13: Option<String>,
    #[serde(rename = "My Rating")]
    my_rating: Option<String>,
    #[serde(rename = "Average Rating")]
    average_rating: Option<String>,
    #[serde(rename = "Publisher")]
    publisher: Option<String>,
    #[serde(rename = "Binding")]
    binding: Option<String>,
    #[serde(rename = "Number of Pages")]
    number_of_pages: Option<String>,
    #[serde(rename = "Year Published")]
    year_published: Option<String>,
    #[serde(rename = "Original Publication Year")]
    original_publication_year: Option<String>,
    #[serde(rename = "Date Read")]
    date_read: Option<String>,
    #[serde(rename = "Date Added")]
    date_added: Option<String>,
    #[serde(rename = "Bookshelves")]
    bookshelves: Option<String>,
    #[serde(rename = "Bookshelves with positions")]
    bookshelves_with_positions: Option<String>,
    #[serde(rename = "Exclusive Shelf")]
    exclusive_shelf: Option<String>,
    #[serde(rename = "My Review")]
    my_review: Option<String>,
    #[serde(rename = "Spoiler")]
    spoiler: Option<String>,
    #[serde(rename = "Private Notes")]
    private_notes: Option<String>,
    #[serde(rename = "Read Count")]
    read_count: Option<String>,
    #[serde(rename = "Owned Copies")]
    owned_copies: String,
}

impl Export {
    pub async fn new(conn: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let progress_updates = Progress::get_all(conn)
            .await?
            .into_iter()
            .filter(|x| {
                if let PagesProgress::Pages(0) = x.pages_progress {
                    false
                } else {
                    true
                }
            })
            .collect::<Vec<Progress>>();
        let mut editions_read = HashMap::new();
        for progress_update in progress_updates.clone() {
            if let PagesProgress::Started = progress_update.pages_progress {
                let matching = progress_updates.iter().find(|x| {
                    if let PagesProgress::Finished = x.pages_progress {
                        x.edition_id == progress_update.edition_id
                    } else {
                        false
                    }
                });
                if let Some(finished) = matching {
                    editions_read.insert(
                        progress_update.edition_id.0,
                        (progress_update.timestamp, finished.timestamp.clone()),
                    );
                }
            }
        }
        let mut result = Vec::new();
        for (edition_id, (timestamp_started, timestamp_finished)) in editions_read.into_iter() {
            let edition = Edition::get_by_id(conn, &crate::types::uuid::Uuid(edition_id)).await?;
            result.push(Self {
                isbn: Some(format!("=\"{}\"", "")),
                isbn13: Some(format!(
                    "=\"{}\"",
                    match edition.isbn {
                        Some(s) => s.0,
                        None => "".to_string(),
                    }
                )),
                date_read: Some(timestamp_finished.0.format("%Y/%m/%d").to_string()),
                date_added: Some(timestamp_started.0.format("%Y/%m/%d").to_string()),
                exclusive_shelf: Some("read".into()),
                ..Self::default()
            });
        }
        Ok(result)
    }

    pub fn export(data: Vec<Self>) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for record in data {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
