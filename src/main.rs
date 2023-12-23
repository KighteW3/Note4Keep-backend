use std::sync::Arc;

use axum::{extract::Extension, routing::post, Router};
use dotenv::dotenv;

use crate::db::connect::connect_db;
use crate::db::connect::DbState;
use crate::handlers::{
    notes::{create_note, get_all_notes, some_note},
    users::{create_user, list_users, log_in},
};

use std::env;

pub mod auth;
pub mod db;
pub mod handlers;
pub mod utils;

type StateExtension = axum::extract::Extension<Arc<DbState>>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_state = Arc::new(DbState {
        db: connect_db().await,
    });

    let app = Router::new()
        .route("/api/users", post(list_users))
        .route("/api/notes", post(get_all_notes))
        .route("/api/users/create-user", post(create_user))
        .route("/api/users/login", post(log_in))
        .route("/api/notes/create-note", post(create_note))
        .route("/api/notes/some-note", post(some_note))
        .layer(Extension(db_state));

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

    axum::Server::bind(&bind_to.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
