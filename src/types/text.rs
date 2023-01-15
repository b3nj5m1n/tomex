use serde::{Deserialize, Serialize};
use std::fmt::Display;

use inquire::validator::StringValidator;

use crate::traits::PromptType;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text(pub String);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl sqlx::Type<sqlx::Sqlite> for Text {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

#[derive(Clone)]
struct ValidatorNonEmpty {}
impl StringValidator for ValidatorNonEmpty {
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        if input.trim().is_empty() {
            return Ok(inquire::validator::Validation::Invalid(
                "Empty string not allowed".into(),
            ));
        }
        Ok(inquire::validator::Validation::Valid)
    }
}

impl PromptType for Text {
    async fn create_by_prompt(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        let mut prompt = inquire::Text::new(prompt).with_validator(ValidatorNonEmpty {});
        if let Some(s) = initial_value {
            prompt = prompt.with_initial_value(&s.0);
        }
        Ok(Text(prompt.prompt()?))
    }

    async fn create_by_prompt_skippable(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        let mut prompt = inquire::Text::new(prompt).with_validator(ValidatorNonEmpty {});
        if let Some(s) = initial_value {
            prompt = prompt.with_initial_value(&s.0);
        }
        match prompt.prompt_skippable()? {
            Some(text) => Ok(Some(Text(text))),
            None => Ok(None),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Text {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(
            self.0.clone().into(),
        ));

        sqlx::encode::IsNull::No
    }
}

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Text
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<DB>>::decode(value)?;
        Ok(Text(value))
    }
}
