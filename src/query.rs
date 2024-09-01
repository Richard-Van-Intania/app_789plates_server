use crate::app_state::AppState;
use axum::{extract::State, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatesFilter {
    pub users_id: i32,
    pub pattern: String,
    pub plates_type_id: i32,
    pub province_id: i32,
    pub search_text: String,
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
    pub liked_plates_id_count: i64,
    pub saved_plates_id_count: i64,
    pub reacts_count: i64,
    pub rownumber: i64,
}

pub async fn query_special_front(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = match payload.sort_by.as_str() {
        "priceLowToHigh" => "latest_price.price ASC",
        "priceHighToLow" => "latest_price.price DESC",
        "reacts" => "latest_price.reacts_count DESC",
        _ => "latest_price.price ASC",
    };
    let sql = format!(
        "WITH latest_price AS (
    SELECT price_history.price_history_id,
        price_history.plates_id,
        price_history.price,
        COUNT(lp.liked_plates_id) AS liked_plates_id_count,
        COUNT(sp.saved_plates_id) AS saved_plates_id_count,
        COUNT(lp.liked_plates_id) + COUNT(sp.saved_plates_id) AS reacts_count,
        ROW_NUMBER() OVER (
            PARTITION BY price_history.plates_id
            ORDER BY price_history.price_history_id DESC
        ) AS rownumber
    FROM public.price_history
        LEFT JOIN public.liked_plates AS lp ON lp.plates_id = price_history.plates_id
        LEFT JOIN public.saved_plates AS sp ON sp.plates_id = price_history.plates_id
    GROUP BY price_history.price_history_id,
        price_history.plates_id,
        price_history.price
)
SELECT plates.plates_id,
    plates.front_text,
    plates.plates_type_id,
    plates.plates_uri,
    plates.total,
    plates.front_number,
    plates.back_number,
    plates.users_id,
    plates.special_front_id,
    plates.province_id,
    plates.information,
    latest_price.price,
    users.name,
    users.profile_uri,
    liked_plates.liked_plates_id,
    saved_plates.saved_plates_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    latest_price.liked_plates_id_count,
    latest_price.saved_plates_id_count,
    latest_price.reacts_count,
    latest_price.rownumber
FROM latest_price
    INNER JOIN public.plates ON plates.plates_id = latest_price.plates_id
    INNER JOIN public.users ON users.users_id = plates.users_id
    LEFT JOIN public.liked_plates ON liked_plates.plates_id = plates.plates_id
    AND liked_plates.users_id = $1
    LEFT JOIN public.saved_plates ON saved_plates.plates_id = plates.plates_id
    AND saved_plates.users_id = $1
    LEFT JOIN public.liked_store ON liked_store.store_id = plates.users_id
    AND liked_store.users_id = $1
    LEFT JOIN public.saved_store ON saved_store.store_id = plates.users_id
    AND saved_store.users_id = $1
WHERE latest_price.rownumber = 1
    AND is_selling IS TRUE
    AND is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.special_front_id != 1
ORDER BY {}
LIMIT $5 OFFSET $6",
        sort_by
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(payload.plates_type_id_list)
        .bind(payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_pattern(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let pattern = payload.pattern;
    let sort_by = match payload.sort_by.as_str() {
        "priceLowToHigh" => "latest_price.price ASC",
        "priceHighToLow" => "latest_price.price DESC",
        "reacts" => "latest_price.reacts_count DESC",
        _ => "latest_price.price ASC",
    };
    let sql = format!(
        "WITH latest_price AS (
    SELECT price_history.price_history_id,
        price_history.plates_id,
        price_history.price,
        COUNT(lp.liked_plates_id) AS liked_plates_id_count,
        COUNT(sp.saved_plates_id) AS saved_plates_id_count,
        COUNT(lp.liked_plates_id) + COUNT(sp.saved_plates_id) AS reacts_count,
        ROW_NUMBER() OVER (
            PARTITION BY price_history.plates_id
            ORDER BY price_history.price_history_id DESC
        ) AS rownumber
    FROM public.price_history
        LEFT JOIN public.liked_plates AS lp ON lp.plates_id = price_history.plates_id
        LEFT JOIN public.saved_plates AS sp ON sp.plates_id = price_history.plates_id
    GROUP BY price_history.price_history_id,
        price_history.plates_id,
        price_history.price
)
SELECT plates.plates_id,
    plates.front_text,
    plates.plates_type_id,
    plates.plates_uri,
    plates.total,
    plates.front_number,
    plates.back_number,
    plates.users_id,
    plates.special_front_id,
    plates.province_id,
    plates.information,
    latest_price.price,
    users.name,
    users.profile_uri,
    liked_plates.liked_plates_id,
    saved_plates.saved_plates_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    latest_price.liked_plates_id_count,
    latest_price.saved_plates_id_count,
    latest_price.reacts_count,
    latest_price.rownumber
FROM latest_price
    INNER JOIN public.plates ON plates.plates_id = latest_price.plates_id
    INNER JOIN public.{pattern} ON {pattern}.plates_id = latest_price.plates_id
    INNER JOIN public.users ON users.users_id = plates.users_id
    LEFT JOIN public.liked_plates ON liked_plates.plates_id = plates.plates_id
    AND liked_plates.users_id = $1
    LEFT JOIN public.saved_plates ON saved_plates.plates_id = plates.plates_id
    AND saved_plates.users_id = $1
    LEFT JOIN public.liked_store ON liked_store.store_id = plates.users_id
    AND liked_store.users_id = $1
    LEFT JOIN public.saved_store ON saved_store.store_id = plates.users_id
    AND saved_store.users_id = $1
WHERE latest_price.rownumber = 1
    AND is_selling IS TRUE
    AND is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
ORDER BY {sort_by}
LIMIT $5 OFFSET $6",
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(payload.plates_type_id_list)
        .bind(payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_plates_type_province() {
    // 0 -- ตจว
    // 1 กทม
    //
}
