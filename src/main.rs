use std::{sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    routing::{delete, get, patch, post},
    Router,
};
use dotenv::dotenv;
use hyper::StatusCode;

use crate::db::connect::connect_db;
use crate::db::connect::DbState;
use crate::handlers::{
    notes::{
        create_note, delete_all_notes, delete_notes, delete_spec_note, get_notes, some_note,
        spec_note, update_note,
    },
    users::{create_user, list_users, log_in, user_check},
};
use crate::utils::check_integrity::check_integrity;
use tower::{
    timeout::{error, Timeout, TimeoutLayer},
    BoxError, ServiceBuilder,
};
use tower_http::cors::{Any, CorsLayer};

use std::env;

pub mod auth;
pub mod db;
pub mod handlers;
pub mod utils;

type StateExtension = axum::extract::Extension<Arc<DbState>>;

pub async fn handle_timeout_error(err: BoxError) -> StatusCode {
    if err.is::<error::Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    check_integrity();

    let db_state = Arc::new(DbState {
        db: connect_db().await,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let timeout_middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_timeout_error))
        .layer(TimeoutLayer::new(Duration::from_secs(60)));

    let app = Router::new()
        .route("/api/users", get(list_users))
        .route("/api/users/check", post(user_check))
        .route("/api/notes", post(get_notes))
        .route("/api/users/create-user", post(create_user))
        .route("/api/users/login", post(log_in))
        .route("/api/notes/create-note", post(create_note))
        .route("/api/notes/some-note", post(some_note))
        .route("/api/notes/spec-note", post(spec_note))
        .route("/api/notes/delete-spec-note", delete(delete_spec_note))
        .route("/api/notes/delete-notes", delete(delete_notes))
        .route("/api/notes/delete-all-notes", delete(delete_all_notes))
        .route("/api/notes/update-note", patch(update_note))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(db_state))
                .layer(timeout_middleware)
                .layer(cors),
        );

    let mut bind_to = String::new();

    let ip = "0.0.0.0";

    let port = if let Ok(res) = env::var("PORT") {
        res
    } else {
        "3000".to_string()
    };

    bind_to.push_str(ip);
    bind_to.push_str(":");
    bind_to.push_str(&port);

    println!("The server is open on {}:{}", ip, port);

    let listener = tokio::net::TcpListener::bind(bind_to).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
