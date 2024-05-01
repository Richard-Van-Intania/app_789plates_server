use crate::{
    authentication::AddSecondaryEmail,
    constants::{ACCESS_TOKEN_KEY, ISSUER, MINUTES, REFRESH_TOKEN_KEY},
    jwt::{Claims, Token},
    mailer::send_email,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::PgPool;

pub async fn add_secondary_email(
    State(pool): State<PgPool>,
    Json(payload): Json<AddSecondaryEmail>,
) -> StatusCode {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if valid {
        let token = decode::<Claims>(
            &payload.refresh_token,
            &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
            &Validation::default(),
        );
        if let Ok(TokenData { header: _, claims }) = token {
            let users_id: i32 = claims.sub.parse().unwrap();
            let fetch_email = sqlx::query(
                "SELECT users_id FROM public.users WHERE primary_email = $1 OR secondary_email = $2",
            )
            .bind(&email)
            .bind(&email)
            .fetch_all(&pool)
            .await;
            if let Ok(rows) = fetch_email {
                if rows.is_empty() {
                    let update_secondary_email = sqlx::query(
                        "UPDATE public.users
                    SET secondary_email = $1
                    WHERE users_id = $2",
                    )
                    .bind(&email)
                    .bind(users_id)
                    .execute(&pool)
                    .await;
                    match update_secondary_email {
                        Ok(_) => StatusCode::OK,
                        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    }
                } else {
                    StatusCode::CONFLICT
                }
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        } else {
            StatusCode::UNAUTHORIZED
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn delete_account(State(pool): State<PgPool>, Json(payload): Json<Token>) -> StatusCode {
    StatusCode::OK
}
