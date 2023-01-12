use anyhow::Result;
use dotenvy::{dotenv, var as envar};
use reedline::Signal;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Pool, SqlitePool,
};
use std::{env, process::exit};

mod command_parser;
mod prompt;
mod repl;

use bokhylle::types::{
    author::Author, book_author::BookAuthor, book_genre::BookGenre, genre::Genre, mood::Mood,
};
use bokhylle::{traits::*, types::book::Book};

async fn handle_command(command: String, conn: &SqlitePool) -> Result<()> {
    let args = command_parser::arg_parser_repl();
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
                Book::insert_by_prompt(conn).await?;
            }
            Some(("author", _matches)) => {
                Author::insert_by_prompt(conn).await?;
            }
            Some(("genre", _matches)) => {
                Genre::insert_by_prompt(conn).await?;
            }
            Some(("mood", _matches)) => {
                Mood::insert_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("edit", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("author", _matches)) => {
                Author::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("genre", _matches)) => {
                Genre::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("mood", _matches)) => {
                Mood::update_by_prompt_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("remove", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::remove_by_prompt(conn).await?;
            }
            Some(("author", _matches)) => {
                Author::remove_by_prompt(conn).await?;
            }
            Some(("genre", _matches)) => {
                Genre::remove_by_prompt(conn).await?;
            }
            Some(("mood", _matches)) => {
                Mood::remove_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("query", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::query_by_clap(conn, _matches).await?;
            }
            Some(("author", _matches)) => {
                Author::query_by_clap(conn, _matches).await?;
            }
            Some(("genre", _matches)) => {
                Genre::query_by_clap(conn, _matches).await?;
            }
            Some(("mood", _matches)) => {
                Mood::query_by_clap(conn, _matches).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("exit", _matches)) => {
            exit(0);
        }
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
    tokio::try_join!(
        Author::init_table(conn),
        Book::init_table(conn),
        Genre::init_table(conn),
        Mood::init_table(conn),
        BookAuthor::create_table(conn),
        BookGenre::create_table(conn),
    )?;
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
                    match handle_command(buffer.clone(), &conn).await {
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
        handle_command(args, &conn).await?;
    }

    conn.close().await;

    Ok(())
}
