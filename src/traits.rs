use std::fmt::Display;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteQueryResult, SqliteRow},
    FromRow,
};

use crate::config;
use crate::types::{option_to_create::OptionToCreate, uuid::Uuid};

/// A trait which corresponds to a junction table between two other types in the database
pub trait JunctionTable<A, B>
where
    A: CRUD + Eq,
    B: CRUD + Eq,
    Self: Sized + Send + Unpin,
    Self: for<'r> FromRow<'r, SqliteRow>,
{
    /// The name of the junction table in the database
    const TABLE_NAME: &'static str;

    /// Should return the Uuid of the first element
    async fn get_id_a(&self) -> &Uuid;
    /// Should return the Uuid of the second element
    async fn get_id_b(&self) -> &Uuid;

    /// Return all records from the database
    async fn get_all(conn: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let results = sqlx::query_as::<_, Self>(&format!(
            r#"
            SELECT * FROM {table_name_self};
            "#,
            table_name_self = Self::TABLE_NAME,
        ))
        .fetch_all(conn)
        .await?;
        Ok(results)
    }

    /// Create the junction table
    async fn create_table(conn: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table_name_self} (
            	{singular_name_b}_id	INT NOT NULL,
            	{singular_name_a}_id	INT	NOT NULL,
            	FOREIGN KEY ({singular_name_a}_id) REFERENCES {table_name_a} (id),
            	FOREIGN KEY ({singular_name_b}_id) REFERENCES {table_name_b} (id),
            	PRIMARY KEY ({singular_name_a}_id, {singular_name_b}_id)
            );
            "#,
            table_name_self = Self::TABLE_NAME,
            table_name_a = A::TABLE_NAME,
            table_name_b = B::TABLE_NAME,
            singular_name_a = A::NAME_SINGULAR,
            singular_name_b = B::NAME_SINGULAR,
        ))
        .execute(conn)
        .await?;
        Ok(())
    }

    /// Insert a new link between `a` and `b`
    async fn insert(conn: &sqlx::SqlitePool, a: &A, b: &B) -> Result<()> {
        sqlx::query(&format!(
            r#"
            INSERT INTO {table_name_self} 
                ( {singular_name_a}_id, {singular_name_b}_id ) 
                VALUES ( ?1, ?2 );
            "#,
            table_name_self = Self::TABLE_NAME,
            singular_name_a = A::NAME_SINGULAR,
            singular_name_b = B::NAME_SINGULAR,
        ))
        .bind(a.id().await)
        .bind(b.id().await)
        .execute(conn)
        .await?;
        Ok(())
    }

    /// Remove the link between `a` and `b`
    async fn remove(conn: &sqlx::SqlitePool, a: &A, b: &B) -> Result<()> {
        sqlx::query(&format!(
            r#"
            DELETE FROM {table_name_self} 
            WHERE
                {singular_name_a}_id = ?1 AND {singular_name_b}_id = ?2 ;
            "#,
            table_name_self = Self::TABLE_NAME,
            singular_name_a = A::NAME_SINGULAR,
            singular_name_b = B::NAME_SINGULAR,
        ))
        .bind(a.id().await)
        .bind(b.id().await)
        .execute(conn)
        .await?;
        Ok(())
    }

    /// Get all B's that `a` is linked with
    async fn get_all_for_a(conn: &sqlx::SqlitePool, a: &A) -> Result<Vec<B>> {
        let results = sqlx::query_as::<_, Self>(&format!(
            r#"
            SELECT * FROM {table_name_self}
                WHERE {singular_name_a}_id = ?1;
            "#,
            table_name_self = Self::TABLE_NAME,
            singular_name_a = A::NAME_SINGULAR,
        ))
        .bind(a.id().await)
        .fetch_all(conn)
        .await?;

        let mut b_s = vec![];
        for result in results {
            let id = result.get_id_b().await;
            b_s.push(B::get_by_id(conn, id).await?);
        }

        Ok(b_s)
    }
    /// Get all A's that `b` is linked with
    async fn get_all_for_b(conn: &sqlx::SqlitePool, b: &B) -> Result<Vec<A>>
    where
        Self: Sized + Send + Unpin,
        Self: for<'r> FromRow<'r, SqliteRow>,
    {
        let results = sqlx::query_as::<_, Self>(&format!(
            r#"
            SELECT * FROM {table_name_self}
                WHERE {singular_name_b}_id = ?1;
            "#,
            table_name_self = Self::TABLE_NAME,
            singular_name_b = B::NAME_SINGULAR,
        ))
        .bind(b.id().await)
        .fetch_all(conn)
        .await?;

        let mut a_s = vec![];
        for result in results {
            let id = result.get_id_a().await;
            a_s.push(A::get_by_id(conn, id).await?);
        }

        Ok(a_s)
    }

    /// Check if a link between `a` and `b` exists
    async fn exists(conn: &sqlx::SqlitePool, a: &A, b: &B) -> Result<bool> {
        Ok(sqlx::query_as::<_, Self>(&format!(
            r#"
            SELECT 1 FROM {table_name_self}
                WHERE {singular_name_a}_id = ?1
                    AND {singular_name_b}_id = ?2;
            "#,
            table_name_self = Self::TABLE_NAME,
            singular_name_a = A::NAME_SINGULAR,
            singular_name_b = B::NAME_SINGULAR,
        ))
        .bind(a.id().await)
        .bind(b.id().await)
        .fetch_optional(conn)
        .await?
        .is_some())
    }

    /// Given an element `a`, update all links from old to new, removing links that no longer exist and adding new ones
    async fn update(
        conn: &sqlx::SqlitePool,
        a: &A,
        old: &Option<Vec<B>>,
        new: &Option<Vec<B>>,
    ) -> Result<()> {
        // There are no B's in new, remove all existing a <-> B links
        if let None = new {
            let existing = Self::get_all_for_a(conn, a).await?;
            for x in existing {
                Self::remove(conn, a, &x).await?;
            }
        }
        // There were no B's in old, simply add all new ones
        else if let None = old {
            if let Some(b_s) = new {
                for b in b_s {
                    Self::insert(conn, a, b).await?;
                }
            }
        }
        // Merge old and new B's
        else {
            let old = old.as_ref().expect("Unreachable");
            let new = new.as_ref().expect("Unreachable");
            for b in new {
                // If the B didn't exist before, add it
                if !old.contains(b) {
                    Self::insert(conn, a, b).await?;
                }
            }
            for b in old {
                // If the B isn't in new, remove it
                if !new.contains(b) {
                    Self::remove(conn, a, b).await?;
                }
            }
        }
        Ok(())
    }
}

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
    async fn fmt(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        _config: &config::Config,
    ) -> Result<()>;
    /// More verbose than [fmt], displays every piece of information about self
    async fn info_card(
        &self,
        f: &mut String,
        conn: &sqlx::SqlitePool,
        config: &config::Config,
    ) -> Result<()> {
        self.fmt(f, conn, &config).await
    }
    /// Format self and return the result as a string, provide optional string as prefix
    async fn fmt_to_string(
        &self,
        conn: &sqlx::SqlitePool,
        prefix: Option<impl ToString>,
        config: &config::Config,
    ) -> Result<String> {
        let mut buf = if let Some(s) = prefix {
            s.to_string()
        } else {
            String::new()
        };
        self.fmt(&mut buf, conn, config).await?;
        Ok(buf)
    }
}

