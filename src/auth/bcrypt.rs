use bcrypt::{hash, verify};

pub async fn encrypt(pass: &String) -> Result<String, bcrypt::BcryptError> {
    let hashed_pass = hash(&pass, 10);
    match hashed_pass {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn compare(pass: &String, encoded: &String) -> Result<bool, bcrypt::BcryptError> {
    let verified_pass = verify(&pass, &encoded);
    match verified_pass {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
