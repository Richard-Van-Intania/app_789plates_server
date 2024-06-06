use crate::app_state::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
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
    pub total: i32,
    pub add_date: String,
    pub unique_text: String,
    pub front_number: i32,
    pub back_number: i32,
    pub special_front_id: Option<i32>,
    pub vehicle_type_id: i32,
    pub price: i32,
}

pub async fn add_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> Result<String, StatusCode> {
    let fetch: Result<Option<(i32,)>, sqlx::Error> =
        sqlx::query_as("SELECT plates_id FROM public.plates WHERE (unique_text = $1)")
            .bind(&payload.unique_text)
            .fetch_optional(&pool)
            .await;
    match fetch {
        Ok(ok) => match ok {
            Some(_) => Err(StatusCode::CONFLICT),
            None => {
                let add_date = Utc::now();
                let insert: Result<(i32,), sqlx::Error> = sqlx::query_as("INSERT INTO public.plates(front_text, province_id, plates_type_id, users_id, total, add_date, unique_text, front_number, back_number, special_front_id, vehicle_type_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING plates_id")
                    .bind(payload.front_text)
                    .bind(payload.province_id)
                    .bind(payload.plates_type_id)
                    .bind(payload.users_id)
                    .bind(payload.total)
                    .bind(add_date)
                    .bind(&payload.unique_text)
                    .bind(payload.front_number)
                    .bind(payload.back_number)
                    .bind(payload.special_front_id)
                    .bind(payload.vehicle_type_id)
                    .fetch_one(&pool)
                    .await;
                match insert {
                    Ok((plates_id,)) => {
                        // add SELECT * FROM public.price_history ORDER BY price_history_id ASC
                        Ok(plates_id.to_string())
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// edit price
// edit photos
// delete
// on/off selling
// on off pin
