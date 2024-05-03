use crate::auth::Authentication;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub primary_email: String,
    pub secondary_email: Option<String>,
    pub image_uri: Option<String>,
    pub information: Option<String>,
}

pub async fn fetch_profile(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Profile>, StatusCode> {
    let fetch_profile: Result<
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
        sqlx::Error,
    > = sqlx::query_as("SELECT name, primary_email, secondary_email, image_uri, information FROM public.users WHERE users_id = $1")
        .bind(payload.users_id)
        .fetch_one(&pool)
        .await;
    match fetch_profile {
        Ok((name, primary_email, secondary_email, image_uri, information)) => Ok(Json(Profile {
            name,
            primary_email,
            secondary_email,
            image_uri,
            information,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

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
