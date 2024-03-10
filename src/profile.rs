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

#[derive(Deserialize)]
pub struct EditInformation {
    pub refresh_token: String,
    pub information: String,
}

pub async fn edit_name(State(pool): State<PgPool>, Json(payload): Json<EditName>) -> StatusCode {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let users_id: i32 = claims.sub.parse().unwrap();
        let update_name = sqlx::query(
            "UPDATE public.users
        SET name = $1
        WHERE users_id = $2",
        )
        .bind(payload.name)
        .bind(users_id)
        .execute(&pool)
        .await;
        match update_name {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::UNAUTHORIZED
    }
}

pub async fn edit_information(
    State(pool): State<PgPool>,
    Json(payload): Json<EditInformation>,
) -> StatusCode {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let users_id: i32 = claims.sub.parse().unwrap();
        let update_information = sqlx::query(
            "UPDATE public.users
        SET information = $1
        WHERE users_id = $2",
        )
        .bind(payload.information)
        .bind(users_id)
        .execute(&pool)
        .await;
        match update_information {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::UNAUTHORIZED
    }
}

pub async fn edit_profile_picture() -> impl IntoResponse {}
