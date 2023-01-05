#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Uuid(pub uuid::Uuid);

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