#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Timestamp(pub Option<chrono::DateTime<chrono::Utc>>);

impl sqlx::Type<sqlx::Sqlite> for Timestamp {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&i8 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for Timestamp {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Int64(match self.0 {
            None => 0_i64,
            Some(ts) => ts.timestamp_millis(),
        }));

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
        if value == 0_i64 {
            return Ok(Self(None));
        }
        let ts = chrono::NaiveDateTime::from_timestamp_millis(value)
            // .filter(|x| *x != chrono::NaiveDateTime::from_timestamp_millis(0).unwrap())
            .map(|x| chrono::DateTime::from_utc(x, chrono::Utc));
        Ok(Self(ts))
    }
}
