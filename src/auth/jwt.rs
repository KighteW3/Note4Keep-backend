use std::env;

use chrono::{prelude::*, Duration};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    username: String,
    userid: String,
    email: String,
    iat: i64,
    exp: i64,
}

pub async fn create_jwt(
    user: String,
    id: String,
    email: String,
) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("SECRET").unwrap();

    let token_structure = Claims {
        username: user,
        userid: id,
        email,
        iat: Utc::now()
            .checked_add_signed(Duration::seconds(60))
            .expect("valid timestamp")
            .timestamp(),
        exp: Utc
            .with_ymd_and_hms(Utc::now().year() + 1, 10, 10, 10, 10, 10)
            .unwrap()
            .timestamp(),
    };

    let token = encode(
        &Header::default(),
        &token_structure,
        &EncodingKey::from_secret(&secret.as_ref()),
    );

    token
}

pub async fn compare_jwt(
    token: String,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = env::var("SECRET").unwrap();

    let decoded_token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(&secret.as_ref()),
        &Validation::default(),
    );

    decoded_token
}
