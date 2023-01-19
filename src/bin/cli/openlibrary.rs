use anyhow::Result;
use tomex::{
    traits::{Insertable, PromptType},
    types::{
        author::Author,
        text::Text,
        timestamp::{OptionalTimestamp, Timestamp},
    },
};

use crate::openlib_schema::{
    author::Author as OpenLibAuthor, book::Book as OpenLibBook, edition::Edition as OpenLibEdition,
};
use tomex::types::{book::Book, edition::Edition};

fn opt_str_to_optional_timestamp(input: &Option<String>) -> OptionalTimestamp {
    match input {
        Some(x) => OptionalTimestamp(match dateparser::parse(x) {
            Ok(timestamp) => Some(Timestamp(timestamp)),
            Err(_) => None,
        }),
        None => OptionalTimestamp(None),
    }
}

pub async fn isbn_to_edition(isbn: &str, _conn: &sqlx::SqlitePool) -> Result<OpenLibEdition> {
    let url = format!("https://openlibrary.org/isbn/{}.json", isbn);
    let resp = reqwest::get(url).await?.json::<OpenLibEdition>().await?;
    Ok(resp)
}

pub async fn build_edition(edition: OpenLibEdition, book: Book, isbn: &str) -> Edition {
    let release_date = opt_str_to_optional_timestamp(&edition.publish_date);
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
    let resp = reqwest::get(url).await?.json::<OpenLibBook>().await?;
    Ok(resp)
}

pub async fn build_book(book: OpenLibBook, authors: Option<Vec<Author>>) -> Book {
    Book {
        id:           tomex::types::uuid::Uuid(uuid::Uuid::new_v4()),
        title:        Text(book.title),
        authors:      authors,
        release_date: OptionalTimestamp(None),
        summary:      book.description.map(Text),
        series_id:    None,
        series_index: None,
        series:       None,
        editions:     None,
        reviews:      None,
        genres:       None,
        deleted:      false,
    }
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

pub async fn build_author(author: OpenLibAuthor) -> Author {
    Author {
        id:        tomex::types::uuid::Uuid(uuid::Uuid::new_v4()),
        name:      Some(Text(author.name)),
        date_born: opt_str_to_optional_timestamp(&author.birth_date),
        date_died: opt_str_to_optional_timestamp(&author.death_date),
        deleted:   false,
        special:   false,
    }
}

pub async fn create_by_isbn(
    isbn: &str,
    conn: &sqlx::SqlitePool,
) -> Result<tomex::types::edition::Edition> {
    let edition = isbn_to_edition(isbn, conn).await?;

    // println!("Edition:\n{}", serde_json::to_string_pretty(&edition)?);

    let authors_auto = edition_to_authors(&edition, conn).await?;
    let mut authors = Vec::with_capacity(authors_auto.len());

    for author in authors_auto {
        let potential_author = Author::get_by_name(conn, author.name.clone()).await?;
        match potential_author {
            Some(author_in_db) => {
                println!("Author found in database: {author_in_db}");

                if inquire::Confirm::new("Use this author?")
                    .with_default(true)
                    .prompt()?
                {
                    authors.push(author_in_db);
                } else {
                    // TODO: Extract this into a function, allow user to select an existing author
                    // if names don't match exactly
                    let author_auto = build_author(author).await;
                    let author: Author =
                        PromptType::update_by_prompt(&author_auto, "", conn).await?;
                    author.insert(conn).await?;
                    authors.push(author);
                };
            }
            None => {
                println!("Author not found in database.");
                let author_auto = build_author(author).await;
                let author: Author = PromptType::update_by_prompt(&author_auto, "", conn).await?;
                author.insert(conn).await?;
                authors.push(author);
            }
        }
    }

    // println!("Authors:\n{}", serde_json::to_string_pretty(&authors)?);

    let book_auto = edition_to_book(&edition, conn).await?;

    let potential_book = Book::get_by_title(conn, book_auto.title.clone()).await?;
    let book = match potential_book {
        Some(book_in_db) => {
            println!("Book found in database: {book_in_db}");

            if inquire::Confirm::new("Use this book?")
                .with_default(true)
                .prompt()?
            {
                book_in_db
            } else {
                let book_auto = build_book(book_auto, Some(authors)).await;
                let book = PromptType::update_by_prompt(&book_auto, "", conn).await?;
                book.insert(conn).await?;
                book
            }
        }
        None => {
            let book_auto = build_book(book_auto, Some(authors)).await;
            let book = PromptType::update_by_prompt(&book_auto, "", conn).await?;
            book.insert(conn).await?;
            book
        }
    };

    // println!("Book:\n{}", serde_json::to_string_pretty(&book)?);

    let edition_auto = build_edition(edition, book, isbn).await;
    let edition = PromptType::update_by_prompt(&edition_auto, "", conn).await?;
    edition.insert(conn).await?;
    Ok(edition)
}
