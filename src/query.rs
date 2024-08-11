use axum::{extract::Query, response::IntoResponse};
use hyper::StatusCode;
use std::collections::HashMap;

fn search(Query(params): Query<HashMap<String, String>>) -> Result<impl IntoResponse, StatusCode> {
    match params.get("query") {
        Some(query) => Ok(query.to_string().to_uppercase()),
        None => Err(StatusCode::BAD_REQUEST),
    }
}