/// A type which corresponds to a database table and can create it's own table in the database
pub trait CreateTable
where
    Self: Sized,
    Self: Names,
    Self: Insertable,
{
    /// Check if the table currently exists
    async fn table_exists(conn: &sqlx::SqlitePool) -> Result<bool> {
        Ok(sqlx::query(&format!(
            r#"
            SELECT name FROM sqlite_master WHERE type='table' AND name='{}';
            "#,
            Self::TABLE_NAME
        ))
        .fetch_all(conn)
        .await?
        .len()
            != 0)
    }
    /// Initialise table, i.e. create and potentially insert data if the table doesn't already exist
    async fn init_table(conn: &sqlx::SqlitePool) -> Result<()> {
        if !Self::table_exists(conn).await? {
            return Self::create_table(conn).await;
        }
        Ok(())
    }
    /// Create the table and potentially insert data (like default genre names) (will insert duplicate data if the table already exists)
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
    async fn get_by_id(conn: &sqlx::SqlitePool, id: &Uuid) -> Result<Self> {
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
    /// Like `query_by_prompt` or create and insert a new record
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
    /// Like `query_by_prompt` but can be skipped
    async fn query_by_prompt_skippable(conn: &sqlx::SqlitePool) -> Result<Option<Self>> {
        Ok(inquire::Select::new(
            &format!("Select {}:", Self::NAME_SINGULAR),
            Self::get_all(conn).await?,
        )
        .prompt_skippable()?)
    }
    /// Like `query_or_create_by_prompt` but can be skipped
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
    async fn query_by_clap(
        conn: &sqlx::SqlitePool,
        matches: &clap::ArgMatches,
        config: &config::Config,
    ) -> Result<()> {
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("interactive") {
            match Self::query_by_prompt_skippable(conn).await? {
                Some(x) => {
                    println!(
                        "{}",
                        DisplayTerminal::fmt_to_string(&x, conn, Some(" "), config).await?
                    )
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
                                &Self::get_by_id(conn, &uuid).await?,
                                conn,
                                Some(" "),
                                config
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
                println!(
                    "{}",
                    DisplayTerminal::fmt_to_string(&x, conn, Some(" • "), config).await?
                );
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
    async fn update(&mut self, conn: &sqlx::SqlitePool, new: Self) -> Result<SqliteQueryResult>;
    /// Update self by prompting for new values
    async fn update_by_prompt(&mut self, conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable;
    /// Update self by prompting for which record to update and prompting for new values
    async fn update_by_prompt_by_prompt(conn: &sqlx::SqlitePool) -> Result<SqliteQueryResult>
    where
        Self: Queryable,
    {
        Self::query_by_prompt(conn)
            .await?
            .update_by_prompt(conn)
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
