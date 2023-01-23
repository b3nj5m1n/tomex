use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crossterm::style::Stylize;

use crate::{
    config::{self, Styleable},
    default_colors::COLOR_DIMMED,
    traits::PromptType,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub chrono::DateTime<chrono::Utc>);

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use chrono_humanize::{Accuracy, HumanTime, Tense};
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let ht = HumanTime::from(self.0);
        let s = ht.to_text_en(Accuracy::Rough, Tense::Past);
        let s = s.style(&config.output_timestamp.style_content);
        write!(f, "{s}")
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalTimestamp(pub Option<Timestamp>);

impl Display for OptionalTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &match &self.0 {
                Some(ts) => ts.0.to_string(),
                None => "Not specified".with(COLOR_DIMMED).to_string(),
            }
        )
    }
}

impl sqlx::Type<sqlx::Sqlite> for Timestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&i8 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}
impl sqlx::Type<sqlx::Sqlite> for OptionalTimestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&i8 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl PromptType for Timestamp {
    async fn create_by_prompt(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Self> {
        const OPTION_DATEPICKER: &'static str = "Datepicker";
        const OPTION_TIMESTAMP: &'static str = "Timestamp";
        let options: Vec<&str> = vec![OPTION_DATEPICKER, OPTION_TIMESTAMP];

        let ans: Result<&str, inquire::InquireError> =
            inquire::Select::new("How would you like to input the timestamp?", options).prompt();

        match ans {
            Ok(OPTION_DATEPICKER) => {
                let mut prompt = inquire::DateSelect::new(prompt);
                if let Some(s) = initial_value {
                    prompt = inquire::DateSelect {
                        starting_date: s.0.date_naive(),
                        ..prompt
                    };
                }
                Ok(Timestamp(chrono::DateTime::from_utc(
                    chrono::NaiveDateTime::new(
                        prompt.prompt()?,
                        chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                    ),
                    chrono::Utc,
                )))
            }
            Ok(OPTION_TIMESTAMP) => {
                fn prompt_for_timestamp() -> DateTime<Utc> {
                    loop {
                        let prompt = match inquire::Text::new("Timestamp:").prompt() {
                            Ok(ts) => ts,
                            Err(_) => continue,
                        };
                        let timestamp = match dateparser::parse(prompt.as_str()) {
                            Ok(ts) => ts,
                            Err(_) => continue,
                        };
                        let result =
                            inquire::Confirm::new(&format!("Is this correct: {} ?", timestamp))
                                .with_default(true)
                                .prompt();
                        match result {
                            Ok(true) => return timestamp,
                            Ok(false) => continue,
                            Err(_) => continue,
                        }
                    }
                }
                let timestamp = prompt_for_timestamp();
                Ok(Timestamp(timestamp))
            }
            Ok(_) => unreachable!("Unexpected response"),
            Err(_) => Err(anyhow::anyhow!("Error getting progress")),
        }
    }

    async fn create_by_prompt_skippable(
        prompt: &str,
        initial_value: Option<&Self>,
        _conn: &sqlx::SqlitePool,
    ) -> anyhow::Result<Option<Self>> {
        let mut prompt = inquire::DateSelect::new(prompt);
        if let Some(s) = initial_value {
            prompt = inquire::DateSelect {
                starting_date: s.0.date_naive(),
                ..prompt
            };
        }
        Ok(prompt
            .prompt_skippable()?
            .map(|x| {
                chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::new(
                        x,
                        chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    ),
                    chrono::Utc,
                )
            })
            .map(Timestamp))
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

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Timestamp {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Int64(
            self.0.timestamp_millis(),
        ));

        sqlx::encode::IsNull::No
    }
}
impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for OptionalTimestamp {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Int64(
            match self.0.clone() {
                None => 0_i64,
                Some(ts) => ts.0.timestamp_millis(),
            },
        ));

        sqlx::encode::IsNull::No
    }
}

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Timestamp
where
    i64: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i64 as sqlx::Decode<DB>>::decode(value)?;
        let ts = chrono::NaiveDateTime::from_timestamp_millis(value)
            .map(|x| chrono::DateTime::from_utc(x, chrono::Utc));
        match ts {
            Some(ts) => Ok(Timestamp(ts)),
            None => Err(Box::new(sqlx::Error::Protocol(
                "Couldn't decode non-optional timestamp".to_string(),
            ))),
        }
    }
}
impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for OptionalTimestamp
where
    i64: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i64 as sqlx::Decode<DB>>::decode(value)?;
        if value == 0_i64 {
            return Ok(Self(None));
        }
        let ts = chrono::NaiveDateTime::from_timestamp_millis(value)
            // .filter(|x| *x != chrono::NaiveDateTime::from_timestamp_millis(0).unwrap())
            .map(|x| chrono::DateTime::from_utc(x, chrono::Utc));
        match ts {
            Some(ts) => Ok(Self(Some(Timestamp(ts)))),
            None => Ok(Self(None)),
        }
    }
}
