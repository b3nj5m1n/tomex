use std::fmt::Display;

use crate::traits::QueryType;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Timestamp(pub chrono::DateTime<chrono::Utc>);

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OptionalTimestamp(pub Option<Timestamp>);

impl Display for OptionalTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &match &self.0 {
                Some(ts) => ts.0.to_string(),
                None => "Not specified".to_string(),
            }
        )
    }
}

impl sqlx::Type<sqlx::Sqlite> for OptionalTimestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&i8 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl QueryType for Timestamp {
    fn create_by_prompt(prompt: &str) -> anyhow::Result<Self> {
        Ok(Timestamp(chrono::DateTime::from_utc(
            chrono::NaiveDateTime::new(
                inquire::DateSelect::new(prompt).prompt()?,
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            chrono::Utc,
        )))
    }

    fn create_by_prompt_skippable(prompt: &str) -> anyhow::Result<Option<Self>> {
        Ok(inquire::DateSelect::new(prompt)
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
            .map(|x| Timestamp(x)))
    }
}

// TODO I'm not sure if it makes sense to implement this trait for OptionalTimestamp
/* impl QueryType for OptionalTimestamp {
    fn create_by_prompt(prompt: &str) -> anyhow::Result<Self> {
        Ok(OptionalTimestamp(Some(Timestamp::create_by_prompt(prompt)?)))
    }

    fn create_by_prompt_skippable(prompt: &str) -> anyhow::Result<Option<Self>> {
        Ok(OptionalTimestamp(Timestamp::create_by_prompt_skippable(prompt)?))
    }
} */

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
