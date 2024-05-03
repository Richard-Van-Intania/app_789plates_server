use crate::auth::Authentication;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn edit_name(
    Query(params): Query<HashMap<String, String>>,
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    match params.get("name") {
        Some(name) => {
            let update_name = sqlx::query("UPDATE public.users SET name = $1 WHERE users_id = $2")
                .bind(name)
                .bind(payload.users_id)
                .execute(&pool)
                .await;
            match update_name {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        None => StatusCode::BAD_REQUEST,
    }
}

pub async fn edit_information(
    Query(params): Query<HashMap<String, String>>,
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    match params.get("information") {
        Some(information) => {
            let update_information =
                sqlx::query("UPDATE public.users SET information = $1 WHERE users_id = $2")
                    .bind(information)
                    .bind(payload.users_id)
                    .execute(&pool)
                    .await;
            match update_information {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        None => StatusCode::BAD_REQUEST,
    }
}

pub async fn edit_profile_picture() -> impl IntoResponse {}
