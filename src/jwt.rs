use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use crate::authentication::{Authentication, NULL_ALIAS_INT, NULL_ALIAS_STRING};

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

pub async fn renew_token(
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let date = Utc::now();
        let access_claims = Claims {
            iat: date.timestamp() as usize,
            exp: (date + Duration::minutes(60)).timestamp() as usize,
            iss: ISSUER.to_string(),
            sub: claims.sub.to_string(),
        };
        let refresh_claims = Claims {
            iat: date.timestamp() as usize,
            exp: (date + Duration::days(14)).timestamp() as usize,
            iss: ISSUER.to_string(),
            sub: claims.sub.to_string(),
        };
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(ACCESS_TOKEN_KEY.as_ref()),
        );
        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        );
        Ok(Json(Authentication {
            verification_id: NULL_ALIAS_INT,
            reference: NULL_ALIAS_INT,
            code: NULL_ALIAS_INT,
            email: NULL_ALIAS_STRING.to_string(),
            secondary_email: NULL_ALIAS_STRING.to_string(),
            password: NULL_ALIAS_STRING.to_string(),
            access_token: access_token.unwrap(),
            refresh_token: refresh_token.unwrap(),
            users_id: NULL_ALIAS_INT,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
