use const_format::formatcp;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    traits::*,
    types::{edition::Edition, language::Language, uuid::Uuid},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct EditionLanguage {
    pub edition_id: Uuid,
    pub language_id: Uuid,
}

impl JunctionTable<Edition, Language> for EditionLanguage {
    const TABLE_NAME: &'static str =
        formatcp!("{}_{}", Edition::NAME_SINGULAR, Language::NAME_SINGULAR);

    async fn get_id_a(&self) -> &Uuid {
        &self.edition_id
    }

    async fn get_id_b(&self) -> &Uuid {
        &self.language_id
    }
}
