use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::mailer::{send_email, MINUTES};

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Serialize)]
pub struct VerificationRes {
    pub verification_id: i32,
    pub email: String,
    pub reference: i32,
}

#[derive(Deserialize)]
pub struct VerificationCode {
    pub verification_id: i32,
    pub reference: i32,
    pub code: i32,
}

#[derive(Deserialize)]
pub struct CreateNewAccount {
    pub verification_id: i32,
    pub reference: i32,
    pub code: i32,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignIn {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePassword {
    pub refresh_token: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AddSecondaryEmail {
    pub refresh_token: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Authentication {
    pub verification_id: i32,
    pub reference: i32,
    pub code: i32,
    pub email: String,
    pub secondary_email: String,
    pub password: String,
    pub access_token: String,
    pub refresh_token: String,
    pub users_id: i32,
}

pub const NULL_ALIAS_STRING: &'static str = "null";
pub const NULL_ALIAS_INT: i32 = 0;

pub async fn check_availability_email(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if valid {
        let fetch_email = sqlx::query(
            "SELECT users_id FROM public.users WHERE primary_email = $1 OR secondary_email = $2",
        )
        .bind(&email)
        .bind(&email)
        .fetch_all(&pool)
        .await;
        if let Ok(rows) = fetch_email {
            if rows.is_empty() {
                let rand: u8 = random();
                let reference: i32 = rand as i32;
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let code: i32 = rng.gen_range(999..999999);
                let expire: DateTime<Utc> = Utc::now() + Duration::minutes(MINUTES);
                let insert_code: Result<(i32, i32), sqlx::Error> = sqlx::query_as(
                    "INSERT INTO public.verification(reference, code, expire)
                VALUES ($1, $2, $3)
                RETURNING verification_id, reference",
                )
                .bind(reference)
                .bind(code)
                .bind(expire)
                .fetch_one(&pool)
                .await;
                if let Ok((verification_id, reference)) = insert_code {
                    let sent = send_email(&email, reference, code);
                    if let Ok(_) = sent {
                        Ok(Json(Authentication {
                            verification_id,
                            reference,
                            code: NULL_ALIAS_INT,
                            email,
                            secondary_email: NULL_ALIAS_STRING.to_owned(),
                            password: NULL_ALIAS_STRING.to_owned(),
                            access_token: NULL_ALIAS_STRING.to_owned(),
                            refresh_token: NULL_ALIAS_STRING.to_owned(),
                            users_id: NULL_ALIAS_INT,
                        }))
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            } else {
                Err(StatusCode::CONFLICT)
            }
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn check_verification_code(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let fetch_code: Result<Option<(i32,DateTime<Utc>)>,  sqlx::Error> = sqlx::query_as(
        "SELECT verification_id, expire FROM public.verification WHERE verification_id = $1 AND reference = $2 AND code = $3 AND verified = false",
    )
    .bind(payload.verification_id)
    .bind(payload.reference)
    .bind(payload.code)
    .fetch_optional(&pool)
    .await;
    match fetch_code {
        Ok(ok) => match ok {
            Some((verification_id, expire)) => {
                let date = Utc::now();
                if expire > date {
                    let update_code = sqlx::query(
                        "UPDATE public.verification SET verified = true WHERE verification_id = $1",
                    )
                    .bind(verification_id)
                    .execute(&pool)
                    .await;
                    match update_code {
                        Ok(_) => Ok(Json(Authentication {
                            verification_id,
                            reference: payload.reference,
                            code: payload.code,
                            email: payload.email,
                            secondary_email: NULL_ALIAS_STRING.to_owned(),
                            password: NULL_ALIAS_STRING.to_owned(),
                            access_token: NULL_ALIAS_STRING.to_owned(),
                            refresh_token: NULL_ALIAS_STRING.to_owned(),
                            users_id: NULL_ALIAS_INT,
                        })),
                        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                    }
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
            None => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
