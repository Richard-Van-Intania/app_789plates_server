use std::collections::HashMap;

use crate::{authentication::Authentication, constants::API_KEY};
use axum::{
    body::to_bytes,
    extract::{Query, Request, State},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use email_address::EmailAddress;
use hyper::StatusCode;
use sqlx::PgPool;

pub async fn validate_email(request: Request, next: Next) -> Result<impl IntoResponse, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 5242880).await;
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

pub async fn validate_email_s(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 5242880).await;
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

// done
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
