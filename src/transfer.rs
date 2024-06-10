use crate::app_state::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub transfer_plates_id: i32,
    pub plates_id: i32,
    pub users_id: i32,
    pub store_id: i32,
    pub add_date: String,
    pub received: bool,
    pub received_date: String,
}

pub async fn transfer_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Transfer>,
) -> StatusCode {
    let add_date = Utc::now();
    let insert = sqlx::query("INSERT INTO public.transfer_plates(plates_id, users_id, store_id, add_date) VALUES ($1, $2, $3, $4)")
        .bind(payload.plates_id)
        .bind(payload.users_id)
        .bind(payload.store_id)
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

pub async fn accept_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Transfer>,
) -> StatusCode {
    // change user
    // update transfer
    StatusCode::INTERNAL_SERVER_ERROR
}
