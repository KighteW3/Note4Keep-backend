use alloc::task;
use bson::{doc, serde_helpers::chrono_datetime_as_bson_datetime};
use chrono::prelude::*;
use hyper::StatusCode;
use jsonwebtoken::TokenData;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use crate::auth::jwt::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notes {
    pub note_id: String,
    pub title: String,
    pub priority: u32,
    pub text: String,
    pub user: String,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub date: DateTime<Utc>,
}

// Pending to decide:
/* #[derive(Debug, Serialize, Deserialize)]
pub struct UserOptions {
    picture: String,
    theme: String,
    filter_by_name: String,
    filter_by_prio: String,
}

pub enum Errors {
    Mongo(mongodb::error::Error),
    NotFound(StatusCode),
}

impl UserOptions {
    pub fn create(
        &self,
        user: &String,
        coll: Collection<UserOptions>,
        claims: TokenData<Claims>,
    ) -> Result<String, Errors> {
        task::block_in_place(move || {
            Handle::current().block_on(async move {
                let coll = &coll;

                let filters = doc! {"user": &user};

                let exists = match coll.find_one(filters, None).await {
                    Ok(res) => match res {
                        Some(res2) => res2,
                        None => return Err(Errors::NotFound(StatusCode::NOT_FOUND)),
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                // const data = UserOptions {}

                let created = match coll.insert_one(data, None).await {
                    Ok(res) => res,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                Ok(String::from("adawd"))
            })
        })
    }
} */
