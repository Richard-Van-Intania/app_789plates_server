use app_789plates_server::{
    app_state::AppState,
    authentication::{
        change_password, create_new_account, create_verification, create_verification_forgot,
        delete_account, renew_token, reset_password, sign_in, validate_verification,
    },
    constants::{AWS_ACCESS_KEY_ID, AWS_REGION, AWS_SECRET_ACCESS_KEY},
    middleware::{validate_api_key, validate_email, validate_email_unique, validate_token},
    plates::{
        add_new_plates, analyze_new_pattern, delete_plates, edit_is_pin, edit_is_selling,
        edit_plates_information, edit_total, insert_new_price,
    },
    profile::{edit_information, edit_name, fetch_profile},
    s3_operations::{generate_presigned_url, update_object},
    shutdown::shutdown_signal,
};
use axum::{
    handler::Handler,
    middleware::{self},
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::{env, time};
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    // logging and diagnostics system
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // aws s3
    env::set_var("AWS_ACCESS_KEY_ID", AWS_ACCESS_KEY_ID);
    env::set_var("AWS_SECRET_ACCESS_KEY", AWS_SECRET_ACCESS_KEY);
    env::set_var("AWS_REGION", AWS_REGION);
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

    // postgresql
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/production")
        .await
        .unwrap();

    let state = AppState { pool, client };
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
                            state.clone(),
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
                            state.clone(),
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
        .route(
            "/generate_presigned_url",
            get(generate_presigned_url.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/update_object",
            put(update_object.layer(middleware::from_fn(validate_token))),
        )
        // here
        .route(
            "/add_new_plates",
            post(add_new_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/delete_plates",
            delete(delete_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/insert_new_price",
            post(insert_new_price.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_plates_information",
            put(edit_plates_information.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_is_selling",
            put(edit_is_selling.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_is_pin",
            put(edit_is_pin.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/edit_total",
            put(edit_total.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/analyze_new_pattern",
            get(analyze_new_pattern.layer(middleware::from_fn(validate_api_key))),
        )
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(time::Duration::from_secs(15)))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
