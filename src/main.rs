use app_789plates_server::authentication::Email;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use email_address::EmailAddress;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:mysecretpassword@localhost:5432/app789plates")
        .await
        .unwrap();
    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/verifyemail", post(verify_email))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn verify_email(
    State(state): State<PgPool>,
    Json(payload): Json<Email>,
) -> (StatusCode, Json<Option<Email>>) {
    let valid = EmailAddress::is_valid(&payload.email.trim().to_lowercase());
    if valid {
        // check exist in db
        (
            StatusCode::OK,
            (Json(Some(Email {
                email: payload.email.trim().to_lowercase(),
            }))),
        )
    } else {
        (StatusCode::FORBIDDEN, (Json(None)))
    }
}
