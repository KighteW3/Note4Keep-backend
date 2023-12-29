use serde::{Deserialize, Serialize};

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
    pub date: mongodb::bson::DateTime,
}
