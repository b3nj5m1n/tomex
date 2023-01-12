use const_format::formatcp;
use sqlx::FromRow;

use crate::{
    traits::*,
    types::{author::Author, book::Book, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct BookAuthor {
    pub book_id: Uuid,
    pub author_id: Uuid,
}

impl JunctionTable<Book, Author> for BookAuthor {
    const TABLE_NAME: &'static str = formatcp!("{}_{}", Book::NAME_SINGULAR, Author::NAME_SINGULAR);

    async fn get_id_a(&self) -> &Uuid {
        &self.book_id
    }

    async fn get_id_b(&self) -> &Uuid {
        &self.author_id
    }
}
