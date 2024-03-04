use app_789plates_server::{
    authentication::{Email, VerificationCode, VerificationRes},
    mailer::{send_email, MINUTES},
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:mysecretpassword@localhost:5432/app789plates")
        .await
        .unwrap();
    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/checkavailabilityemail", post(check_availability_email))
        .route("/checkverificationcode", post(check_verification_code))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn debug() {}

async fn check_availability_email(
    State(pool): State<PgPool>,
    Json(payload): Json<Email>,
) -> (StatusCode, Json<Option<VerificationRes>>) {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if valid {
        let fetch = sqlx::query(
            "SELECT users_id FROM public.users WHERE primary_email = $1 OR secondary_email = $2",
        )
        .bind(&email)
        .bind(&email)
        .fetch_all(&pool)
        .await;
        if let Ok(rows) = fetch {
            if rows.is_empty() {
                let rand: u8 = random();
                let reference: i32 = rand as i32;
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let code: i32 = rng.gen_range(999..999999);
                let expire: DateTime<Utc> = Utc::now() + Duration::minutes(MINUTES);
                let insert: Result<(i32, i32), sqlx::Error> = sqlx::query_as(
                    "INSERT INTO public.verification(reference, code, expire)
                VALUES ($1, $2, $3)
                RETURNING verification_id, reference",
                )
                .bind(reference)
                .bind(code)
                .bind(expire)
                .fetch_one(&pool)
                .await;
                if let Ok((verification_id, reference)) = insert {
                    let sent = send_email(&email, reference, code);
                    match sent {
                        Ok(_) => (
                            StatusCode::OK,
                            (Json(Some(VerificationRes {
                                verification_id,
                                email,
                                reference,
                            }))),
                        ),
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, (Json(None))),
                    }
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, (Json(None)))
                }
            } else {
                (StatusCode::FORBIDDEN, (Json(None)))
            }
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, (Json(None)))
        }
    } else {
        (StatusCode::BAD_REQUEST, (Json(None)))
    }
}

async fn check_verification_code(
    State(pool): State<PgPool>,
    Json(payload): Json<VerificationCode>,
) {
}
