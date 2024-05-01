use std::collections::HashMap;

use crate::{
    authentication::Authentication,
    constants::{ACCESS_TOKEN_KEY, API_KEY, LIMIT},
    jwt::Claims,
};
use axum::{
    body::to_bytes,
    extract::{Query, Request, State},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use email_address::EmailAddress;
use hyper::StatusCode;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;

pub async fn validate_api_key(
    Query(params): Query<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match params.get("api_key") {
        Some(api_key) => {
            if api_key == API_KEY {
                let response = next.run(request).await;
                Ok(response)
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        None => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn validate_email(request: Request, next: Next) -> Result<impl IntoResponse, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, LIMIT).await;
    match bytes {
        Ok(bytes) => {
            let json = Json::<Authentication>::from_bytes(&bytes);
            match json {
                Ok(Json(mut payload)) => {
                    payload.email = payload.email.trim().to_lowercase();
                    let valid = EmailAddress::is_valid(&payload.email);
                    if valid {
                        let body = Json(payload).into_response().into_body();
                        let req = Request::from_parts(parts, body);
                        let response = next.run(req).await;
                        Ok(response)
                    } else {
                        Err(StatusCode::BAD_REQUEST)
                    }
                }
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn check_email_already_use(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, LIMIT).await;
    match bytes {
        Ok(bytes) => {
            let json = Json::<Authentication>::from_bytes(&bytes);
            match json {
                Ok(Json(payload)) => {
                    let fetch_email = sqlx::query(
                        "SELECT users_id FROM public.users WHERE (primary_email = $1 OR secondary_email = $2)",
                    )
                    .bind(&payload.email)
                    .bind(&payload.email)
                    .fetch_all(&pool)
                    .await;
                    if let Ok(rows) = fetch_email {
                        if rows.is_empty() {
                            let body = Json(payload).into_response().into_body();
                            let req = Request::from_parts(parts, body);
                            let response = next.run(req).await;
                            Ok(response)
                        } else {
                            Err(StatusCode::CONFLICT)
                        }
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn verify_signature(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let token = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(ACCESS_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    match token {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
