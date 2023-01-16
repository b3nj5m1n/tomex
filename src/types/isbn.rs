use inquire::{validator::Validation, CustomUserError};
use std::fmt::Display;

use crate::{
    config::{self, Styleable},
    traits::PromptType,
};

use super::text::Text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isbn(pub isbn2::Isbn);

impl Isbn {
    pub fn to_text(&self) -> Text {
        Text(self.to_string())
    }
}

fn validator_isbn(input: &str) -> Result<Validation, CustomUserError> {
    match input.parse::<isbn2::Isbn>() {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            inquire::validator::ErrorMessage::Custom("Input isn't a valid isbn".to_string()),
        )),
    }
}

impl PromptType for Isbn {
    async fn create_by_prompt(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        let mut prompt = inquire::Text::new(prompt).with_validator(validator_isbn);
        let initial_value = initial_value.map(|x| x.to_string());
        if let Some(initial_value) = &initial_value {
            prompt = prompt.with_initial_value(initial_value);
        }
        let isbn = prompt
            .prompt()?
            .parse::<isbn2::Isbn>()
            .expect("Unreachable");
        Ok(Self(isbn))
    }

    async fn create_by_prompt_skippable(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        let mut prompt = inquire::Text::new(prompt).with_validator(validator_isbn);
        let initial_value = initial_value.map(|x| x.to_string());
        if let Some(initial_value) = &initial_value {
            prompt = prompt.with_initial_value(initial_value);
        }
        let isbn = prompt
            .prompt_skippable()?
            .map(|x| x.parse::<isbn2::Isbn>().expect("Unreachable"))
            .map(|x| Self(x));
        Ok(isbn)
    }

    async fn update_by_prompt(&self, prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: Display,
    {
        PromptType::create_by_prompt(prompt, Some(self), conn).await
    }

    async fn update_by_prompt_skippable(
        s: &Option<Self>,
        prompt: &str,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>>
    where
        Self: Display,
    {
        PromptType::create_by_prompt_skippable(prompt, s.as_ref(), conn).await
    }
}

impl Display for Isbn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let s = self
            .0
            .hyphenate()
            .expect("Hyphenating isbn failed")
            .style(&config.output_isbn.style_content);
        write!(f, "{s}")
    }
}

impl sqlx::Type<sqlx::Sqlite> for Isbn {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&str as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Isbn {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s: String = self
            .0
            .hyphenate()
            .expect("Hyphenating isbn failed")
            .to_string();
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(
            std::borrow::Cow::Owned(s),
        ));

        sqlx::encode::IsNull::No
    }
}

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Isbn
where
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&str as sqlx::Decode<DB>>::decode(value)?;
        let id = value.parse().expect("Invalid isbn stored in database");
        Ok(Self(id))
    }
}
