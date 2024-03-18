use bson::{doc, serde_helpers::chrono_datetime_as_bson_datetime, Bson};
use chrono::prelude::*;
use hyper::StatusCode;
use jsonwebtoken::TokenData;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::task;

use crate::{auth::jwt::Claims, StateExtension};

use super::connect::{database_coll, USERS_OPTIONS};

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
    Newest,
    Latest,
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
        coll_search: Collection<User>,
        user: String,
        state: StateExtension,
    ) -> Result<String, Errors> {
        task::block_in_place(move || {
            Handle::current().block_on(async move {
                let coll = database_coll::<UserOptions>(&state.db, USERS_OPTIONS).await;

                let filters = doc! {"user_id": &user};

                let exists = match coll_search.find_one(filters.clone(), None).await {
                    Ok(res) => match res {
                        Some(res2) => res2,
                        None => {
                            println!("Aqui es problema");
                            return Err(Errors::Status(StatusCode::NOT_FOUND));
                        }
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
                    filter_order: OrderType::Newest,
                    filter_by: FilterType::ByName,
                };

                let filters = doc! {"user": user};

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
        claims: TokenData<Claims>,
        state: StateExtension,
        update: UserOptions,
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

                let data = doc! {"user": exists.user,
                "picture": update.picture,
                "theme": update.theme,
                "filter_order": OrderType::Newest,
                "filter_by": FilterType::ByName};

                match coll.update_one(filters, data, None).await {
                    Ok(_) => Ok(String::from("User options updated succesfully")),
                    Err(e) => {
                        println!("Error: {:?}", e);
                        Err(Errors::Mongo(e))
                    }
                }
            })
        })
    }
}

impl From<OrderType> for Bson {
    fn from(order_type: OrderType) -> Self {
        match order_type {
            OrderType::Newest => Bson::String(String::from("newest")),
            OrderType::Latest => Bson::String(String::from("latest")),
        }
    }
}

impl From<FilterType> for Bson {
    fn from(filter_type: FilterType) -> Self {
        match filter_type {
            FilterType::ByName => Bson::String(String::from("name")),
            FilterType::ByPrio => Bson::String(String::from("priority")),
        }
    }
}
