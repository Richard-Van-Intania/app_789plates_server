use crate::app_state::AppState;
use axum::{extract::State, Json};
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
    //
    StatusCode::OK
}
