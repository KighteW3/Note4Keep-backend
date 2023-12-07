use axum::Json;
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::StatusCode;
use mongodb::{bson::doc, options::FindOptions};
use serde_json::{json, Value};

use crate::{
    db::{connect::database_coll, models::User},
    StateExtension,
};

#[debug_handler]
pub async fn list_users(state: StateExtension) -> Result<(StatusCode, Json<Value>), StatusCode> {
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
