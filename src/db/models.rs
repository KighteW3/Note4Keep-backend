use bson::{doc, serde_helpers::chrono_datetime_as_bson_datetime, Document};
use chrono::prelude::*;
// use hyper::StatusCode;
// use jsonwebtoken::TokenData;
// use mongodb::Collection;
use serde::{Deserialize, Serialize};
// use tokio::runtime::Handle;
// use tokio::task;

// use crate::{auth::jwt::Claims, StateExtension};

// use super::connect::{database_coll, USERS_OPTIONS};

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
#[derive(Debug, Serialize, Deserialize)]
pub struct UserOptions {
    user: String,
    picture: String,
    theme: String,
    filter_order: OrderType,
    filter_by: FilterType,
}

#[derive(Debug, Serialize, Deserialize)]
enum OrderType {
    Ascendant,
    Descendant,
}

#[derive(Debug, Serialize, Deserialize)]
enum FilterType {
    ByName,
    ByPrio,
}

pub enum Errors {
    Mongo(mongodb::error::Error),
    Status(StatusCode),
}

impl UserOptions {
    pub fn create(
        &self,
        coll_search: Collection<User>,
        claims: TokenData<Claims>,
        state: StateExtension,
    ) -> Result<String, Errors> {
        task::block_in_place(move || {
            Handle::current().block_on(async move {
                let coll = database_coll::<UserOptions>(&state.db, USERS_OPTIONS).await;

                let filters = doc! {"user": &claims.claims.userid};

                let exists = match coll_search.find_one(filters.clone(), None).await {
                    Ok(res) => match res {
                        Some(res2) => res2,
                        None => return Err(Errors::Status(StatusCode::NOT_FOUND)),
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                let data = UserOptions {
                    user: exists.user_id,
                    picture: String::from("default.jpg"),
                    theme: String::from("default"),
                    filter_order: OrderType::Ascendant,
                    filter_by: FilterType::ByName,
                };

                match coll.find_one(filters, None).await {
                    Ok(res) => match res {
                        None => {}
                        Some(_) => return Err(Errors::Status(StatusCode::CONFLICT)),
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                match coll.insert_one(data, None).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                Ok(String::from("User options created with no trouble."))
            })
        })
    }

    pub fn update(
        &self,
        claims: TokenData<Claims>,
        state: StateExtension,
        _update: Document,
    ) -> Result<String, Errors> {
        task::block_in_place(move || {
            Handle::current().block_on(async move {
                let coll = database_coll::<UserOptions>(&state.db, USERS_OPTIONS).await;

                let filters = doc! {"user": claims.claims.userid};

                let exists = match coll.find_one(filters.clone(), None).await {
                    Ok(res) => match res {
                        Some(options) => options,
                        None => return Err(Errors::Status(StatusCode::NOT_FOUND)),
                    },
                    Err(e) => {
                        println!("Error {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                let _data = UserOptions {
                    user: exists.user.clone(),
                    picture: String::from("default.jpg"),
                    theme: String::from("default.jpg"),
                    filter_by: FilterType::ByName,
                    filter_order: OrderType::Ascendant,
                };

                let data = doc! {"user": exists.user, "picture":
                String::from("default.jpg"),
                "theme": String::from("default"),
                "filter_order": 0, "filter_by": 0};

                let _options = match coll.update_one(filters, data, None).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Errors::Mongo(e));
                    }
                };

                Ok(String::from("Not functional yet"))
            })
        })
    }
}

/* impl Borrow<T> for UserOptions {
    fn borrow() {}
} */
