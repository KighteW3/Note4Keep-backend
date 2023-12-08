use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use std::{any, collections::BTreeMap};

pub async fn create_jwt(user: String, id: String, email: String, iat: u32) {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"some-secret")?;
    let mut claims: BTreeMap<&str, String> = BTreeMap::new();
    claims.insert("username", &user);
    claims.insert("userid", &id);
    claims.insert("email", &email);
    claims.insert("iat", &iat);
}
