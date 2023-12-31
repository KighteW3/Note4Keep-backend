use axum::Json;
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{HeaderMap, StatusCode};
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::{
    bcrypt::{compare, encrypt},
    jwt::{compare_jwt, create_jwt},
};
use crate::db::connect::USERS;
use crate::utils::{get_token::get_token, random_id::random_id};
use crate::{
    db::{connect::database_coll, models::User},
    StateExtension,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUser {
    username: String,
    email: String,
}

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
pub async fn user_check(
    state: StateExtension,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<CheckUser>(&state.db, USERS).await;

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let filters = doc! {"user_id": &claims.claims.userid};

    let opts = FindOneOptions::builder()
        .projection(doc! { "username": 1, "email": 1 })
        .build();

    match coll.find_one(filters, opts).await {
        Ok(res) => match res {
            Some(user) => {
                //let data = doc! {"username": user.username, "email": user.email};
                Ok((StatusCode::OK, Json(json!(user))))
            }
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(e) => {
            println!("Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn create_user(
    state: StateExtension,
    Json(req): Json<CreateUser>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<User>(&state.db, USERS).await;
    let filters = doc! {"username": &req.username};

    let cursor = match coll.find_one(filters, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match cursor {
        None => {}
        Some(_) => return Err(StatusCode::CONFLICT),
    }

    let encoded_pass = match encrypt(&req.password).await {
        Ok(pass) => pass,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let userid = random_id();

    let email = if let Some(email) = req.email {
        Some(email)
    } else {
        None
    };

    let data = User {
        user_id: userid.clone(),
        username: req.username.clone(),
        password: encoded_pass,
        email: email.clone(),
        ip: None,
    };

    match coll.insert_one(data, None).await {
        Ok(_) => match create_jwt(req.username, userid, email).await {
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

#[debug_handler]
pub async fn log_in(
    state: StateExtension,
    Json(req): Json<LogUser>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<User>(&state.db, USERS).await;
    let filters = doc! {"username": &req.username};

    let cursor = match coll.find_one(filters, None).await {
        Ok(res) => res,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let user_stored = if let Some(res) = cursor {
        res
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    let authenticated = match compare(&req.password, &user_stored.password).await {
        Ok(res) => res,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if authenticated {
        let email = if let Some(e) = user_stored.email {
            Some(e)
        } else {
            None
        };

        match create_jwt(req.username, user_stored.user_id, email).await {
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
