use app_789plates_server::{
    account::{delete_account, edit_information, edit_name, edit_profile_picture},
    auth_handlers::{
        add_secondary_email, check_availability_email, check_verification_code, create_new_account,
        forgot_password, reset_password, sign_in,
    },
    graceful_shutdown::shutdown_signal,
    jwt::{renew_token, verify_signature},
};
use axum::{
    handler::Handler,
    http::StatusCode,
    middleware::{self},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::time;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/app789plates")
        .await
        .unwrap();
    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/checkavailabilityemail", post(check_availability_email))
        .route("/checkverificationcode", post(check_verification_code))
        .route("/createnewaccount", post(create_new_account))
        .route("/signin", post(sign_in))
        .route("/renewtoken", post(renew_token))
        // done mark
        .route("/forgotpassword", post(forgot_password))
        .route("/resetpassword", put(reset_password))
        .route(
            "/addsecondaryemail",
            post(add_secondary_email.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/deleteaccount",
            delete(delete_account.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/editname",
            put(edit_name.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/editprofilepicture",
            put(edit_profile_picture.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/editinformation",
            put(edit_information.layer(middleware::from_fn(verify_signature))),
        )
        .route(
            "/search",
            get(search.layer(middleware::from_fn(verify_signature))),
        )
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(time::Duration::from_secs(30)))
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

// user

async fn search() -> impl IntoResponse {}

//  plates
// add edit delete transfer show hide

// search search with condition

// home feed fetch timeline

// like plate, like profile
