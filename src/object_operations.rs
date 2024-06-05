use crate::{
    app_state::AppState,
    constants::{BUCKET_NAME, COVER_KEY, PLATES_KEY, PROFILE_KEY},
};
use aws_sdk_s3::presigning::PresigningConfig;
use axum::extract::{Query, State};
use hyper::StatusCode;
use std::{collections::HashMap, time::Duration};

pub async fn generate_presigned_url(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool: _, client }): State<AppState>,
) -> Result<String, StatusCode> {
    match params.get("object_key") {
        Some(object_key) => {
            let expires_in = Duration::from_secs(7200);
            match PresigningConfig::expires_in(expires_in) {
                Ok(presigning_config) => {
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
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn update_object(
    Query(params): Query<HashMap<String, String>>,
    State(AppState { pool, client }): State<AppState>,
) -> StatusCode {
    let id = match params.get("id") {
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

    let sql = if object_key.contains(PROFILE_KEY) {
        "SELECT profile_uri FROM public.users WHERE users_id = $1"
    } else if object_key.contains(COVER_KEY) {
        "SELECT cover_uri FROM public.users WHERE users_id = $1"
    } else if object_key.contains(PLATES_KEY) {
        "SELECT plates_uri FROM public.plates WHERE plates_id = $1"
    } else {
        return StatusCode::BAD_REQUEST;
    };
    let fetch: Result<(Option<String>,), sqlx::Error> =
        sqlx::query_as(sql).bind(id).fetch_one(&pool).await;
    let _ = match fetch {
        Ok((ok,)) => match ok {
            Some(key) => match client
                .delete_object()
                .bucket(BUCKET_NAME)
                .key(key)
                .send()
                .await
            {
                Ok(_) => (),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
            },
            None => (),
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if object_key == PROFILE_KEY || object_key == COVER_KEY || object_key == PLATES_KEY {
        let sql = if object_key.contains(PROFILE_KEY) {
            "UPDATE public.users SET profile_uri = null WHERE users_id = $1"
        } else if object_key.contains(COVER_KEY) {
            "UPDATE public.users SET cover_uri = null WHERE users_id = $1"
        } else if object_key.contains(PLATES_KEY) {
            "UPDATE public.plates SET plates_uri = null WHERE plates_id = $1"
        } else {
            return StatusCode::BAD_REQUEST;
        };
        match sqlx::query(sql).bind(id).execute(&pool).await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        let sql = if object_key.contains(PROFILE_KEY) {
            "UPDATE public.users SET profile_uri = $1 WHERE users_id = $2"
        } else if object_key.contains(COVER_KEY) {
            "UPDATE public.users SET cover_uri = $1 WHERE users_id = $2"
        } else if object_key.contains(PLATES_KEY) {
            "UPDATE public.plates SET plates_uri = $1 WHERE plates_id = $2"
        } else {
            return StatusCode::BAD_REQUEST;
        };
        match sqlx::query(sql)
            .bind(object_key)
            .bind(id)
            .execute(&pool)
            .await
        {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
