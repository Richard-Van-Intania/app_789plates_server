use crate::{
    app_state::AppState,
    auth::Authentication,
    aws_operations::{generate_presigned_url, remove_object},
};
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::BUCKET_NAME;
use aws_sdk_s3::{presigning::PresigningConfig, Client, Error};
use std::time::Duration;

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
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool: _, client }): State<AppState>,
) -> impl IntoResponse {
    let result = remove_object(&client, params.get("object_key").unwrap().to_string()).await;
    match result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn update_cover_photo() -> impl IntoResponse {}

pub async fn update_profile_photo(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool, client }): State<AppState>,
    body: Bytes,
) -> StatusCode {
    let users_id = match params.get("users_id") {
        Some(some) => match some.parse::<i32>() {
            Ok(ok) => ok,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => return StatusCode::BAD_REQUEST,
    };
    let object_key = match params.get("object_key") {
        Some(some) => some.to_string(),
        None => return StatusCode::BAD_REQUEST,
    };
    let result = generate_presigned_url(&client, object_key.to_string()).await;
    let url = match result {
        Ok(ok) => ok,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let cln = reqwest::Client::new();
    // let res = cln.post(url).body(body).send().await;
    // cannot use body

    //
    StatusCode::OK
}

pub async fn create_presigned_url(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool: _, client }): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    match params.get("object_key") {
        Some(object_key) => {
            let expires_in = Duration::from_secs(7200);
            let presigning_config = match PresigningConfig::expires_in(expires_in) {
                Ok(ok) => ok,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };
            let presigned_request = client
                .put_object()
                .bucket(BUCKET_NAME)
                .key(object_key)
                .presigned(presigning_config)
                .await;
            match presigned_request {
                Ok(ok) => Ok(ok.uri().to_string()),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::BAD_REQUEST),
    }
}
