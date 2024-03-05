use app_789plates_server::{
    authentication::{CreateNewAccount, Email, VerificationCode, VerificationRes},
    mailer::{send_email, MINUTES},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
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
        .route("/createnewaccount", post(create_new_account))
        .route("/debug", get(debug))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn debug() -> impl IntoResponse {
    let email = String::from("lillpu@live.com");
    let text = *email.split("@").collect::<Vec<&str>>().get(0).unwrap();
    text.to_owned()
}

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
                (StatusCode::CONFLICT, (Json(None)))
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
) -> impl IntoResponse {
    let fetch: Result<Option<(i32,DateTime<Utc>)>,  sqlx::Error> = sqlx::query_as(
        "SELECT verification_id, expire FROM public.verification WHERE verification_id = $1 AND reference = $2 AND code = $3 AND verified = false",
    )
    .bind(payload.verification_id)
    .bind(payload.reference)
    .bind(payload.code)
    .fetch_optional(&pool)
    .await;
    match fetch {
        Ok(ok) => match ok {
            Some((verification_id, expire)) => {
                let date = Utc::now();
                if expire > date {
                    let update = sqlx::query(
                        "UPDATE public.verification SET verified = true WHERE verification_id = $1",
                    )
                    .bind(verification_id)
                    .execute(&pool)
                    .await;
                    match update {
                        Ok(_) => StatusCode::OK,
                        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    }
                } else {
                    StatusCode::BAD_REQUEST
                }
            }
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn create_new_account(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNewAccount>,
) -> impl IntoResponse {
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
        match fetch {
            Ok(rows) => {
                if rows.is_empty() {
                    let fetch: Result<Option<(i32,)>,  sqlx::Error> = sqlx::query_as(
                        "SELECT verification_id FROM public.verification WHERE verification_id = $1 AND reference = $2 AND code = $3 AND verified = true",
                    )
                    .bind(payload.verification_id)
                    .bind(payload.reference)
                    .bind(payload.code)
                    .fetch_optional(&pool)
                    .await;
                    match fetch {
                        Ok(_) => {
                            let insert = sqlx::query(
                                "INSERT INTO public.users (name, primary_email, password, created_date) VALUES ($1, $2, $3, $4)",
                            )
                            .bind(*email.split("@").collect::<Vec<&str>>().get(0).unwrap())
                            .bind(&email)
                            .bind(blake3::hash(payload.password.as_bytes()).to_string())
                            .bind(Utc::now())
                            .execute(&pool)
                            .await;
                            match insert {
                                Ok(_) => {
                                    // return jwt here
                                    StatusCode::OK
                                }
                                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                            }
                        }
                        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    }
                } else {
                    StatusCode::BAD_REQUEST
                }
            }
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}
