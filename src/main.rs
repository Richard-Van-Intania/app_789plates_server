use app_789plates_server::{
    authentication::{Email, VerifyCode, VerifyEmailRes},
    mailer::send_email,
};
use axum::{
    extract::State,
    http::StatusCode,
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
        .route("/verifycode", post(verify_code))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() {}

async fn verify_email(
    State(pool): State<PgPool>,
    Json(payload): Json<Email>,
) -> (StatusCode, Json<Option<VerifyEmailRes>>) {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if !valid {
        (StatusCode::FORBIDDEN, (Json(None)))
    } else {
        let rows = sqlx::query("SELECT * FROM public.users")
            .fetch_all(&pool)
            .await;
        if let Ok(rows) = rows {
            if rows.is_empty() {
                let sent = send_email(&email, 99, 99999);
                match sent {
                    Ok(_) => (
                        StatusCode::OK,
                        (Json(Some(VerifyEmailRes {
                            email,
                            uuid: String::from("94048355-895f-46e6-b5f9-56e08a35fa2a"),
                            reference: 99,
                        }))),
                    ),
                    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, (Json(None))),
                }
            } else {
                (StatusCode::FORBIDDEN, (Json(None)))
            }
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, (Json(None)))
        }
    }
}

async fn verify_code(State(pool): State<PgPool>, Json(payload): Json<VerifyCode>) {}
