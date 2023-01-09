use std::fmt::Display;

use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, FromRow};

use crate::types::{option_to_create::OptionToCreate, uuid::Uuid};

/// A (more or less primitive) type which can be created and updated by command line prompts
/// This is for things like Text, Timestamps, etc., while [Insertable] is for structs
/// corresponding to a database table. A struct which implements [Insertable] should be made up
/// of types which implement [PromptType]
pub trait PromptType
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

/// An alternative to std::fmt::Display which can query the database to retrieve addtional
/// information
pub trait DisplayTerminal
where
    Self: Sized,
{
    /// Format self to the provided string buffer
    async fn fmt(&self, f: &mut String, conn: &sqlx::SqlitePool) -> Result<()>;
    /// Format self and return the result as a string
    async fn fmt_to_string(&self, conn: &sqlx::SqlitePool) -> Result<String> {
        let mut buf = String::new();
        self.fmt(&mut buf, conn).await?;
        Ok(buf)
    }
}

/// A type which corresponds to a database table and can create it's own table in the database
pub trait CreateTable
where
    Self: Sized,
    Self: Names,
{
    /// Creates the table (if it doesn't already exist)
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()>;
}

/// Singular and plural names for type & name of table in database, for example:
/// ```
/// const NAME_SINGULAR = "book";
/// const NAME_PLURAL = "books";
/// const TABLE_NAME = "books";
/// ```
pub trait Names
where
    Self: Sized,
{
    const NAME_SINGULAR: &'static str;
    const NAME_PLURAL: &'static str;
    const TABLE_NAME: &'static str = Self::NAME_PLURAL;
}

/// A type which can be uniquely identified by an id
pub trait Id {
    /// Return the unique identifier for self
    async fn id(&self) -> Uuid;
}

/// A type which corresponds to a database table entry and can be inserted, queried, updated and removed
pub trait CRUD
where
    Self: Insertable,
    Self: Queryable,
    Self: Updateable,
    Self: Removeable,
{
}

/// A type which corresponds to a database table entry and can be inserted
pub trait Insertable
where
    Self: Sized,
{
    /// Insert self into database
    async fn insert(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>;
    /// Create self by prompts
    async fn create_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self>;
    /// Create self by prompts and insert
    async fn insert_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self>
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

/// A type which corresponds to a database table entry and can be queried
pub trait Queryable
where
    for<'r> Self: FromRow<'r, sqlx::sqlite::SqliteRow>,
    Self: Names,
    Self: Sized,
    Self: DisplayTerminal,
    Self: Display,
    Self: Send,
    Self: Unpin,
{
    /// Return record with id from database
    async fn get_by_id(conn: &sqlx::SqlitePool, id: Uuid) -> Result<Self> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE id = ?1 AND deleted = 0;",
            Self::TABLE_NAME
        ))
        .bind(id)
        .fetch_one(conn)
        .await?)
    }
    /// Get all records from this database
    async fn get_all(conn: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM {} WHERE deleted = 0;",
            Self::TABLE_NAME
        ))
        .fetch_all(conn)
        .await?)
    }
    /// Select a record by a prompt from a list of all records
    async fn query_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt()?)
    }
    /// Like [query_by_prompt] or create and insert a new record
    async fn query_or_create_by_prompt(conn: &sqlx::SqlitePool) -> Result<Self>
    where
        Self: Insertable,
    {
        let options = OptionToCreate::create_option_to_create(Self::get_all(conn).await?);
        let result =
            inquire::Select::new(&format!("Select {}:", Self::NAME_SINGULAR), options).prompt()?;
        match result {
            OptionToCreate::Value(value) => Ok(value),
            OptionToCreate::Create => Self::create_by_prompt(conn).await,
        }
    }
    /// Like [query_by_prompt] but can be skipped
    async fn query_by_prompt_skippable(conn: &sqlx::SqlitePool) -> Result<Option<Self>> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt_skippable()?)
    }
    /// Like [query_or_create_by_prompt] but can be skipped
    async fn query_or_create_by_prompt_skippable(conn: &sqlx::SqlitePool) -> Result<Option<Self>>
    where
        Self: Insertable,
    {
        let options = OptionToCreate::create_option_to_create(Self::get_all(conn).await?);
        let result = inquire::Select::new(&format!("Select {}:", Self::NAME_SINGULAR), options)
            .prompt_skippable()?;
        match result {
            Some(result) => match result {
                OptionToCreate::Value(value) => Ok(Some(value)),
                OptionToCreate::Create => {
                    let new = Self::insert_by_prompt(conn).await?;
                    return Ok(Some(new));
                }
            },
            None => Ok(None),
        }
    }
    /// Select a single record from the database by parsing [clap] matches
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

/// A type which corresponds to a database table entry and can be updated
pub trait Updateable
where
    Self: Names,
    Self: Sized,
    Self: Id,
{
    /// Update self to new values in `new`
    async fn update(&self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult>;
    /// Update self by prompting for new values
    async fn update_by_query(&self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable;
    /// Update self by prompting for which record to update and prompting for new values
    async fn update_by_query_by_query(conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable,
    {
        Self::query_by_prompt(conn)
            .await?
            .update_by_query(conn)
            .await
    }
    // async fn update_by_clap(conn: &sqlx::SqlitePool, matches: &clap::ArgMatches) -> Result<()>;
}

/// A type which corresponds to a database table entry and can be removed
pub trait Removeable
where
    Self: Names,
    Self: Sized,
    Self: Id,
{
    /// Remove self from database
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
    /// Prompt for which record to remove from the database
    async fn remove_by_prompt(conn: &sqlx::SqlitePool) -> Result<()>
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
