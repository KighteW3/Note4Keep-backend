use axum::Json;
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::{HeaderMap, StatusCode};
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::utils::{get_token::get_token, mongo_health::mongo_query_error, random_id::random_id};
use crate::{
    auth::jwt::compare_jwt,
    db::{
        connect::{database_coll, NOTES},
        models::Notes,
    },
    StateExtension,
};
use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct NotesOutgoing {
    pub note_id: String,
    pub title: String,
    pub priority: u32,
    pub text: String,
    pub user: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNote {
    title: String,
    priority: u32,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNote {
    note_id: String,
    title: String,
    priority: u32,
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

    let token = match get_token(&headers) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let claims = match compare_jwt(&token).await {
        Ok(res) => res,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let filter = doc! {"user": &claims.claims.userid};
    let opts = FindOptions::builder().sort(doc! {"date": - 1}).build();

    let notes = match coll.find(filter, opts).await {
        Ok(mut cursor) => {
            let mut notes = Vec::new();

            while let Some(note) = cursor.try_next().await.unwrap() {
                // This thing right down here is a mess, i'll fix some day.
                let note = NotesOutgoing {
                    note_id: note.note_id,
                    title: note.title,
                    priority: note.priority,
                    text: note.text,
                    user: note.user,
                    date: note.date.to_rfc3339(),
                };
                notes.push(note)
            }

            notes
        }
        Err(e) => {
            println!("Error: {:?}", e);
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
    Json(req): Json<CreateNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
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
        date: Utc::now(),
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
        Ok(_) => {}
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!(doc! {"Response": "Note created succesfully"})),
    ))
}

#[debug_handler]
pub async fn some_note(
    state: StateExtension,
    headers: HeaderMap,
    Json(req): Json<SomeNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

    match mongo_query_error(&req.note_phrase) {
        false => {}
        true => return Err(StatusCode::BAD_REQUEST),
    };

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
            println!("Error: {:?}", e);
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

    let filters = doc! {"title": formated, "user": &claims.claims.userid};
    let opts = FindOptions::builder().sort(doc! {"date": -1}).build();

    let note = match coll.find(filters, opts).await {
        Ok(mut res) => {
            let mut all_results = Vec::new();

            while let Some(note) = match res.try_next().await {
                Ok(res) => res,
                Err(e) => {
                    println!("Error: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            } {
                all_results.push(note)
            }

            all_results
        }
        Err(e) => {
            println!("Error: {:?}", e);
            println!("Me gustan las patatas");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if note.len() < 1 {
        return Err(StatusCode::NO_CONTENT);
    }

    Ok((StatusCode::OK, Json(json!(note))))
}

#[debug_handler]
pub async fn spec_note(
    state: StateExtension,
    headers: HeaderMap,
    Json(req): Json<SpecificNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

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
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(json!(note))))
}

#[debug_handler]
pub async fn delete_spec_note(
    state: StateExtension,
    headers: HeaderMap,
    Json(req): Json<SpecificNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

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

    let filters = doc! {"note_id": &req.note_id, "user": &claims.claims.userid};

    match coll.find_one(filters.clone(), None).await {
        Ok(res) => match res {
            Some(_) => {}
            None => return Err(StatusCode::NOT_FOUND),
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let deleted: bool = match coll.delete_one(filters, None).await {
        Ok(_) => true,
        Err(e) => {
            println!("Error: {:?}", e);
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
    Json(req): Json<DeleteNotes>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&status.db, NOTES).await;

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

#[debug_handler]
pub async fn delete_all_notes(
    state: StateExtension,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

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

    let filters = doc! {"user": &claims.claims.userid};

    match coll.delete_many(filters, None).await {
        Ok(_) => {
            return Ok((
                StatusCode::OK,
                Json(json!(doc! {"response": "ALl notes deleted succesfully"})),
            ))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
}

#[debug_handler]
pub async fn update_note(
    state: StateExtension,
    headers: HeaderMap,
    Json(req): Json<UpdateNote>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let coll = database_coll::<Notes>(&state.db, NOTES).await;

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

    let filters = doc! {"note_id": &req.note_id, "user": &claims.claims.userid};
    let mods = doc! {"$set": {"title": &req.title, "priority": &req.priority,
    "text": &req.text}};

    match coll.find_one(filters.clone(), None).await {
        Ok(res) => match res {
            None => return Err(StatusCode::NOT_FOUND),
            _ => {}
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match coll.update_one(filters, mods, None).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!(doc! {"response": "Note updated succesfully"})),
        )),
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
