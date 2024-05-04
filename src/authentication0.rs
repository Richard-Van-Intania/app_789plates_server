use crate::{
    auth::{Authentication, Claims},
    constants::{
        ACCESS_TOKEN_KEY, EXP_DAY, EXP_MIN, ISSUER, MINUTES, NULL_ALIAS_INT, NULL_ALIAS_STRING,
        REFRESH_TOKEN_KEY,
    },
    mailer::send_email,
};
use axum::{extract::State, Json};
use chrono::{DateTime, Duration, Utc};
use hyper::StatusCode;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::PgPool;

// done
pub async fn create_verification(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let rand: u8 = random();
    let reference: i32 = rand as i32;
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let code: i32 = rng.gen_range(999..999999);
    let expire: DateTime<Utc> = Utc::now() + Duration::minutes(MINUTES);
    let insert: Result<(i32, i32), sqlx::Error> = sqlx::query_as("INSERT INTO public.verification(reference, code, expire) VALUES ($1, $2, $3) RETURNING verification_id, reference")
        .bind(reference)
        .bind(code)
        .bind(expire)
        .fetch_one(&pool)
        .await;
    if let Ok((verification_id, reference)) = insert {
        let sent = send_email(&payload.email, reference, code);
        if let Ok(_) = sent {
            Ok(Json(Authentication {
                verification_id,
                reference,
                code: payload.code,
                email: payload.email,
                password: payload.password,
                access_token: payload.access_token,
                refresh_token: payload.refresh_token,
                users_id: payload.users_id,
            }))
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// done
pub async fn validate_verification(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let fetch: Result<Option<(i32, DateTime<Utc>)>, sqlx::Error> = sqlx::query_as("SELECT verification_id, expire FROM public.verification WHERE (verification_id = $1 AND reference = $2 AND code = $3 AND verified = false)")
        .bind(payload.verification_id)
        .bind(payload.reference)
        .bind(payload.code)
        .fetch_optional(&pool)
        .await;
    match fetch {
        Ok(ok) => {
            match ok {
                Some((verification_id, expire)) => {
                    let date = Utc::now();
                    if expire > date {
                        let update = sqlx::query("UPDATE public.verification SET verified = true WHERE verification_id = $1").bind(verification_id).execute(&pool).await;
                        match update {
                            Ok(_) => Ok(Json(Authentication {
                                verification_id: payload.verification_id,
                                reference: payload.reference,
                                code: payload.code,
                                email: payload.email,
                                password: payload.password,
                                access_token: payload.access_token,
                                refresh_token: payload.refresh_token,
                                users_id: payload.users_id,
                            })),
                            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                        }
                    } else {
                        Err(StatusCode::BAD_REQUEST)
                    }
                }
                None => Err(StatusCode::BAD_REQUEST),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// done
pub async fn create_new_account(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email;
    let password = payload.password;
    let fetch: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as("SELECT verification_id FROM public.verification WHERE (verification_id = $1 AND reference = $2 AND code = $3 AND verified = true)")
        .bind(payload.verification_id)
        .bind(payload.reference)
        .bind(payload.code)
        .fetch_optional(&pool)
        .await;
    match fetch {
        Ok(ok) => match ok {
            Some(_) => {
                let date = Utc::now();
                let insert: Result<(i32,), sqlx::Error> = sqlx::query_as("INSERT INTO public.users (name, email, password, created_date, latest_sign_in) VALUES ($1, $2, $3, $4, $5) RETURNING users_id")
                    .bind(*email.split("@").collect::<Vec<&str>>().get(0).unwrap())
                    .bind(&email)
                    .bind(blake3::hash(password.as_bytes()).to_string())
                    .bind(date)
                    .bind(date)
                    .fetch_one(&pool)
                    .await;
                match insert {
                    Ok((users_id,)) => {
                        let access_claims = Claims {
                            iat: date.timestamp() as usize,
                            exp: (date + Duration::minutes(EXP_MIN)).timestamp() as usize,
                            iss: ISSUER.to_string(),
                            sub: users_id.to_string(),
                        };
                        let refresh_claims = Claims {
                            iat: date.timestamp() as usize,
                            exp: (date + Duration::days(EXP_DAY)).timestamp() as usize,
                            iss: ISSUER.to_string(),
                            sub: users_id.to_string(),
                        };
                        let access_token = encode(
                            &Header::default(),
                            &access_claims,
                            &EncodingKey::from_secret(ACCESS_TOKEN_KEY.as_ref()),
                        );
                        let refresh_token = encode(
                            &Header::default(),
                            &refresh_claims,
                            &EncodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
                        );
                        Ok(Json(Authentication {
                            verification_id: NULL_ALIAS_INT,
                            reference: NULL_ALIAS_INT,
                            code: NULL_ALIAS_INT,
                            email,
                            password,
                            access_token: access_token.unwrap(),
                            refresh_token: refresh_token.unwrap(),
                            users_id,
                        }))
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
            None => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
