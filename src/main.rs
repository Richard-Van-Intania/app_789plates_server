use app_789plates_server::{
    auth_handlers::{add_secondary_email, delete_account},
    authentication::{
        change_password, check_availability_email, check_verification_code, create_new_account,
        forgot_password, reset_password, sign_in, verify_key, Authentication,
    },
    jwt::{renew_token, verify_signature},
    profile::{edit_information, edit_name, edit_profile_picture},
    shutdown_signal::shutdown_signal,
};
use axum::{
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Query, Request},
    handler::Handler,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use sqlx::PgPool;
use std::{collections::HashMap, time};
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/app789plates")
        .await
        .unwrap();
    let app = Router::new()
        .route("/", get(|| async {}))
        .route(
            "/checkavailabilityemail",
            post(check_availability_email.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/checkverificationcode",
            post(check_verification_code.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/createnewaccount",
            post(create_new_account.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/signin",
            post(sign_in.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/forgotpassword",
            post(forgot_password.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/resetpassword",
            put(reset_password.layer(middleware::from_fn(verify_key))),
        )
        .route(
            "/renewtoken",
            post(renew_token.layer(middleware::from_fn(verify_key))),
        )
        // here
        .route(
            "/changepassword",
            put(change_password.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/addsecondaryemail",
            post(add_secondary_email.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/deleteaccount",
            // later
            delete(delete_account.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/editname",
            put(edit_name.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            // later
            "/editprofilepicture",
            put(edit_profile_picture.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/editinformation",
            put(edit_information.layer(middleware::from_fn(verify_signature))),
        )
        // here
        .route(
            "/search",
            get(search.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/test_bytes",
            post(test_bytes.layer(DefaultBodyLimit::max(5242880))),
        )
        // .route("/test_bytes", post(test_bytes))
        .route("/test", get(test.layer(middleware::from_fn(verify_key))))
        // without middleware
        // .nest_service("/assets", ServeDir::new("assets"))
        // with middleware
        .nest_service(
            "/assets",
            ServiceBuilder::new()
                .layer(middleware::from_fn(verify_key))
                .service(ServeDir::new("assets")),
        )
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(time::Duration::from_secs(15)))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn root() -> Result<impl IntoResponse, StatusCode> {
    Ok(())
}

async fn test() -> Result<impl IntoResponse, StatusCode> {
    Ok(())
}

async fn search(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    match params.get("query") {
        Some(query) => Ok(query.to_string().to_uppercase()),
        None => Err(StatusCode::BAD_REQUEST),
    }
}

async fn test_bytes(
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    match params.get("file_name") {
        Some(file_name) => {
            let write_result = fs::write(format!("assets/{}", file_name), body).await;
            match write_result {
                Ok(_) => Ok(file_name.to_string()),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::BAD_REQUEST),
    }
}

//  plates
// add edit delete transfer show hide

// search search with condition

// home feed fetch timeline

// like plate, like profile

// add plate

// fetch profile

// impl IntoResponse
