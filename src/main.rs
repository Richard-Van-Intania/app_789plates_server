use app_789plates_server::{
    app_state::AppState,
    authentication::{
        change_password, create_new_account, create_verification, create_verification_forgot,
        delete_account, renew_token, reset_password, sign_in, validate_verification,
    },
    constants::{AWS_ACCESS_KEY_ID, AWS_REGION, AWS_SECRET_ACCESS_KEY},
    middleware::{validate_api_key, validate_email, validate_email_unique, validate_token},
    plates::{
        add_liked_plates, add_liked_store, add_new_plates, add_saved_plates, add_saved_store,
        analyze_new_pattern, delete_plates, edit_is_pin, edit_is_selling, edit_plates_information,
        edit_total, fetch_special_front, insert_new_price, remove_liked_plates, remove_liked_store,
        remove_saved_plates, remove_saved_store,
    },
    profile::{edit_information, edit_name, fetch_profile},
    query::{
        query_explore, query_pattern, query_plates_info, query_plates_type_province,
        query_special_front, query_suggestion_back_number, query_users_info,
        query_users_plates_pin, query_users_plates_unpin, query_vehicle_type_province,
        search_number, search_number_text, search_number_text_number, search_text,
        search_text_number, search_users_info,
    },
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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    env::set_var("AWS_ACCESS_KEY_ID", AWS_ACCESS_KEY_ID);
    env::set_var("AWS_SECRET_ACCESS_KEY", AWS_SECRET_ACCESS_KEY);
    env::set_var("AWS_REGION", AWS_REGION);
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

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
        .route(
            "/fetch_special_front",
            get(fetch_special_front.layer(middleware::from_fn(validate_token))),
        )
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
        .route(
            "/add_liked_plates",
            post(add_liked_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/remove_liked_plates",
            post(remove_liked_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/add_saved_plates",
            post(add_saved_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/remove_saved_plates",
            post(remove_saved_plates.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/add_liked_store",
            post(add_liked_store.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/remove_liked_store",
            post(remove_liked_store.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/add_saved_store",
            post(add_saved_store.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/remove_saved_store",
            post(remove_saved_store.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_special_front",
            post(query_special_front.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_pattern",
            post(query_pattern.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_plates_type_province",
            post(query_plates_type_province.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_vehicle_type_province",
            post(query_vehicle_type_province.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_suggestion_back_number",
            post(query_suggestion_back_number.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_explore",
            post(query_explore.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_number_text_number",
            post(search_number_text_number.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_number_text",
            post(search_number_text.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_text_number",
            post(search_text_number.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_text",
            post(search_text.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_number",
            post(search_number.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_plates_info",
            post(query_plates_info.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/search_users_info",
            post(search_users_info.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_users_info",
            post(query_users_info.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_users_plates_pin",
            post(query_users_plates_pin.layer(middleware::from_fn(validate_token))),
        )
        .route(
            "/query_users_plates_unpin",
            post(query_users_plates_unpin.layer(middleware::from_fn(validate_token))),
        )
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(time::Duration::from_secs(10)))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
