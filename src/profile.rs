use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::Deserialize;
use sqlx::PgPool;

use crate::jwt::{Claims, REFRESH_TOKEN_KEY};

#[derive(Deserialize)]
pub struct EditName {
    pub refresh_token: String,
    pub name: String,
}

pub async fn edit_name(
    State(pool): State<PgPool>,
    Json(payload): Json<EditName>,
) -> impl IntoResponse {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let users_id: i32 = claims.sub.parse().unwrap();
        // do things
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}
pub async fn edit_profile_picture() -> impl IntoResponse {}
pub async fn edit_information() -> impl IntoResponse {}
