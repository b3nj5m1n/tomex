/* #[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use bokhylle::types::*;

    #[test]
    fn builder_book() {
        let uuid = Uuid(uuid::Uuid::new_v4());
        let book = BookBuilder::default()
            .id(uuid.clone())
            .title("Dracula")
            .build()
            .unwrap();
        assert_eq!(
            book,
            Book {
                id: uuid.clone(),
                title: "Dracula".into(),
                author: None,
                release_date: Timestamp(None),
                editions: vec![],
                reviews: vec![],
                genres: vec![],
                deleted: false,
            }
        );
    }
} */
