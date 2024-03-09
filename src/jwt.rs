use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

pub const ACCESS_TOKEN_KEY: &'static str = "618C654BBBF31A6D315BA7AB8AB2A";
pub const REFRESH_TOKEN_KEY: &'static str = "D586891172B4BFC6AD15B449DB593";
pub const ISSUER: &'static str = "app789plates";

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

pub async fn verify_signature(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let token = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(ACCESS_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    match token {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
