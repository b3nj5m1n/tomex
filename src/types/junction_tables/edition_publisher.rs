use const_format::formatcp;
use sqlx::FromRow;

use crate::{
    traits::*,
    types::{edition::Edition, publisher::Publisher, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow)]
pub struct EditionPublisher {
    pub edition_id: Uuid,
    pub publisher_id: Uuid,
}

impl JunctionTable<Edition, Publisher> for EditionPublisher {
    const TABLE_NAME: &'static str =
        formatcp!("{}_{}", Edition::NAME_SINGULAR, Publisher::NAME_SINGULAR);

    async fn get_id_a(&self) -> &Uuid {
        &self.edition_id
    }

    async fn get_id_b(&self) -> &Uuid {
        &self.publisher_id
    }
}
