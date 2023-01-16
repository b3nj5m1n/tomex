use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::config::{self, Styleable};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Uuid(pub uuid::Uuid);

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO this is very obviously very bad
        let config = match config::Config::read_config() {
            Ok(config) => config,
            Err(_) => return Err(std::fmt::Error),
        };
        let s = self
            .0
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
            .style(&config.output_uuid.style_content);
        write!(f, "{s}")
    }
}

impl sqlx::Type<sqlx::Sqlite> for Uuid {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&str as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Uuid {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        let s: String = self.0.to_string();
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(
            std::borrow::Cow::Owned(s),
        ));

        sqlx::encode::IsNull::No
    }
}

impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Uuid
where
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&str as sqlx::Decode<DB>>::decode(value)?;
        let id = uuid::Uuid::parse_str(value)?;
        Ok(Self(id))
    }
}
