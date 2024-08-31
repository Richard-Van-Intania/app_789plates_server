use axum::{extract::State, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;

pub async fn search_plates() {}
pub async fn query_users() {}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatesFilter {
    pub users_id: i32,
    pub search_text: Option<String>,
    pub price_under: i32,
    pub sort_by: String,
    pub plates_type_id_list: Vec<i32>,
    pub province_id_list: Vec<i32>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlatesData {
    pub plates_id: i32,
    pub front_text: String,
    pub plates_type_id: i32,
    pub plates_uri: Option<String>,
    pub total: i32,
    pub front_number: i32,
    pub back_number: i32,
    pub users_id: i32,
    pub special_front_id: i32,
    pub province_id: i32,
    pub information: Option<String>,
    pub price: i32,
    pub name: String,
    pub profile_uri: Option<String>,
    pub liked_plates_id: Option<i32>,
    pub saved_plates_id: Option<i32>,
    pub liked_store_id: Option<i32>,
    pub saved_store_id: Option<i32>,
    pub liked_plates_id_count: i32,
    pub saved_plates_id_count: i32,
    pub reacts_count: i32,
    pub rownumber: i32,
}

pub async fn query_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let order_by: String = match payload.sort_by.as_str() {
        "priceLowToHigh" => "latest_price.price ASC".to_string(),
        "priceHighToLow" => "latest_price.price DESC".to_string(),
        "reacts" => "latest_price.reacts_count DESC".to_string(),
        _ => "latest_price.price ASC".to_string(),
    };

    let sql = format!("");

    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.plates_type_id_list)
        .bind(payload.province_id_list)
        .fetch_all(&pool)
        .await;
    todo!()
}
