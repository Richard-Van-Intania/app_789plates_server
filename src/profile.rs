use crate::{app_state::AppState, auth::Authentication, constants::BUCKET_NAME};
use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub email: String,
    pub profile_uri: Option<String>,
    pub cover_uri: Option<String>,
    pub information: Option<String>,
}

pub async fn fetch_profile(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Profile>, StatusCode> {
    let fetch: Result<
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
        sqlx::Error,
    > = sqlx::query_as("SELECT name, email, profile_uri, cover_uri, information FROM public.users WHERE users_id = $1")
        .bind(payload.users_id)
        .fetch_one(&pool)
        .await;
    match fetch {
        Ok((name, email, profile_uri, cover_uri, information)) => Ok(Json(Profile {
            name,
            email,
            profile_uri,
            cover_uri,
            information,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn edit_name(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    match params.get("name") {
        Some(name) => {
            let update = sqlx::query("UPDATE public.users SET name = $1 WHERE users_id = $2")
                .bind(name)
                .bind(payload.users_id)
                .execute(&pool)
                .await;
            match update {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        None => StatusCode::BAD_REQUEST,
    }
}

pub async fn edit_information(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    match params.get("information") {
        Some(information) => {
            let update =
                sqlx::query("UPDATE public.users SET information = $1 WHERE users_id = $2")
                    .bind(information)
                    .bind(payload.users_id)
                    .execute(&pool)
                    .await;
            match update {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        None => StatusCode::BAD_REQUEST,
    }
}

pub async fn edit_profile_photo(
    State(AppState { pool, client }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> impl IntoResponse {
    let expires_in = Duration::from_secs(7200);
    let presigned_request = client
        .put_object()
        .bucket(BUCKET_NAME)
        .key("plates/plates-20d0da2a-254f-41ff-8d2c-4e477a7c4ee5.jpg")
        .presigned(PresigningConfig::expires_in(expires_in).unwrap())
        .await
        .unwrap();

    presigned_request.uri().to_string()
}
pub async fn edit_cover_photo() -> impl IntoResponse {}
