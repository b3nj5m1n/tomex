use axum::extract::{Query, State};
use axum::{extract::Path, http::StatusCode, routing::get, Router};
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct TheStateOfAffairs {
    conn: sqlx::SqlitePool,
}

pub async fn start(conn: &sqlx::SqlitePool) {
    let conn = conn.clone();
    let state = Arc::new(TheStateOfAffairs { conn });

    let app = Router::new()
        .route("/api/isbn", get(isbn_query))
        .route("/api/isbn/:isbn", get(isbn))
        .with_state(state);

    let addr = SocketAddr::from((local_ip().expect("Couldn't get local ip address"), 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn isbn(
    Path(isbn): Path<String>,
    State(state): State<Arc<TheStateOfAffairs>>,
) -> Result<String, StatusCode> {
    println!("{}", isbn);
    match isbn.parse::<isbn2::Isbn>() {
        Ok(isbn) => {
            match crate::openlibrary::create_by_isbn(&isbn.to_string(), &state.conn).await {
                Ok(_) => Ok(format!("Handling of {} complete.", isbn)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::IM_A_TEAPOT),
    }
}

async fn isbn_query(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<TheStateOfAffairs>>,
) -> Result<String, StatusCode> {
    let isbn = match params.get("content") {
        Some(isbn) => isbn,
        None => return Err(StatusCode::IM_A_TEAPOT),
    };
    println!("{}", isbn);
    match isbn.parse::<isbn2::Isbn>() {
        Ok(isbn) => {
            match crate::openlibrary::create_by_isbn(&isbn.to_string(), &state.conn).await {
                Ok(_) => Ok(format!("Handling of {} complete.", isbn)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::IM_A_TEAPOT),
    }
}
