use crate::app_state::AppState;
use axum::{extract::State, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatesFilter {
    pub plates_id: i32,
    pub users_id: i32,
    pub pattern: String,
    pub plates_type_id: i32,
    pub province_id: i32,
    pub vehicle_type_id: i32,
    pub search_text: String,
    pub search_text_pattern_id: i32,
    pub search_text_front_number: i32,
    pub search_text_front_text: String,
    pub search_text_back_number: i32,
    pub back_number: i32,
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
    pub vehicle_type_id: i32,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersFilter {
    pub users_id: i32,
    pub store_id: i32,
    pub search_text: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsersData {
    pub users_id: i32,
    pub name: String,
    pub profile_uri: Option<String>,
    pub cover_uri: Option<String>,
    pub information: Option<String>,
    pub liked_store_id: Option<i32>,
    pub saved_store_id: Option<i32>,
    pub liked_store_id_count: i64,
    pub saved_store_id_count: i64,
    pub total_assets: i64,
    pub plates_count: i64,
    pub average_score: Option<i64>,
}

fn order_by(sort_by: String) -> &'static str {
    match sort_by.as_str() {
        "priceLowToHigh" => "latest_price.price ASC",
        "priceHighToLow" => "latest_price.price DESC",
        "reacts" => "latest_price.reacts_count DESC",
        "random" => "RANDOM()",
        _ => "latest_price.price ASC",
    }
}

fn province_text(province_id: i32) -> &'static str {
    match province_id {
        0 => "!= 1",
        1 => "= 1",
        _ => "!= 0",
    }
}

pub async fn query_special_front(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.special_front_id != 1
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
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
    let sort_by = order_by(payload.sort_by);
    let pattern = payload.pattern;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_plates_type_province(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let province = province_text(payload.province_id);
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.plates_type_id = $7
    AND plates.province_id {province}
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(payload.plates_type_id)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_vehicle_type_province(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let province = province_text(payload.province_id);
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.vehicle_type_id = $7
    AND plates.province_id {province}
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(payload.vehicle_type_id)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_plates_info(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
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
    WHERE price_history.plates_id = $2
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.plates_id)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_suggestion_back_number(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let back_number = payload.back_number;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.back_number = $7
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(back_number)
        .fetch_all(&pool)
        .await;
    let mut plates_list = match fetch {
        Ok(ok) => {
            if ok.len() < 20 {
                ok
            } else {
                return Ok(Json(ok));
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND CAST(plates.back_number AS text) LIKE '%{back_number}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(mut ok) => {
            plates_list.append(&mut ok);
            Ok(Json(plates_list))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_explore(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_number_text_number(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let search_text_front_text = payload.search_text_front_text;
    let search_text_back_number = payload.search_text_back_number;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.front_number = $7
    AND plates.back_number = $8
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(payload.search_text_front_number)
        .bind(payload.search_text_back_number)
        .fetch_all(&pool)
        .await;
    let mut plates_list = match fetch {
        Ok(ok) => {
            if ok.len() < 20 {
                ok
            } else {
                return Ok(Json(ok));
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.front_number = $7
    AND CAST(plates.back_number AS text) LIKE '%{search_text_back_number}%'
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(payload.search_text_front_number)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(mut ok) => {
            plates_list.append(&mut ok);
            Ok(Json(plates_list))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_number_text(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let search_text_front_text = payload.search_text_front_text;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.front_number = $7
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(payload.search_text_front_number)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_text_number(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let search_text_front_text = payload.search_text_front_text;
    let search_text_back_number = payload.search_text_back_number;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.back_number = $7
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(search_text_back_number)
        .fetch_all(&pool)
        .await;
    let mut plates_list = match fetch {
        Ok(ok) => {
            if ok.len() < 20 {
                ok
            } else {
                return Ok(Json(ok));
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND CAST(plates.back_number AS text) LIKE '%{search_text_back_number}%'
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(mut ok) => {
            plates_list.append(&mut ok);
            Ok(Json(plates_list))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_text(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let search_text_front_text = payload.search_text_front_text;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.front_text LIKE '{search_text_front_text}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_number(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<PlatesFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
    let sort_by = order_by(payload.sort_by);
    let search_text_back_number = payload.search_text_back_number;
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND plates.back_number = $7
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .bind(search_text_back_number)
        .fetch_all(&pool)
        .await;
    let mut plates_list = match fetch {
        Ok(ok) => {
            if ok.len() < 20 {
                ok
            } else {
                return Ok(Json(ok));
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND latest_price.price <= $2
    AND plates.plates_type_id IN (
        SELECT unnest ($3::integer [])
    )
    AND plates.province_id IN (
        SELECT unnest ($4::integer [])
    )
    AND CAST(plates.back_number AS text) LIKE '%{search_text_back_number}%'
ORDER BY {sort_by}
LIMIT $5 OFFSET $6"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.price_under)
        .bind(&payload.plates_type_id_list)
        .bind(&payload.province_id_list)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(mut ok) => {
            plates_list.append(&mut ok);
            Ok(Json(plates_list))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_users_info(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<UsersFilter>,
) -> Result<Json<Vec<UsersData>>, StatusCode> {
    let sql = format!(
        "WITH latest_price AS (
    SELECT price_history.price_history_id,
        price_history.plates_id,
        price_history.price,
        ROW_NUMBER() OVER (
            PARTITION BY price_history.plates_id
            ORDER BY price_history.price_history_id DESC
        ) AS rownumber,
        plates.users_id
    FROM public.price_history
        INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
        AND plates.users_id = $2
        AND plates.is_selling IS TRUE
        AND plates.is_temporary IS NOT TRUE
)
SELECT users.users_id,
    users.name,
    users.profile_uri,
    users.cover_uri,
    users.information,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    COUNT(ls.liked_store_id) AS liked_store_id_count,
    COUNT(ss.saved_store_id) AS saved_store_id_count,
    SUM(latest_price.price) AS total_assets,
    COUNT(latest_price.plates_id) AS plates_count,
    AVG(rating.score) AS average_score
FROM latest_price
    INNER JOIN public.users ON users.users_id = latest_price.users_id
    LEFT JOIN public.liked_store ON liked_store.store_id = latest_price.users_id
    AND liked_store.users_id = $1
    LEFT JOIN public.saved_store ON saved_store.store_id = latest_price.users_id
    AND saved_store.users_id = $1
    LEFT JOIN public.rating ON rating.store_id = latest_price.users_id
    LEFT JOIN public.liked_store AS ls ON ls.store_id = latest_price.users_id
    LEFT JOIN public.saved_store AS ss ON ss.store_id = latest_price.users_id
WHERE latest_price.rownumber = 1
GROUP BY users.users_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id"
    );
    let fetch: Result<Vec<UsersData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.store_id)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_users_plates_pin(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<UsersFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
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
        INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
        AND plates.users_id = $2
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND plates.users_id = $2
    AND plates.is_pin IS TRUE
ORDER BY plates.add_date DESC
LIMIT $3 OFFSET $4"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.store_id)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn query_users_plates_unpin(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<UsersFilter>,
) -> Result<Json<Vec<PlatesData>>, StatusCode> {
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
        INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
        AND plates.users_id = $2
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
    plates.vehicle_type_id,
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
    AND plates.is_selling IS TRUE
    AND plates.is_temporary IS NOT TRUE
    AND plates.users_id = $2
    AND plates.is_pin IS NOT TRUE
ORDER BY plates.add_date DESC
LIMIT $3 OFFSET $4"
    );
    let fetch: Result<Vec<PlatesData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .bind(payload.store_id)
        .bind(payload.limit)
        .bind(payload.offset)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn search_users_info(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<UsersFilter>,
) -> Result<Json<Vec<UsersData>>, StatusCode> {
    let search_text = payload.search_text;
    let sql = format!(
        "WITH latest_price AS (
    SELECT price_history.price_history_id,
        price_history.plates_id,
        price_history.price,
        ROW_NUMBER() OVER (
            PARTITION BY price_history.plates_id
            ORDER BY price_history.price_history_id DESC
        ) AS rownumber,
        plates.users_id
    FROM public.price_history
        INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
        INNER JOIN public.users ON users.users_id = plates.users_id
        AND users.name LIKE '%{search_text}%'
        AND plates.is_selling IS TRUE
        AND plates.is_temporary IS NOT TRUE
)
SELECT users.users_id,
    users.name,
    users.profile_uri,
    users.cover_uri,
    users.information,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    COUNT(ls.liked_store_id) AS liked_store_id_count,
    COUNT(ss.saved_store_id) AS saved_store_id_count,
    SUM(latest_price.price) AS total_assets,
    COUNT(latest_price.plates_id) AS plates_count,
    AVG(rating.score) AS average_score
FROM latest_price
    INNER JOIN public.users ON users.users_id = latest_price.users_id
    LEFT JOIN public.liked_store ON liked_store.store_id = latest_price.users_id
    AND liked_store.users_id = $1
    LEFT JOIN public.saved_store ON saved_store.store_id = latest_price.users_id
    AND saved_store.users_id = $1
    LEFT JOIN public.rating ON rating.store_id = latest_price.users_id
    LEFT JOIN public.liked_store AS ls ON ls.store_id = latest_price.users_id
    LEFT JOIN public.saved_store AS ss ON ss.store_id = latest_price.users_id
WHERE latest_price.rownumber = 1
GROUP BY users.users_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id"
    );
    let fetch: Result<Vec<UsersData>, sqlx::Error> = sqlx::query_as(&sql)
        .bind(payload.users_id)
        .fetch_all(&pool)
        .await;
    match fetch {
        Ok(ok) => Ok(Json(ok)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
