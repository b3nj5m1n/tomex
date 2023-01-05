use std::env;

use anyhow::Result;
use dotenvy::{dotenv, var as envar};
use reedline::Signal;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Pool, SqlitePool,
};
// use uuid::Uuid;

mod command_parser;
mod prompt;
mod repl;

use bokhylle::types::author::Author;
use bokhylle::{traits::*, types::book::Book};

async fn handle_command(command: String, conn: SqlitePool) -> Result<()> {
    let args = command_parser::arg_parser();
    let command = shlex::split(&command);
    if let None = command {
        anyhow::bail!("Invalid command");
    }
    let command = command.unwrap();
    let matches = args.try_get_matches_from(command);
    if let Err(e) = matches {
        anyhow::bail!(e);
    }
    let matches = matches.unwrap();
    match matches.subcommand() {
        Some(("add", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                let book = Book::create_by_prompt(conn.clone()).await?;
                book.insert(conn).await?;
            }
            Some(("author", _matches)) => {
                let author = Author::create_by_prompt(conn.clone()).await?;
                author.insert(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("remove", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                println!("Removing book")
            }
            Some(("author", _matches)) => {
                println!("Removing author")
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("query", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                println!("\nBooks:");
                let books = sqlx::query_as::<_, Book>("SELECT * FROM books;")
                    .fetch_all(&conn)
                    .await?;
                for book in books {
                    println!("{}", book);
                }
            }
            Some(("author", _matches)) => {
                println!("\nAuthors:");
                let authors = sqlx::query_as::<_, Author>("SELECT * FROM authors;")
                    .fetch_all(&conn)
                    .await?;
                for author in authors {
                    println!("{}", author);
                }
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some((name, _matches)) => unimplemented!("{}", name),
        None => unreachable!("subcommand required"),
    }
    Ok(())
}

const DEFAULT_DB: &str = "test.db";

async fn connect_to_db(db_url: Option<String>) -> Result<SqlitePool> {
    let db_url = match db_url {
        Some(db_url) => db_url,
        None => match dotenv() {
            Ok(_) => match envar("DATABASE_URL").ok() {
                Some(db_url) => db_url,
                None => DEFAULT_DB.to_string(),
            },
            Err(_) => DEFAULT_DB.to_string(),
        },
    };

    Ok(Pool::connect_with(
        SqliteConnectOptions::new()
            .filename(db_url)
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true),
    )
    .await?)
}

async fn create_tables(conn: &SqlitePool) -> Result<()> {
    // Create test table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS authors (
            id TEXT PRIMARY KEY NOT NULL,
            name_first TEXT,
            name_last TEXT,
            date_born INTEGER,
            date_died INTEGER,
            deleted BOOL DEFAULT FALSE
        );
        CREATE TABLE IF NOT EXISTS books (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            author TEXT,
            release_date INTEGER,
            deleted BOOL DEFAULT FALSE,
            FOREIGN KEY (author) REFERENCES authors (id)
        );"#,
    )
    .execute(conn)
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args_parsed = command_parser::arg_parser_cli().get_matches_from(env::args_os().skip(1));

    let conn = connect_to_db(None).await?;

    create_tables(&conn).await?;

    if let Some(("repl", _)) = args_parsed.subcommand() {
        let mut repl = repl::Repl::new(command_parser::generate_completions());
        loop {
            match repl.read_line() {
                Ok(Signal::Success(buffer)) => {
                    match handle_command(buffer.clone(), conn.clone()).await {
                        Ok(_) => (),
                        Err(e) => println!("Error: {}", e),
                    };
                }
                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                }
                x => {
                    println!("Event: {:?}", x);
                }
            }
        }
    } else {
        let args = env::args_os()
            .skip(1)
            .map(|x| x.into_string().expect("Invalid unicode in arguments"))
            .collect::<Vec<String>>()
            .join(" ");
        handle_command(args, conn).await?;
    }

    Ok(())
}