use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Authentication {
    pub verification_id: i32,
    pub reference: i32,
    pub code: i32,
    pub email: String,
    pub password: String,
    pub access_token: String,
    pub refresh_token: String,
    pub users_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}
