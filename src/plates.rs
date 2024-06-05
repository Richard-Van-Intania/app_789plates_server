use crate::app_state::AppState;
use axum::{extract::State, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Plates {
    pub plates_id: i32,
    pub front_text: String,
    pub province_id: i32,
    pub plates_type_id: i32,
    pub users_id: i32,
    pub plates_uri: Option<String>,
    pub is_selling: bool,
    pub is_pin: bool,
    pub sum: i32,
    pub add_date: String,
    pub unique_text: String,
    pub front_number: i32,
    pub number: i32,
    pub special_front_id: Option<i32>,
    pub vehicle_type_id: i32,
}

pub async fn add_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> Result<Json<Plates>, StatusCode> {
    Err(StatusCode::BAD_REQUEST)
}
