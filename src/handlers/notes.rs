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
pub async fn create_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<CreateNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let note_data = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    match headers.get("Authorization") {
        Some(res) => {
            let headers_raw = res;
            match headers_raw.to_str() {
                Ok(res) => {
                    let bearer = res.to_lowercase().starts_with("bearer");
                    if !bearer {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    let headers_spaces: String = res.chars().skip(7).collect();
                    let headers_clean = headers_spaces.trim().to_string();

                    match compare_jwt(&headers_clean).await {
                        Ok(claims) => {
                            let data = Notes {
                                note_id: "adawed".to_string(),
                                title: note_data.title,
                                priority: note_data.priority,
                                text: note_data.text,
                                user: claims.claims.userid,
                                date: DateTime::now(),
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
                        Err(_) => Err(StatusCode::UNAUTHORIZED),
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => Err(StatusCode::BAD_REQUEST),
    }
}
