use std::fmt::Display;

use inquire::{validator::Validation, CustomUserError};
use liquidity_check::validate;
use serde::{Deserialize, Serialize};

use crate::{
    config::{self, Styleable},
    traits::PromptType,
};

use super::{text::Text, timestamp::OptionalTimestamp};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    pub value:     Text,
    pub timestamp: OptionalTimestamp,
}

impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let s = self
            .value
            .to_string()
            .style(&config.output_price.style_content);
        write!(f, "Purchased for {s}")?;
        if let Some(timestamp) = &self.timestamp.0 {
            write!(f, " {}", timestamp.to_string())?;
        }
        Ok(())
    }
}

// TODO
fn validator(input: &str) -> Result<Validation, CustomUserError> {
    match validate(input) {
        true => Ok(Validation::Valid),
        false => Ok(Validation::Invalid(
            inquire::validator::ErrorMessage::Custom(
                "Not recognised as monetary value".to_string(),
            ),
        )),
    }
}

impl PromptType for Price {
    async fn create_by_prompt(
        _prompt: &str,
        initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        let init_value = initial_value.map(|x| &x.value);
        let init_timestamp = initial_value.map(|x| x.timestamp.0.clone()).flatten();
        let mut prompt =
            inquire::Text::new("How much did you pay for this edition?").with_validator(validator);
        if let Some(s) = init_value {
            prompt = prompt.with_initial_value(&s.0);
        }
        let value = prompt.prompt()?;
        let timestamp = PromptType::create_by_prompt(
            "When did you purchase the edition for this price?",
            init_timestamp.as_ref(),
            conn,
        )
        .await?;
        Ok(Self {
            value:     Text(value),
            timestamp: OptionalTimestamp(Some(timestamp)),
        })
    }

    async fn create_by_prompt_skippable(
        _prompt: &str,
        initial_value: Option<&Self>,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        let init_value = initial_value.map(|x| &x.value);
        let init_timestamp = initial_value.map(|x| x.timestamp.0.clone()).flatten();
        let mut prompt =
            inquire::Text::new("How much did you pay for this edition?").with_validator(validator);
        if let Some(s) = init_value {
            prompt = prompt.with_initial_value(&s.0);
        }
        let value = prompt.prompt_skippable()?;
        if value.is_none() {
            return Ok(None);
        }
        let timestamp = PromptType::create_by_prompt_skippable(
            "When did you purchase the edition for this price?",
            init_timestamp.as_ref(),
            conn,
        )
        .await?;
        Ok(Some(Self {
            value:     Text(value.expect("Unreachable")),
            timestamp: OptionalTimestamp(timestamp),
        }))
    }

    async fn update_by_prompt(&self, prompt: &str, conn: &sqlx::SqlitePool) -> anyhow::Result<Self>
    where
        Self: std::fmt::Display,
    {
        PromptType::create_by_prompt(prompt, Some(self), conn).await
    }

    async fn update_by_prompt_skippable(
        s: &Option<Self>,
        prompt: &str,
        conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>>
    where
        Self: std::fmt::Display,
    {
        PromptType::create_by_prompt_skippable(prompt, s.as_ref(), conn).await
    }
}
