use std::fmt::Display;

use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, FromRow};

use crate::types::{option_to_create::OptionToCreate, uuid::Uuid};

pub trait QueryType
where
    Self: Sized,
    Self: Display,
    Self: Clone,
{
    /// Prompts the user to create this type, a type has to be returned
    fn create_by_prompt(prompt: &str, _initial_value: Option<&Self>) -> Result<Self>;

    /// Prompts the user to create this type, can be skipped
    fn create_by_prompt_skippable(
        prompt: &str,
        _initial_value: Option<&Self>,
    ) -> Result<Option<Self>>;

    /// Prompts the user to update this type, the result will be the updated type
    fn update_by_prompt(&self, prompt: &str) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        Self::create_by_prompt(&format!("{} (Currently {})", prompt, self), Some(self))
    }

    /// Prompts the user to update this type, can be skipped, if skipped, the result will be the old
    /// value
    fn update_by_prompt_skippable(&self, prompt: &str) -> anyhow::Result<Self> {
        match Self::create_by_prompt_skippable(
            &format!("{} (Currently {})", prompt, self),
            Some(self),
        )? {
            Some(new) => Ok(new),
            None => Ok(self.clone()),
        }
    }

    /// Prompts the user to update or delete this type
    fn update_by_prompt_deleteable(
        &self,
        prompt_delete: &str,
        prompt_update: &str,
    ) -> anyhow::Result<Option<Self>> {
        if !inquire::Confirm::new(prompt_delete)
            .with_default(false)
            .prompt()?
        {
            return Ok(None);
        };
        Ok(Some(Self::update_by_prompt(&self, prompt_update)?))
    }

    /// Prompts the user to update or delete this type, can be skipped, if skipped, the result will
    /// be the old value
    fn update_by_prompt_skippable_deleteable(
        &self,
        prompt_delete: &str,
        prompt_update: &str,
    ) -> anyhow::Result<Option<Self>> {
        if inquire::Confirm::new(prompt_delete)
            .with_default(false)
            .prompt()?
        {
            return Ok(None);
        };
        Ok(Some(Self::update_by_prompt_skippable(
            &self,
            prompt_update,
        )?))
    }
}

pub trait CRUD
where
    Self: Insertable,
    Self: Queryable,
    Self: Updateable,
    Self: Removeable,
{
}

pub trait Updateable
where
    Self: DbTable,
    Self: Sized,
    Self: Id,
{
    async fn update(&self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult>;
    async fn update_by_query(conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable;
    // async fn update_by_clap(conn: &sqlx::SqlitePool, matches: &clap::ArgMatches) -> Result<()>;
}

pub trait DisplayTerminal
where
    Self: Sized,
{
    async fn fmt(&self, f: &mut String, conn: &sqlx::SqlitePool) -> Result<()>;
    async fn fmt_to_string(&self, conn: &sqlx::SqlitePool) -> Result<String> {
        let mut buf = String::new();
        self.fmt(&mut buf, conn).await?;
        Ok(buf)
    }
}

pub trait CreateByPrompt
where
    Self: Sized,
{
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self>;
    async fn create_by_prompt_insert(conn: &sqlx::SqlitePool) -> Result<Self>
    where
        Self: Insertable,
    {
        let x = Self::create_by_prompt(conn).await?;
        if !inquire::Confirm::new("Add to database?")
            .with_default(true)
            .prompt()?
        {
            anyhow::bail!("Aborted");
        };
        x.insert(conn).await?;
        Ok(x)
    }
}

pub trait Insertable
where
    Self: Sized,
{
    async fn insert(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>;
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
pub trait Id {
    async fn id(&self) -> Uuid;
}
pub trait Removeable
where
    Self: DbTable,
    Self: Sized,
    Self: Id,
{
    async fn remove(&self, conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            UPDATE {} SET deleted = 1 WHERE id = ?1"#,
            Self::TABLE_NAME
        ))
        .bind(self.id().await)
        .execute(conn)
        .await?;
        Ok(())
    }
    async fn remove_by_query(conn: &sqlx::SqlitePool) -> Result<()>
    where
        Self: Queryable,
    {
        let x = Self::query_by_prompt_skippable(conn).await?;
        match x {
            Some(x) => {
                if !inquire::Confirm::new(&format!("Are you sure you want to remove {}?", x))
                    .with_default(false)
                    .prompt()?
                {
                    anyhow::bail!("Aborted");
                };
                Self::remove(&x, conn).await?;
                println!("Deleted");
            }
            None => println!("Nothing selected, doing nothing"),
        }
        Ok(())
    }
}

pub trait Queryable
where
    for<'r> Self: FromRow<'r, sqlx::sqlite::SqliteRow>,
    Self: DbTable,
    Self: Sized,
    Self: DisplayTerminal,
    Self: Display,
    Self: Send,
    Self: Unpin,
{
    async fn get_by_id(conn: &sqlx::SqlitePool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE id = ?1 AND deleted = 0;",
            Self::TABLE_NAME
        ))
        .bind(id)
        .fetch_one(conn)
        .await?)
    }
    async fn get_all(conn: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE deleted = 0;",
            Self::TABLE_NAME
        ))
        .fetch_all(conn)
        .await?)
    }
    async fn query_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt()?)
    }
    async fn query_or_create_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self>
    where
        Self: CreateByPrompt,
    {
        let options = OptionToCreate::create_option_to_create(Self::get_all(conn).await?);
        let result =
            inquire::Select::new(&format!("Select {}:", Self::NAME_SINGULAR), options).prompt()?;
        match result {
            OptionToCreate::Value(value) => Ok(value),
            OptionToCreate::Create => Self::create_by_prompt(conn).await,
        }
    }
    async fn query_by_prompt_skippable(conn: &sqlx::SqlitePool) -> Result<Option<Self>> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt_skippable()?)
    }
    async fn query_or_create_by_prompt_skippable(conn: &sqlx::SqlitePool) -> Result<Option<Self>>
    where
        Self: CreateByPrompt,
        Self: Insertable,
    {
        let options = OptionToCreate::create_option_to_create(Self::get_all(conn).await?);
        let result = inquire::Select::new(&format!("Select {}:", Self::NAME_SINGULAR), options)
            .prompt_skippable()?;
        match result {
            Some(result) => match result {
                OptionToCreate::Value(value) => Ok(Some(value)),
                OptionToCreate::Create => {
                    let new = Self::create_by_prompt_insert(conn).await?;
                    return Ok(Some(new));
                }
            },
            None => Ok(None),
        }
    }
    async fn query_by_clap(conn: &sqlx::SqlitePool, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("interactive") {
            match Self::query_by_prompt_skippable(conn).await? {
                Some(x) => {
                    println!("{}", DisplayTerminal::fmt_to_string(&x, conn).await?)
                }
                None => println!("No {} selected.", Self::NAME_SINGULAR),
            }
        }
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("uuid") {
            match matches.get_one::<String>("uuid") {
                Some(uuid_str) => match uuid::Uuid::parse_str(uuid_str) {
                    Ok(uuid) => {
                        let uuid = Uuid(uuid);
                        println!(
                            "{}",
                            DisplayTerminal::fmt_to_string(
                                &Self::get_by_id(conn, uuid).await?,
                                conn
                            )
                            .await?
                        );
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
            let xs = Self::get_all(conn).await?;
            for x in xs {
                println!("{}", DisplayTerminal::fmt_to_string(&x, conn).await?);
            }
        }
        Ok(())
    }
}
