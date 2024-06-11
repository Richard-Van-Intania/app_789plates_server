use crate::app_state::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hashtag {
    pub hashtag_id: i32,
    pub tag: String,
    pub add_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatesHashtag {
    pub plates_hashtag_id: i32,
    pub plates_id: i32,
    pub hashtag_id: i32,
    pub add_date: String,
}

pub async fn add_new_hashtag(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Hashtag>,
) -> StatusCode {
    let add_date = Utc::now();
    let insert = sqlx::query("INSERT INTO public.hashtag(tag, add_date) VALUES ($1, $2)")
        .bind(payload.tag)
        .bind(add_date)
        .fetch_optional(&pool)
        .await;
    match insert {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn add_hashtag_to_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesHashtag>,
) -> StatusCode {
    let unique_text = format!(
        "plates_id({})-hashtag_id({})",
        payload.plates_id, payload.hashtag_id
    );
    let fetch =
        sqlx::query("SELECT plates_hashtag_id FROM public.plates_hashtag WHERE (unique_text = $1)")
            .bind(&unique_text)
            .fetch_optional(&pool)
            .await;
    match fetch {
        Ok(ok) => match ok {
            Some(_) => StatusCode::BAD_REQUEST,
            None => {
                let add_date = Utc::now();
                let insert = sqlx::query("INSERT INTO public.plates_hashtag(plates_id, hashtag_id, add_date, unique_text) VALUES ($1, $2, $3, $4)")
                    .bind(payload.plates_id)
                    .bind(payload.hashtag_id)
                    .bind(add_date)
                    .bind(&unique_text)
                    .fetch_optional(&pool)
                    .await;
                match insert {
                    Ok(ok) => match ok {
                        Some(_) => StatusCode::OK,
                        None => StatusCode::BAD_REQUEST,
                    },
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
