use chrono::prelude::*;
use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

struct Claims {
    username: String,
    user_id: String,
    email: String,
    iat: DateTime<Utc>,
}

pub async fn create_jwt(user: String, id: String, email: String) {
    let token_structure = Claims {
        username: user,
        user_id: id,
        email: email,
        iat: Utc::now(),
    };
}
