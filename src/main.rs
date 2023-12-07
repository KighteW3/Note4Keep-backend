use std::sync::Arc;

use axum::{
    extract::Extension,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use db::models::Notes;

use crate::db::connect::AppState;
use crate::db::{
    connect::{connect_db, database_coll},
    models::User,
};
use futures::stream::TryStreamExt;
use hyper::StatusCode;
use mongodb::{bson::doc, options::FindOptions};
use serde_json::{json, Value};

pub mod db;

type StateExtension = axum::extract::Extension<Arc<AppState>>;

#[debug_handler]
async fn list_users(state: StateExtension) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let user_coll = database_coll::<User>(&state.db, "users").await;

    let find_options = FindOptions::builder().sort(doc! {}).build();

    let mut cursor = if let Ok(cursor) = user_coll.find(None, find_options).await {
        cursor
    } else {
        panic!("Error")
    };

    let mut result = Vec::new();

    while let Some(users) = cursor.try_next().await.unwrap() {
        result.push(users)
    }

    Ok((StatusCode::OK, Json(json!(result))))
}

async fn get_all_notes(state: StateExtension) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let user_coll = database_coll::<Notes>(&state.db, "notes").await;

    let find_options = FindOptions::builder().sort(doc! {}).build();

    let mut cursor = if let Ok(cursor) = user_coll.find(None, find_options).await {
        cursor
    } else {
        panic!("Error")
    };

    let mut result = Vec::new();

    while let Some(notes) = cursor.try_next().await.unwrap() {
        result.push(notes)
    }

    Ok((StatusCode::OK, Json(json!(result))))
}

#[tokio::main]
async fn main() {
    let db = connect_db().await;

    let app_state = AppState { db };

    let app_state = Arc::new(app_state);

    let app = Router::new()
        .route("/api/users", post(list_users))
        .route("/api/notes", post(get_all_notes))
        .layer(Extension(app_state));

    let mut bind_to = String::new();

    let ip = "0.0.0.0";
    let port = "3000";

    bind_to.push_str(ip);
    bind_to.push_str(":");
    bind_to.push_str(port);

    println!("The server is open on {}:{}", ip, port);

    axum::Server::bind(&bind_to.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
