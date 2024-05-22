use app_789plates_server::{
    auth::Authentication,
    authentication::{
        change_password, create_new_account, create_verification, create_verification_forgot,
        delete_account, renew_token, reset_password, sign_in, validate_verification,
    },
    middleware::{validate_api_key, validate_email, validate_email_unique, validate_token},
    profile::{edit_information, edit_name, edit_profile_photo, fetch_profile},
    shutdown::shutdown_signal,
};
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{DefaultBodyLimit, FromRef, Query, Request, State},
    handler::Handler,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Local;
use sqlx::{PgPool, Pool, Postgres};
use std::{array::from_ref, collections::HashMap, time};
use tokio::{fs, time::sleep};
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/production")
        .await
        .unwrap();
    let app = Router::new()
        .route(
            "/",
            get((|| async {}).layer(middleware::from_fn(validate_api_key))),
        )
        .route(
            "/create_verification",
            post(
                create_verification.layer(
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(validate_api_key))
                        .layer(middleware::from_fn(validate_email))
                        .layer(middleware::from_fn_with_state(
                            pool.clone(),
                            validate_email_unique,
                        )),
                ),
            ),
        )
        .route(
            "/validate_verification",
            post(validate_verification.layer(middleware::from_fn(validate_api_key))),
        )
        .route(
            "/create_new_account",
            post(
                create_new_account.layer(
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(validate_api_key))
                        .layer(middleware::from_fn(validate_email))
                        .layer(middleware::from_fn_with_state(
                            pool.clone(),
                            validate_email_unique,
                        )),
                ),
            ),
        )
        .route(
            "/sign_in",
            post(
                sign_in.layer(
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(validate_api_key))
                        .layer(middleware::from_fn(validate_email)),
                ),
            ),
        )
        .route(
            "/create_verification_forgot",
            post(
                create_verification_forgot.layer(
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(validate_api_key))
                        .layer(middleware::from_fn(validate_email)),
                ),
            ),
        )
        .route(
            "/reset_password",
            put(reset_password.layer(
                ServiceBuilder::new()
                    .layer(middleware::from_fn(validate_api_key))
                    .layer(middleware::from_fn(validate_email)),
            )),
        )
        .route(
            "/renew_token",
            post(renew_token.layer(middleware::from_fn(validate_api_key))),
        )
        .route(
            "/change_password",
            put(change_password.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/delete_account",
            delete(
                delete_account.layer(
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(validate_token))
                        .layer(middleware::from_fn(validate_email)),
                ),
            ),
        )
        .route(
            "/fetch_profile",
            post(fetch_profile.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_name",
            put(edit_name.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_information",
            put(edit_information.layer(middleware::from_fn(validate_token))),
        )
        // here
        .route(
            "/editprofilepicture",
            put(edit_profile_photo.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search",
            get(search.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/test_bytes",
            post(test_bytes.layer(DefaultBodyLimit::max(5242880))),
        )
        // .route("/test_bytes", post(test_bytes))
        // .route("/test", get(test.layer(middleware::from_fn(verify_key))))
        // without middleware
        // .nest_service("/assets", ServeDir::new("assets"))
        // with middleware
        .nest_service(
            "/assets",
            ServiceBuilder::new()
                .layer(middleware::from_fn(validate_api_key))
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

async fn root(Json(payload): Json<Authentication>) -> Result<impl IntoResponse, StatusCode> {
    println!("{:#?}", payload);
    println!("{:?}", payload);
    Ok(())
}

async fn test() -> Result<impl IntoResponse, StatusCode> {
    println!("hello from test at {}", Local::now());
    Ok(())
}

async fn search(
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
