use crate::app_state::AppState;
use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

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
pub struct PlatesId {
    pub plates_id: i32,
}

pub async fn insert_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> Result<Json<PlatesId>, StatusCode> {
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
                                Ok(Json(PlatesId { plates_id }))
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

pub async fn delete_plates(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(PlatesId { plates_id }): Json<PlatesId>,
) -> StatusCode {
    let delete: Result<Option<(i32,)>, sqlx::Error> =
        sqlx::query_as("DELETE FROM public.plates WHERE plates_id = $1 RETURNING plates_id")
            .bind(plates_id)
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

pub async fn insert_price(
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

pub async fn update_plates_uri(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET plates_uri = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.plates_uri)
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

pub async fn update_information(
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

pub async fn update_is_selling(
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

pub async fn update_is_pin(
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

pub async fn update_total(
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

pub async fn update_users_id(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Plates>,
) -> StatusCode {
    let update: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
        "UPDATE public.plates SET users_id = $1 WHERE plates_id = $2 RETURNING plates_id",
    )
    .bind(payload.users_id)
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

pub async fn analyze_pattern(
    plates_id: i32,
    front_text: &String,
    front_number: i32,
    back_number: i32,
    add_date: DateTime<Utc>,
    vehicle_type_id: i32,
    pool: &Pool<Postgres>,
) {
    // constants
    // pattern_168
    if back_number == 168 {
        let _ = sqlx::query("INSERT INTO public.pattern_168(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_789
    if back_number == 789 {
        let _ = sqlx::query("INSERT INTO public.pattern_789(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_289
    if back_number == 289 {
        let _ = sqlx::query("INSERT INTO public.pattern_289(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_456
    if back_number == 456 {
        let _ = sqlx::query("INSERT INTO public.pattern_456(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_911
    if back_number == 911 {
        let _ = sqlx::query("INSERT INTO public.pattern_911(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_718
    if back_number == 718 {
        let _ = sqlx::query("INSERT INTO public.pattern_718(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_992
    if back_number == 992 {
        let _ = sqlx::query("INSERT INTO public.pattern_992(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_35
    if back_number == 35 {
        let _ = sqlx::query("INSERT INTO public.pattern_35(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_488
    if back_number == 488 {
        let _ = sqlx::query("INSERT INTO public.pattern_488(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9
    if back_number == 9 {
        let _ = sqlx::query("INSERT INTO public.pattern_9(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_99
    if back_number == 99 {
        let _ = sqlx::query("INSERT INTO public.pattern_99(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_999
    if back_number == 999 {
        let _ = sqlx::query("INSERT INTO public.pattern_999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9999
    if back_number == 9999 {
        let _ = sqlx::query("INSERT INTO public.pattern_9999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_7
    if back_number == 7 {
        let _ = sqlx::query("INSERT INTO public.pattern_7(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_77
    if back_number == 77 {
        let _ = sqlx::query("INSERT INTO public.pattern_77(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_777
    if back_number == 777 {
        let _ = sqlx::query("INSERT INTO public.pattern_777(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_7777
    if back_number == 7777 {
        let _ = sqlx::query("INSERT INTO public.pattern_7777(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5
    if back_number == 5 {
        let _ = sqlx::query("INSERT INTO public.pattern_5(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_55
    if back_number == 55 {
        let _ = sqlx::query("INSERT INTO public.pattern_55(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_555
    if back_number == 555 {
        let _ = sqlx::query("INSERT INTO public.pattern_555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5555
    if back_number == 5555 {
        let _ = sqlx::query("INSERT INTO public.pattern_5555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_8
    if back_number == 8 {
        let _ = sqlx::query("INSERT INTO public.pattern_8(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_88
    if back_number == 88 {
        let _ = sqlx::query("INSERT INTO public.pattern_88(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_888
    if back_number == 888 {
        let _ = sqlx::query("INSERT INTO public.pattern_888(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_8888
    if back_number == 8888 {
        let _ = sqlx::query("INSERT INTO public.pattern_8888(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_1
    if back_number == 1 {
        let _ = sqlx::query("INSERT INTO public.pattern_1(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_599
    if back_number == 599 {
        let _ = sqlx::query("INSERT INTO public.pattern_599(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_595
    if back_number == 595 {
        let _ = sqlx::query("INSERT INTO public.pattern_595(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_959
    if back_number == 959 {
        let _ = sqlx::query("INSERT INTO public.pattern_959(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_955
    if back_number == 955 {
        let _ = sqlx::query("INSERT INTO public.pattern_955(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5959
    if back_number == 5959 {
        let _ = sqlx::query("INSERT INTO public.pattern_5959(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9595
    if back_number == 9595 {
        let _ = sqlx::query("INSERT INTO public.pattern_9595(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5599
    if back_number == 5599 {
        let _ = sqlx::query("INSERT INTO public.pattern_5599(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9955
    if back_number == 9955 {
        let _ = sqlx::query("INSERT INTO public.pattern_9955(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5995
    if back_number == 5995 {
        let _ = sqlx::query("INSERT INTO public.pattern_5995(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9559
    if back_number == 9559 {
        let _ = sqlx::query("INSERT INTO public.pattern_9559(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // variable
    // pattern_x
    if back_number > 0 && back_number < 10 {
        let _ = sqlx::query("INSERT INTO public.pattern_x(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xx
    if back_number == 11
        || back_number == 22
        || back_number == 33
        || back_number == 44
        || back_number == 55
        || back_number == 66
        || back_number == 77
        || back_number == 88
        || back_number == 99
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xxx
    if back_number == 111
        || back_number == 222
        || back_number == 333
        || back_number == 444
        || back_number == 555
        || back_number == 666
        || back_number == 777
        || back_number == 888
        || back_number == 999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xxxx
    if back_number == 1111
        || back_number == 2222
        || back_number == 3333
        || back_number == 4444
        || back_number == 5555
        || back_number == 6666
        || back_number == 7777
        || back_number == 8888
        || back_number == 9999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xxxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xy
    if back_number > 9 && back_number < 100 {
        let _ = sqlx::query("INSERT INTO public.pattern_xy(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyy
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyy(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyyy
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyyy(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xxyy
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xxyy(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyxy
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyxy(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyyx
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyyx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_yxx
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_yxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_yxxx
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_yxxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyx
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xyz
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_xyz(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_zyx
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_zyx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_wxyz
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_wxyz(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_zyxw
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_zyxw(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x00
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x00(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x000
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x000(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x99
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x99(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x999
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x55
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x55(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x555
    if false {
        let _ = sqlx::query("INSERT INTO public.pattern_x555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_rakhang
    if front_number == 0 || vehicle_type_id == 1 || front_text.starts_with("ฆ") {
        let _ =
            sqlx::query("INSERT INTO public.pattern_rakhang(plates_id, add_date) VALUES ($1, $2)")
                .bind(plates_id)
                .bind(add_date)
                .execute(pool)
                .await;
    }
}
