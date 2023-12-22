use axum::{extract, Json};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{HeaderMap, StatusCode};
use mongodb::{
    bson::{doc, DateTime},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::utils::random_id::random_id;
use crate::{
    auth::jwt::compare_jwt,
    db::{
        connect::{database_coll, NOTES},
        models::Notes,
    },
    StateExtension,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNote {
    title: String,
    priority: u8,
    text: String,
}

#[debug_handler]
pub async fn get_all_notes(state: StateExtension) -> Result<(StatusCode, Json<Value>), StatusCode> {
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

#[debug_handler]
pub async fn get_notes(
    state: StateExtension,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    let auth_raw = if let Some(headers) = headers.get("Authorization") {
        headers
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let auth = if let Ok(a) = auth_raw.to_str() {
        a
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let bearer = auth.to_lowercase().starts_with("bearer");

    if !bearer {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token_raw: String = auth.chars().skip(7).collect();

    let token = token_raw.trim().to_string();

    match compare_jwt(&token).await {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {}
    };

    match coll.find(None, None).await {
        Ok(mut cursor) => {
            let mut notes = Vec::new();

            while let Some(not) = cursor.try_next().await.unwrap() {
                notes.push(not)
            }

            println!("Notes: {:?}", notes);

            Ok((StatusCode::OK, Json(json!(notes))))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn create_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<CreateNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let note_data = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    let headers = if let Some(res) = headers.get("Authorization") {
        match res.to_str() {
            Ok(str) => str,
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let headers_raw = headers;

    if !headers_raw.to_lowercase().starts_with("bearer") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let headers_spaces: String = headers_raw.chars().skip(7).collect();
    let headers_clean = headers_spaces.trim().to_string();

    let note_id = random_id();

    let claims = if let Ok(claims) = compare_jwt(&headers_clean).await {
        claims
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let data = Notes {
        note_id,
        title: note_data.title,
        priority: note_data.priority,
        text: note_data.text,
        user: claims.claims.userid,
        date: DateTime::now(),
    };

    match coll.find_one(doc! {"note_id": &data.note_id}, None).await {
        Ok(e) => match e {
            Some(_) => {
                return Err(StatusCode::CONFLICT);
            }
            None => {}
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match coll.insert_one(data, None).await {
        Ok(_ins) => Ok((
            StatusCode::OK,
            Json(json!(doc! {"Response": "Note created succesfully"})),
        )),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
