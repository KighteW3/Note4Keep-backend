use axum::{extract, Json};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{HeaderMap, StatusCode};
use log::error;
use mongodb::{
    bson::{doc, DateTime},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::utils::{get_token::get_token, mongo_health, random_id::random_id};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SomeNote {
    note_phrase: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecificNote {
    note_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteNotes {
    notes_id: Vec<String>,
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
        return Err(StatusCode::BAD_REQUEST);
    };

    let auth = match auth_raw.to_str() {
        Ok(res) => res,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
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

    let notes = match coll.find(None, None).await {
        Ok(mut cursor) => {
            let mut notes = Vec::new();

            while let Some(not) = cursor.try_next().await.unwrap() {
                notes.push(not)
            }

            notes
        }
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if notes.len() < 1 {
        return Err(StatusCode::NO_CONTENT);
    }

    Ok((StatusCode::OK, Json(json!(notes))))
}

#[debug_handler]
pub async fn create_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<CreateNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let req = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let note_id = random_id();

    let claims = if let Ok(claims) = compare_jwt(&token).await {
        claims
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let data = Notes {
        note_id,
        title: req.title,
        priority: req.priority,
        text: req.text,
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
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match coll.insert_one(data, None).await {
        Ok(_ins) => Ok((
            StatusCode::OK,
            Json(json!(doc! {"Response": "Note created succesfully"})),
        )),
        Err(e) => {
            error!("Error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn some_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<SomeNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let headers = headers;
    let req = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    if req.note_phrase.len() < 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let mut passphrase = String::from(".*");
    passphrase.push_str(&req.note_phrase);
    passphrase.push_str(".*");

    let formated = mongodb::bson::Regex {
        pattern: passphrase,
        options: String::new(),
    };

    match mongo_health::mongo_query_error(req.note_phrase) {
        true => return Err(StatusCode::BAD_REQUEST),
        _ => {}
    };

    let filters = doc! {"title": formated, "user": &claims.claims.userid};

    let note = match coll.find(filters, None).await {
        Ok(mut res) => {
            let mut all_results = Vec::new();

            while let Some(note) = match res.try_next().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Error: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            } {
                all_results.push(note)
            }

            if all_results.len() < 1 {
                return Err(StatusCode::NO_CONTENT);
            }

            all_results
        }
        Err(e) => {
            error!("Error: {:?}", e);
            println!("Me gustan las patatas");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(json!(note))))
}

#[debug_handler]
pub async fn spec_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<SpecificNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let headers = headers;
    let req = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let filters = doc! {"note_id": &req.note_id, "user": &claims.claims.userid};

    let note = match coll.find_one(filters, None).await {
        Ok(res) => {
            let note = if let Some(res) = res {
                res
            } else {
                return Err(StatusCode::NOT_FOUND);
            };

            note
        }
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(json!(note))))
}

#[debug_handler]
pub async fn delete_spec_note(
    state: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<SpecificNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let headers = headers;
    let req = payload;
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let filters = doc! {"note_id": &req.note_id, "user": &claims.claims.userid};

    let to_delete = match coll.find_one(filters, None).await {
        Ok(res) => match res {
            Some(result) => result,
            None => return Err(StatusCode::NOT_FOUND),
        },
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let filters2 = doc! {"note_id": to_delete.note_id, "user": &claims.claims.userid};

    let deleted: bool = match coll.delete_one(filters2, None).await {
        Ok(_) => true,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match deleted {
        true => Ok((
            StatusCode::OK,
            Json(json!(doc! {"response": "Succesfully deleted"})),
        )),
        false => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[debug_handler]
pub async fn delete_notes(
    status: StateExtension,
    headers: HeaderMap,
    extract::Json(payload): extract::Json<DeleteNotes>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let headers = headers;
    let req = payload;
    let coll = database_coll::<Notes>(&status.db, NOTES).await;

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let mut to_delete: Vec<String> = Vec::new();
    let mut errored: Vec<String> = Vec::new();

    for note in req.notes_id {
        let filter = doc! {"note_id": &note, "user": &claims.claims.userid};

        match coll.find_one(filter, None).await {
            Ok(res) => match res {
                Some(result) => to_delete.push(result.note_id),
                _ => {
                    errored.push(note);
                    continue;
                }
            },
            Err(e) => {
                println!("Error: {:?}", e);
                continue;
            }
        };
    }

    let mut deleted = Vec::new();
    let mut not_deleted = Vec::new();

    for delete in to_delete {
        let filter = doc! {"note_id": &delete, "user": &claims.claims.userid};

        match coll.delete_one(filter, None).await {
            Ok(_) => deleted.push(delete),
            Err(_) => not_deleted.push(delete),
        }
    }

    match deleted.len() {
        0 => return Err(StatusCode::NOT_FOUND),
        _ => match not_deleted.len() {
            0 => {
                return Ok((
                    StatusCode::OK,
                    Json(json!(
                        doc! {"response": "All notes were succesfully deleted"}
                    )),
                ))
            }

            _ => {
                return Ok((
                    StatusCode::OK,
                    Json(json!(
                        doc! {"response": format!("{:?} notes deleted and {:?} cannot be deleted", deleted.len(), not_deleted.len())}
                    )),
                ))
            }
        },
    }
}
