use crate::{app_state::AppState, pattern::analyze_pattern};
use axum::{extract::State, Json};
use chrono::Utc;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Plates {
    pub plates_id: i32,
    pub front_text: String,
    pub plates_type_id: i32,
    pub plates_uri: Option<String>,
    pub is_selling: bool,
    pub is_pin: bool,
    pub total: i32,
    pub add_date: String,
    pub front_number: i32,
    pub back_number: i32,
    pub vehicle_type_id: i32,
    pub users_id: i32,
    pub special_front_id: i32,
    pub province_id: i32,
    pub information: Option<String>,
    pub price: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UniversalId {
    pub id: i32,
}

pub async fn add_new_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> Result<Json<UniversalId>, StatusCode> {
    let unique_text = format!(
        "province_id({})-vehicle_type_id({})-front_number({})-front_text({})-back_number({})",
        payload.province_id,
        payload.vehicle_type_id,
        payload.front_number,
        payload.front_text,
        payload.back_number
    );
    let fetch: Result<Option<(i32,)>, sqlx::Error> =
        sqlx::query_as("SELECT plates_id FROM public.plates WHERE (unique_text = $1)")
            .bind(&unique_text)
            .fetch_optional(&pool)
            .await;
    match fetch {
        Ok(ok) => match ok {
            Some(_) => Err(StatusCode::CONFLICT),
            None => {
                let add_date = Utc::now();
                let insert: Result<(i32,), sqlx::Error> = sqlx::query_as("INSERT INTO public.plates(front_text, province_id, plates_type_id, users_id, total, add_date, unique_text, front_number, back_number, special_front_id, vehicle_type_id, information) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING plates_id")
                    .bind(&payload.front_text)
                    .bind(payload.province_id)
                    .bind(payload.plates_type_id)
                    .bind(payload.users_id)
                    .bind(payload.total)
                    .bind(add_date)
                    .bind(&unique_text)
                    .bind(payload.front_number)
                    .bind(payload.back_number)
                    .bind(payload.special_front_id)
                    .bind(payload.vehicle_type_id)
                    .bind(payload.information)
                    .fetch_one(&pool)
                    .await;
                match insert {
                    Ok((plates_id,)) => {
                        let insert_price = sqlx::query("INSERT INTO public.price_history(plates_id, price, add_date) VALUES ($1, $2, $3)")
                            .bind(plates_id)
                            .bind(payload.price)
                            .bind(add_date)
                            .execute(&pool)
                            .await;
                        match insert_price {
                            Ok(_) => {
                                analyze_pattern(
                                    plates_id,
                                    &payload.front_text,
                                    payload.front_number,
                                    payload.back_number,
                                    add_date,
                                    payload.vehicle_type_id,
                                    &pool,
                                )
                                .await;
                                Ok(Json(UniversalId { id: plates_id }))
                            }
                            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                        }
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn insert_new_price(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let add_date = Utc::now();
    let insert: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as("INSERT INTO public.price_history(plates_id, price, add_date) VALUES ($1, $2, $3) RETURNING plates_id")
        .bind(payload.plates_id)
        .bind(payload.price)
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

pub async fn edit_plates_information(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET information = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.information)
    .bind(payload.plates_id)
    .fetch_optional(&pool)
    .await;
    match update {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn edit_is_selling(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET is_selling = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.is_selling)
    .bind(payload.plates_id)
    .fetch_optional(&pool)
    .await;
    match update {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn edit_is_pin(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET is_pin = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.is_pin)
    .bind(payload.plates_id)
    .fetch_optional(&pool)
    .await;
    match update {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn edit_total(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET total = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.total)
    .bind(payload.plates_id)
    .fetch_optional(&pool)
    .await;
    match update {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn delete_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(UniversalId { id }): Json<UniversalId>,
) -> StatusCode {
    let delete: Result<Option<(i32,)>, sqlx::Error> =
        sqlx::query_as("DELETE FROM public.plates WHERE plates_id = $1 RETURNING plates_id")
            .bind(id)
            .fetch_optional(&pool)
            .await;
    match delete {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn edit_plates_type_id(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    StatusCode::BAD_REQUEST
}

// temporary_plates
pub async fn add_new_temporary_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    StatusCode::BAD_REQUEST
}

pub async fn analyze_new_pattern(
    State(AppState { pool, client: _ }): State<AppState>,
) -> StatusCode {
    let fetch: Result<Vec<(i32, String, i32, i32, i32)>, sqlx::Error> =
        sqlx::query_as("SELECT plates_id, front_text, front_number, back_number, vehicle_type_id FROM public.plates").fetch_all(&pool).await;
    match fetch {
        Ok(ok) => {
            let list = ok.iter();
            let add_date = Utc::now();
            for (plates_id, front_text, front_number, back_number, vehicle_type_id) in list {
                analyze_pattern(
                    *plates_id,
                    front_text,
                    *front_number,
                    *back_number,
                    add_date,
                    *vehicle_type_id,
                    &pool,
                )
                .await;
            }
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
