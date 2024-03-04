use std::env;

use chrono::{prelude::*, Duration, LocalResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub userid: String,
    pub email: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

fn add_to_date(y: i32, m: u32, _d: u32) -> LocalResult<DateTime<Utc>> {
    let current = Utc::now();
    let mut year = current.year();
    let mut month = current.month();
    let mut _day = current.day();

    match y {
        0 => {}
        _ => year += y,
    }

    match m {
        0 => {}
        _ => {
            let mut years_added = 0;
            let mut munits = month + m;

            loop {
                if munits > 12 {
                    munits = munits - 12;
                    years_added += 1;
                } else {
                    break;
                }
            }

            year += years_added;
            month = munits;
        }
    }

    Utc.with_ymd_and_hms(year, month, 5, 0, 0, 0)
}

pub async fn create_jwt(
    user: String,
    id: String,
    email: Option<String>,
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
        exp: add_to_date(0, 3, 0).unwrap().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &token_structure,
        &EncodingKey::from_secret(&secret.as_ref()),
    );

    token
}

pub async fn compare_jwt(
    token: &String,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = env::var("SECRET").unwrap();

    let decoded_token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(&secret.as_ref()),
        &Validation::default(),
    );

    decoded_token
}
