use const_format::formatcp;
use sqlx::FromRow;

use crate::{traits::*, types::uuid::Uuid};

use super::{book::Book, genre::Genre};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct BookGenre {
    pub book_id: Uuid,
    pub genre_id: Uuid,
    pub deleted: bool,
}

impl JunctionTable<Book, Genre> for BookGenre {
    const TABLE_NAME: &'static str = formatcp!("{}_{}", Book::NAME_SINGULAR, Genre::NAME_SINGULAR);

    async fn get_id_a(&self) -> &Uuid {
        &self.book_id
    }

    async fn get_id_b(&self) -> &Uuid {
        &self.genre_id
    }
}
