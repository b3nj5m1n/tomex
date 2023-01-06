use std::fmt::Display;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow, Row,
};

use crate::types::uuid::Uuid;

pub trait CreateByPrompt {
    async fn create_by_prompt(conn: sqlx::SqlitePool) -> Result<Self>
    where
        Self: Sized;
}

pub trait Insertable {
    async fn insert(self, conn: sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Sized;
}

pub trait CreateTable
where
    Self: Sized,
    Self: DbTable,
{
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()>;
}

pub trait DbTable
where
    Self: Sized,
{
    const NAME_SINGULAR: &'static str;
    const NAME_PLURAL: &'static str;
    const TABLE_NAME: &'static str = Self::NAME_PLURAL;
}

pub trait Queryable
where
    for<'r> Self: FromRow<'r, sqlx::sqlite::SqliteRow>,
    Self: DbTable,
    Self: Sized,
    Self: Display,
    Self: Send,
    Self: Unpin,
{
    async fn get_by_id(conn: sqlx::SqlitePool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE id = ?1;",
            Self::TABLE_NAME
        ))
        .bind(id)
        .fetch_one(&conn)
        .await?)
    }
    async fn get_all(conn: sqlx::SqlitePool) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as::<_, Self>(&format!("SELECT * FROM {};", Self::TABLE_NAME))
                .fetch_all(&conn)
                .await?,
        )
    }
    async fn query_by_prompt(conn: sqlx::SqlitePool) -> Result<Option<Self>> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt_skippable()?)
    }
    async fn query_by_clap(conn: sqlx::SqlitePool, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("interactive") {
            match Self::query_by_prompt(conn.clone()).await? {
                Some(x) => println!("{}", x),
                None => println!("No {} selected.", Self::NAME_SINGULAR),
            }
        }
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("uuid") {
            match matches.get_one::<String>("uuid") {
                Some(uuid_str) => match uuid::Uuid::parse_str(uuid_str) {
                    Ok(uuid) => {
                        let uuid = Uuid(uuid);
                        println!("{}", Self::get_by_id(conn.clone(), uuid).await?);
                    }
                    Err(_) => println!("Invalid uuid"),
                },
                None => println!("No uuid supplied"),
            }
        }
        //else if let Some(ValueSource::CommandLine) = _matches.value_source("all")
        else {
            println!(
                "\n{}{}:",
                Self::NAME_PLURAL
                    .chars()
                    .next()
                    .expect("Empty name")
                    .to_uppercase()
                    .collect::<String>(),
                Self::NAME_PLURAL.chars().skip(1).collect::<String>()
            );
            let xs = Self::get_all(conn.clone()).await?;
            for x in xs {
                println!("{}", x);
            }
        }
        Ok(())
    }
}
