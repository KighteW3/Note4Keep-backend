use axum::http::HeaderValue;
use axum::{extract, Json};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{Request, StatusCode};
use mongodb::bson::Document;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};

use crate::auth::bcrypt::encrypt;
use crate::auth::jwt::create_jwt;
use crate::auth::{bcrypt, jwt};
use crate::db::connect::USERS;
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
    let user_coll = database_coll::<User>(&state.db, USERS).await;

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
    let user_data = payload;
    let coll = database_coll::<User>(&state.db, USERS).await;
    let filters = doc! {"username": &user_data.username};
    let find_options = FindOptions::builder().sort(doc! {}).build();

    match coll.find(filters, find_options).await {
        Ok(mut cursor) => {
            let mut result = Vec::new();

            while let Some(users) = cursor.try_next().await.unwrap() {
                result.push(users)
            }

            if result.len() > 0 {
                Err(StatusCode::CONFLICT)
            } else {
                let encoded_pass = encrypt(&user_data.password)
                    .await
                    .unwrap_or_else(|_| String::from("awdawds"));

                let userid = "awdwda".to_string();

                let email = if let Some(email) = user_data.email {
                    Some(email)
                } else {
                    None
                };

                let data = User {
                    user_id: userid.clone(),
                    username: user_data.username.clone(),
                    password: encoded_pass,
                    email: email.clone(),
                    ip: None,
                };

                let inserted = coll.insert_one(data, None).await;

                match inserted {
                    Ok(_) => match create_jwt(user_data.username, userid, email).await {
                        Ok(token) => Ok((
                            StatusCode::OK,
                            Json(json!(doc! {"response": "User Created", "token": token})),
                        )),
                        Err(e) => {
                            println!("Error: {:?}", e);

                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        }
        Err(e) => {
            println!("{:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
