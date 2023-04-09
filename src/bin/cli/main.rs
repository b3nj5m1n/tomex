use anyhow::Result;
use reedline::Signal;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    Pool, SqlitePool,
};
use std::{env, fs, path::PathBuf, process::exit};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod command_parser;
mod openlib_schema;
mod openlibrary;
mod prompt;
mod repl;
mod server;

use tomex::{
    backup, config,
    export::Export,
    traits::*,
    types::{
        author::Author, binding::Binding, book::Book, book_author::BookAuthor,
        book_genre::BookGenre, edition::Edition, edition_language::EditionLanguage,
        edition_publisher::EditionPublisher, edition_review::EditionReview, format::EditionFormat,
        genre::Genre, language::Language, mood::Mood, pace::Pace, progress::Progress,
        publisher::Publisher, review::Review, review_mood::ReviewMood, series::Series,
    },
};

async fn handle_command(command: String, conn: &SqlitePool, config: &config::Config) -> Result<()> {
    let args = command_parser::arg_parser_repl();
    let command = shlex::split(&command);
    if command.is_none() {
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
            Some(("by_isbn", _matches)) => {
                let isbn = PromptType::create_by_prompt(
                    "What is the ISBN?",
                    None::<&tomex::types::isbn::Isbn>,
                    conn,
                )
                .await?;
                openlibrary::create_by_isbn(&isbn.0.to_string(), conn).await?;
            }
            Some(("book", _matches)) => {
                Book::insert_by_prompt(conn).await?;
            }
            Some(("series", _matches)) => {
                Series::insert_by_prompt(conn).await?;
            }
            Some(("review", _matches)) => {
                Review::insert_by_prompt(conn).await?;
            }
            Some(("edition", _matches)) => {
                Edition::insert_by_prompt(conn).await?;
            }
            Some(("edition-review", _matches)) => {
                EditionReview::insert_by_prompt(conn).await?;
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
            Some(("pace", _matches)) => {
                Pace::insert_by_prompt(conn).await?;
            }
            Some(("language", _matches)) => {
                Language::insert_by_prompt(conn).await?;
            }
            Some(("publisher", _matches)) => {
                Publisher::insert_by_prompt(conn).await?;
            }
            Some(("progress", _matches)) => {
                Progress::insert_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("edit", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("series", _matches)) => {
                Series::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("review", _matches)) => {
                Review::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("edition", _matches)) => {
                Edition::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("edition-review", _matches)) => {
                EditionReview::update_by_prompt_by_prompt(conn).await?;
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
            Some(("pace", _matches)) => {
                Pace::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("language", _matches)) => {
                Language::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("publisher", _matches)) => {
                Publisher::update_by_prompt_by_prompt(conn).await?;
            }
            Some(("progress", _matches)) => {
                Progress::update_by_prompt_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("remove", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::remove_by_prompt(conn).await?;
            }
            Some(("series", _matches)) => {
                Series::remove_by_prompt(conn).await?;
            }
            Some(("review", _matches)) => {
                Review::remove_by_prompt(conn).await?;
            }
            Some(("edition", _matches)) => {
                Edition::remove_by_prompt(conn).await?;
            }
            Some(("edition-review", _matches)) => {
                EditionReview::remove_by_prompt(conn).await?;
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
            Some(("pace", _matches)) => {
                Pace::remove_by_prompt(conn).await?;
            }
            Some(("language", _matches)) => {
                Language::remove_by_prompt(conn).await?;
            }
            Some(("publisher", _matches)) => {
                Publisher::remove_by_prompt(conn).await?;
            }
            Some(("progress", _matches)) => {
                Progress::remove_by_prompt(conn).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("query", _matches)) => match _matches.subcommand() {
            Some(("book", _matches)) => {
                Book::query_by_clap(conn, _matches, config).await?;
            }
            Some(("series", _matches)) => {
                Series::query_by_clap(conn, _matches, config).await?;
            }
            Some(("review", _matches)) => {
                Review::query_by_clap(conn, _matches, config).await?;
            }
            Some(("edition", _matches)) => {
                Edition::query_by_clap(conn, _matches, config).await?;
            }
            Some(("edition-review", _matches)) => {
                EditionReview::query_by_clap(conn, _matches, config).await?;
            }
            Some(("author", _matches)) => {
                Author::query_by_clap(conn, _matches, config).await?;
            }
            Some(("genre", _matches)) => {
                Genre::query_by_clap(conn, _matches, config).await?;
            }
            Some(("mood", _matches)) => {
                Mood::query_by_clap(conn, _matches, config).await?;
            }
            Some(("pace", _matches)) => {
                Pace::query_by_clap(conn, _matches, config).await?;
            }
            Some(("language", _matches)) => {
                Language::query_by_clap(conn, _matches, config).await?;
            }
            Some(("publisher", _matches)) => {
                Publisher::query_by_clap(conn, _matches, config).await?;
            }
            Some(("progress", _matches)) => {
                Progress::query_by_clap(conn, _matches, config).await?;
            }
            Some((name, _matches)) => unimplemented!("{}", name),
            None => unreachable!("subcommand required"),
        },
        Some(("listen", _matches)) => {
            crate::server::start(conn).await;
        }
        Some(("exit", _matches)) => {
            exit(0);
        }
        Some((name, _matches)) => unimplemented!("{}", name),
        None => unreachable!("subcommand required"),
    }
    Ok(())
}

async fn connect_to_db(db_url: PathBuf) -> Result<SqlitePool> {
    let db_url = shellexpand::full(
        db_url
            .to_str()
            .ok_or(anyhow::anyhow!("Invalid unicode found in path to database"))?,
    )?;
    let db_url = PathBuf::from(db_url.into_owned());
    std::fs::create_dir_all(db_url.parent().ok_or(anyhow::anyhow!(
        "Couldn't extract parent directory from database location"
    ))?)?;
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
        Series::init_table(conn),
        Review::init_table(conn),
        Edition::init_table(conn),
        EditionReview::init_table(conn),
        Publisher::init_table(conn),
        Genre::init_table(conn),
        Mood::init_table(conn),
        Pace::init_table(conn),
        Language::init_table(conn),
        Progress::init_table(conn),
        Binding::init_table(conn),
        EditionFormat::init_table(conn),
        BookAuthor::create_table(conn),
        BookGenre::create_table(conn),
        EditionLanguage::create_table(conn),
        EditionPublisher::create_table(conn),
        ReviewMood::create_table(conn),
    )?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args_parsed = command_parser::arg_parser_cli().get_matches_from(env::args_os().skip(1));

    let config = config::Config::read_config()?;

    let conn = connect_to_db(config.database_location.clone()).await?;

    create_tables(&conn).await?;
    // println!("{}", config::Config::default_as_string()?);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    if let Some(("repl", _)) = args_parsed.subcommand() {
        let mut repl = repl::Repl::new(command_parser::generate_completions());
        loop {
            match repl.read_line() {
                Ok(Signal::Success(buffer)) => {
                    match handle_command(buffer.clone(), &conn, &config).await {
                        Ok(_) => (),
                        Err(e) => println!("Error: {e}"),
                    };
                }
                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                }
                x => {
                    println!("Event: {x:?}");
                }
            }
        }
    } else if let Some(("backup", _)) = args_parsed.subcommand() {
        let mut state = backup::State::load(&conn).await?;
        state.sort();
        println!("{}", state.serialize()?);
    } else if let Some(("restore", x)) = args_parsed.subcommand() {
        let content = fs::read_to_string(
            x.get_one::<String>("file")
                .ok_or(anyhow::anyhow!("Couldn't read backup from specified file."))?,
        )?;
        let mut state = backup::State::deserialize(content)?;
        backup::State::rebuild(&state, &conn).await?;
    } else if let Some(("export", _)) = args_parsed.subcommand() {
        let export = Export::new(&conn).await?;
        Export::export(export)?;
    } else {
        let args = env::args_os()
            .skip(1)
            .map(|x| x.into_string().expect("Invalid unicode in arguments"))
            .collect::<Vec<String>>()
            .join(" ");
        handle_command(args, &conn, &config).await?;
    }

    conn.close().await;

    Ok(())
}
