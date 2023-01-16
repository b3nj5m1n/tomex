use inquire::{validator::Validation, CustomUserError};

use crate::traits::PromptType;

pub type Rating = u32;

fn validator(input: &str) -> Result<Validation, CustomUserError> {
    match input.parse::<u32>() {
        Ok(n) => {
            if n <= 100 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(
                        "Rating has to be between 0-100".to_string(),
                    ),
                ))
            }
        }
        Err(_) => Ok(Validation::Invalid(
            inquire::validator::ErrorMessage::Custom("Input isn't a valid number".to_string()),
        )),
    }
}

impl PromptType for Rating {
    async fn create_by_prompt(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        let mut prompt = inquire::Text::new(prompt).with_validator(validator);
        let initial_value = initial_value.map(|x| x.to_string());
        if let Some(s) = &initial_value {
            prompt = prompt.with_initial_value(s);
        }
        Ok(prompt.prompt()?.parse::<u32>().expect("Unreachable"))
    }

    async fn create_by_prompt_skippable(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        let mut prompt = inquire::Text::new(prompt).with_validator(validator);
        let initial_value = initial_value.map(|x| x.to_string());
        if let Some(s) = &initial_value {
            prompt = prompt.with_initial_value(s);
        }
        Ok(prompt
            .prompt_skippable()?
            .map(|x| x.parse::<u32>().expect("Unreachable")))
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
