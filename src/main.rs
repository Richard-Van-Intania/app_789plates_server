use app_789plates_server::{
    authentication::{Email, VerifyCode, VerifyRef},
    mailer::send_email,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use email_address::EmailAddress;
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::PgPool;
use uuid::Uuid;

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
) -> (StatusCode, Json<Option<VerifyRef>>) {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if !valid {
        (StatusCode::BAD_REQUEST, (Json(None)))
    } else {
        let rows = sqlx::query("SELECT * FROM public.users")
            .fetch_all(&pool)
            .await;
        if let Ok(rows) = rows {
            if rows.is_empty() {
                let uuid = Uuid::new_v4().to_string();
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let code = rng.gen_range(1000..999999);
                let reference: u8 = random();
                // write to db
                let sent = send_email(&email, reference, code);
                match sent {
                    Ok(_) => (
                        StatusCode::OK,
                        (Json(Some(VerifyRef {
                            email,
                            uuid,
                            reference,
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
