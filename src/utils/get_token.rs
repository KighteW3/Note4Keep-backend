use hyper::{HeaderMap, StatusCode};

pub fn get_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    match headers.get("Authorization") {
        Some(res) => {
            let authorization = match res.to_str() {
                Ok(res) => res,
                Err(e) => {
                    println!("Error: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let bearer = authorization.to_lowercase().starts_with("bearer");

            if !bearer {
                return Err(StatusCode::BAD_REQUEST);
            }

            let mut token: String = authorization.chars().skip(7).collect();

            token = token.trim().to_string();

            Ok(token)
        }
        None => return Err(StatusCode::BAD_REQUEST),
    }
}
