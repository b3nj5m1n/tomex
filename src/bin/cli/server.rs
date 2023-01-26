use axum::extract::{Query, State};
use axum::{extract::Path, http::StatusCode, routing::get, Router};
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

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

    let ip = local_ip().expect("Couldn't get local ip address");
    let port = 3000;
    let addr = SocketAddr::from((ip, port));
    info!("Listening on {ip}:{port}.");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn isbn(
    Path(isbn): Path<String>,
    State(state): State<Arc<TheStateOfAffairs>>,
) -> Result<String, StatusCode> {
    info!("Received {}.", isbn);
    match isbn.parse::<isbn2::Isbn>() {
        Ok(isbn) => {
            match crate::openlibrary::create_by_isbn(&isbn.to_string(), &state.conn).await {
                Ok(_) => {
                    info!("Handling of {} complete.", isbn);
                    Ok(format!("Handling of {} complete.", isbn))
                }
                Err(_) => {
                    error!("Handling of {} failed.", isbn);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(_) => {
            error!("{} is not an isbn.", isbn);
            Err(StatusCode::IM_A_TEAPOT)
        }
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
    info!("Received {}.", isbn);
    match isbn.parse::<isbn2::Isbn>() {
        Ok(isbn) => {
            match crate::openlibrary::create_by_isbn(&isbn.to_string(), &state.conn).await {
                Ok(_) => {
                    info!("Handling of {} complete.", isbn);
                    Ok(format!("Handling of {} complete.", isbn))
                }
                Err(_) => {
                    error!("Handling of {} failed.", isbn);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(_) => {
            error!("{} is not an isbn.", isbn);
            Err(StatusCode::IM_A_TEAPOT)
        }
    }
}
