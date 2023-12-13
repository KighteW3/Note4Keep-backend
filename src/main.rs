use std::{env, sync::Arc};

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;

use crate::db::connect::connect_db;
use crate::db::connect::AppState;
use crate::handlers::{
    notes::get_all_notes,
    users::{create_user, list_users, log_in},
};

pub mod auth;
pub mod db;
pub mod handlers;

type StateExtension = axum::extract::Extension<Arc<AppState>>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = connect_db().await;

    let app_state = AppState { db };

    let app_state = Arc::new(app_state);

    let app = Router::new()
        .route("/api/users", post(list_users))
        .route("/api/notes", post(get_all_notes))
        .route("/api/users/create-user", post(create_user))
        .route("/api/users/login", post(log_in))
        .layer(Extension(app_state));

    let mut bind_to = String::new();

    let ip = "0.0.0.0";
    /* let port = if let Ok(res) = env::var("SECRET") {
        res
    } else {
        "3000".to_string()
    }; */
    let port = "3000".to_string();

    bind_to.push_str(ip);
    bind_to.push_str(":");
    bind_to.push_str(&port);

    println!("The server is open on {}:{}", ip, port);

    axum::Server::bind(&bind_to.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
