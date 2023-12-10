use axum::{extract, Json};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{Request, StatusCode};
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};

use crate::auth::{bcrypt, jwt};
use crate::{
    db::{connect::database_coll, models::User},
    StateExtension,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
    email: Option<String>,
}

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

pub async fn create_user(
    state: StateExtension,
    extract::Json(payload): extract::Json<CreateUser>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    println!("adasd: {:?}", payload);
    let payload2 = payload.password;
    println!("{:?}", payload2);
    Ok((StatusCode::OK, Json(json!({"something": 4}))))
}
