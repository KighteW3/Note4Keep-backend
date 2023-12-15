use axum::{extract, Json};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::StatusCode;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::{
    bcrypt::{compare, encrypt},
    jwt::create_jwt,
};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LogUser {
    username: String,
    password: String,
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

#[debug_handler]
pub async fn create_user(
    state: StateExtension,
    extract::Json(payload): extract::Json<CreateUser>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let user_data = payload;
    let coll = database_coll::<User>(&state.db, USERS).await;
    let filters = doc! {"username": &user_data.username};

    match coll.find_one(filters, None).await {
        Ok(cursor) => match cursor {
            None => {
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

                match coll.insert_one(data, None).await {
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
            Some(_) => Err(StatusCode::CONFLICT),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn log_in(
    state: StateExtension,
    extract::Json(payload): extract::Json<LogUser>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let user_data = payload;
    let coll = database_coll::<User>(&state.db, USERS).await;
    let filters = doc! {"username": &user_data.username};

    match coll.find_one(filters, None).await {
        Ok(cursor) => match cursor {
            Some(res) => match compare(&user_data.password, &res.password).await {
                Ok(authenticated) => {
                    if authenticated {
                        let email = if let Some(e) = res.email {
                            Some(e)
                        } else {
                            None
                        };

                        match create_jwt(user_data.username, res.user_id, email).await {
                            Ok(token) => Ok((
                                StatusCode::OK,
                                Json(json!(doc! {"response": "Login Successful", "token": token})),
                            )),
                            Err(e) => {
                                println!("Error: {:?}", e);
                                Err(StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                    } else {
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            },
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(e) => {
            println!("Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
