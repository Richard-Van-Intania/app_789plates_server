use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

pub const ACCESS_TOKEN_KEY: &'static str = "618C654BBBF31A6D315BA7AB8AB2A";
pub const REFRESH_TOKEN_KEY: &'static str = "D586891172B4BFC6AD15B449DB593";
pub const ISSUER: &'static str = "app789plates";
