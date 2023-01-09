use crossterm::style::Stylize;
use std::fmt::Display;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Uuid(pub uuid::Uuid);

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{}", self.0);
        let s = s.with(crossterm::style::Color::Rgb {
            r: 110,
            g: 115,
            b: 141,
        });
        write!(f, "{}", s)
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
        let s: String = self.0.to_string().clone();
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
