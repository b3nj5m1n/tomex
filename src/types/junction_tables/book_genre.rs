use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    traits::*,
    types::{book::Book, genre::Genre, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct BookGenre {
    pub book_id:  Uuid,
    pub genre_id: Uuid,
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
