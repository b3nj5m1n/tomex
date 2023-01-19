use anyhow::Result;
use tomex::types::{
    text::Text,
    timestamp::{OptionalTimestamp, Timestamp},
};

use crate::openlib_schema::{
    author::Author as OpenLibAuthor, book::Book as OpenLibBook, edition::Edition as OpenLibEdition,
};
use tomex::types::{book::Book, edition::Edition};

pub async fn isbn_to_edition(isbn: &str, _conn: &sqlx::SqlitePool) -> Result<OpenLibEdition> {
    let url = format!("https://openlibrary.org/isbn/{}.json", isbn);
    let resp = reqwest::get(url).await?.json::<OpenLibEdition>().await?;
    Ok(resp)
}

pub async fn build_edition(edition: OpenLibEdition, book: Book, isbn: &str) -> Edition {
    let release_date = match &edition.publish_date {
        Some(publish_date) => OptionalTimestamp(match dateparser::parse(publish_date) {
            Ok(timestamp) => Some(Timestamp(timestamp)),
            Err(_) => None,
        }),
        None => OptionalTimestamp(None),
    };
    Edition {
        id:                  tomex::types::uuid::Uuid(uuid::Uuid::new_v4()),
        book_id:             book.id,
        edition_title:       if Some(book.title.0.clone()) == edition.title {
            None
        } else {
            edition.title.map(|x| Text(x))
        },
        edition_description: None,
        isbn:                Some(Text(isbn.to_string())),
        pages:               edition.number_of_pages,
        languages:           None, // TODO
        release_date:        release_date,
        format_id:           None,
        format:              None,
        height:              None,
        width:               None,
        thickness:           None,
        weight:              None,
        binding_id:          None,
        binding:             None,
        publishers:          None, // TODO
        cover:               None,
        part_index:          None,
        reviews:             None,
        progress:            None,
        deleted:             false,
        book_title:          book.title,
    }
}

pub async fn edition_to_book(
    edition: &OpenLibEdition,
    _conn: &sqlx::SqlitePool,
) -> Result<OpenLibBook> {
    let url = format!(
        "https://openlibrary.org{}.json",
        edition
            .works
            .clone()
            .ok_or(anyhow::anyhow!(
                "Couldn't find any books linked to this edition"
            ))?
            .get(0)
            .ok_or(anyhow::anyhow!(
                "Couldn't find any books linked to this edition"
            ))?
            .key
    );
    println!("{url}");
    let resp = reqwest::get(url).await?.json::<OpenLibBook>().await?;
    Ok(resp)
}

pub async fn edition_to_authors(
    edition: &OpenLibEdition,
    _conn: &sqlx::SqlitePool,
) -> Result<Vec<OpenLibAuthor>> {
    let authors_keys = edition
        .authors
        .clone()
        .ok_or(anyhow::anyhow!("No authors found on OpenLibrary"))?
        .into_iter()
        .map(|x| x.key)
        .collect::<Vec<String>>();
    let mut authors = Vec::with_capacity(authors_keys.len());
    for key in authors_keys {
        let url = format!("https://openlibrary.org{}.json", key);
        let resp = reqwest::get(url).await?.json::<OpenLibAuthor>().await?;
        authors.push(resp);
    }
    Ok(authors)
}

pub async fn create_by_isbn(
    isbn: &str,
    conn: &sqlx::SqlitePool,
) -> Result<tomex::types::book::Book> {
    let edition = isbn_to_edition(isbn, conn).await?;

    println!("Edition:\n{}", serde_json::to_string_pretty(&edition)?);

    let authors = edition_to_authors(&edition, conn).await?;

    println!("Authors:\n{}", serde_json::to_string_pretty(&authors)?);

    let book = edition_to_book(&edition, conn).await?;

    println!("Book:\n{}", serde_json::to_string_pretty(&book)?);

    todo!()

    // TODO: Author
    // TODO: Edition
    // Ok(tomex::types::book::Book {
    //     id:           tomex::types::uuid::Uuid(uuid::Uuid::new_v4()),
    //     title:        Text(book.title.clone().ok_or(anyhow!("Title not
    // found"))?),     authors:      None,                    // TODO
    //     release_date: OptionalTimestamp(None), // TODO
    //     summary:      None,                    // TODO
    //     series_id:    None,                    // TODO
    //     series_index: None,                    // TODO
    //     series:       None,                    // TODO
    //     editions:     None,                    // TODO
    //     reviews:      None,                    // TODO
    //     genres:       None,                    // TODO
    //     deleted:      false,
    // })
}
