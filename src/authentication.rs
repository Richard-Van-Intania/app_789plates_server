use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    constants::{
        ACCESS_TOKEN_KEY, ISSUER, KEY_TOKEN, MINUTES, NULL_ALIAS_INT, NULL_ALIAS_STRING,
        REFRESH_TOKEN_KEY,
    },
    jwt::Claims,
    mailer::send_email,
};

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

// done
pub async fn check_availability_email(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email;
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
                secondary_email: NULL_ALIAS_STRING.to_string(),
                password: NULL_ALIAS_STRING.to_string(),
                access_token: NULL_ALIAS_STRING.to_string(),
                refresh_token: NULL_ALIAS_STRING.to_string(),
                users_id: NULL_ALIAS_INT,
            }))
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// done
// use both new account and forgot password
pub async fn check_verification_code(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let fetch_code: Result<Option<(i32,DateTime<Utc>)>,  sqlx::Error> = sqlx::query_as(
        "SELECT verification_id, expire FROM public.verification WHERE (verification_id = $1 AND reference = $2 AND code = $3 AND verified = false)",
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
                            secondary_email: NULL_ALIAS_STRING.to_string(),
                            password: NULL_ALIAS_STRING.to_string(),
                            access_token: NULL_ALIAS_STRING.to_string(),
                            refresh_token: NULL_ALIAS_STRING.to_string(),
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

pub async fn create_new_account(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    let password = payload.password;
    if valid && !password.is_empty() {
        let fetch_email = sqlx::query(
            "SELECT users_id FROM public.users WHERE (primary_email = $1 OR secondary_email = $2)",
        )
        .bind(&email)
        .bind(&email)
        .fetch_all(&pool)
        .await;
        match fetch_email {
            Ok(rows) => {
                if rows.is_empty() {
                    let fetch_code: Result<Option<(i32,)>,  sqlx::Error> = sqlx::query_as(
                        "SELECT verification_id FROM public.verification WHERE (verification_id = $1 AND reference = $2 AND code = $3 AND verified = true)",
                    )
                    .bind(payload.verification_id)
                    .bind(payload.reference)
                    .bind(payload.code)
                    .fetch_optional(&pool)
                    .await;
                    match fetch_code {
                        Ok(ok) => match ok {
                            Some(_) => {
                                let date = Utc::now();
                                let insert_user: Result<(i32,), sqlx::Error> = sqlx::query_as(
                                    "INSERT INTO public.users (name, primary_email, password, created_date) VALUES ($1, $2, $3, $4) RETURNING users_id",
                                )
                                .bind(*email.split("@").collect::<Vec<&str>>().get(0).unwrap())
                                .bind(&email)
                                .bind(blake3::hash(password.as_bytes()).to_string())
                                .bind(date)
                                .fetch_one(&pool)
                                .await;
                                match insert_user {
                                    Ok((users_id,)) => {
                                        let access_claims = Claims {
                                            iat: date.timestamp() as usize,
                                            exp: (date + Duration::minutes(60)).timestamp()
                                                as usize,
                                            iss: ISSUER.to_string(),
                                            sub: users_id.to_string(),
                                        };
                                        let refresh_claims = Claims {
                                            iat: date.timestamp() as usize,
                                            exp: (date + Duration::days(14)).timestamp() as usize,
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
                                            secondary_email: NULL_ALIAS_STRING.to_string(),
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
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn sign_in(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    let password = payload.password;
    if valid && !password.is_empty() {
        let fetch_users_id: Result<Option<(i32,String)>, sqlx::Error> = sqlx::query_as(
            "SELECT users_id, primary_email FROM public.users WHERE (primary_email = $1 OR secondary_email = $2) AND password = $3",
        )
        .bind(&email)
        .bind(&email)
        .bind(blake3::hash(password.as_bytes()).to_string())
        .fetch_optional(&pool)
        .await;
        if let Ok(opt_users_id) = fetch_users_id {
            if let Some((users_id, primary_email)) = opt_users_id {
                let date = Utc::now();
                let access_claims = Claims {
                    iat: date.timestamp() as usize,
                    exp: (date + Duration::minutes(60)).timestamp() as usize,
                    iss: ISSUER.to_string(),
                    sub: users_id.to_string(),
                };
                let refresh_claims = Claims {
                    iat: date.timestamp() as usize,
                    exp: (date + Duration::days(14)).timestamp() as usize,
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
                    email: primary_email,
                    secondary_email: NULL_ALIAS_STRING.to_string(),
                    password,
                    access_token: access_token.unwrap(),
                    refresh_token: refresh_token.unwrap(),
                    users_id,
                }))
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn forgot_password(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if valid {
        let fetch_email = sqlx::query(
            "SELECT users_id FROM public.users WHERE (primary_email = $1 OR secondary_email = $2)",
        )
        .bind(&email)
        .bind(&email)
        .fetch_all(&pool)
        .await;
        if let Ok(rows) = fetch_email {
            if !rows.is_empty() {
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
                            secondary_email: NULL_ALIAS_STRING.to_string(),
                            password: NULL_ALIAS_STRING.to_string(),
                            access_token: NULL_ALIAS_STRING.to_string(),
                            refresh_token: NULL_ALIAS_STRING.to_string(),
                            users_id: NULL_ALIAS_INT,
                        }))
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    let password = payload.password;
    if valid && !password.is_empty() {
        let fetch_code: Result<Option<(i32,)>,  sqlx::Error> = sqlx::query_as(
            "SELECT verification_id FROM public.verification WHERE (verification_id = $1 AND reference = $2 AND code = $3 AND verified = true)",
        )
        .bind(payload.verification_id)
        .bind(payload.reference)
        .bind(payload.code)
        .fetch_optional(&pool)
        .await;
        match fetch_code {
            Ok(ok) => match ok {
                Some(_) => {
                    let update_password: Result<(i32, String), sqlx::Error> = sqlx::query_as(
                        "UPDATE public.users
                    SET password = $1
                    WHERE (primary_email = $2 OR secondary_email = $3)
                    RETURNING users_id, primary_email",
                    )
                    .bind(blake3::hash(password.as_bytes()).to_string())
                    .bind(&email)
                    .bind(&email)
                    .fetch_one(&pool)
                    .await;
                    if let Ok((users_id, primary_email)) = update_password {
                        let date = Utc::now();
                        let access_claims = Claims {
                            iat: date.timestamp() as usize,
                            exp: (date + Duration::minutes(60)).timestamp() as usize,
                            iss: ISSUER.to_string(),
                            sub: users_id.to_string(),
                        };
                        let refresh_claims = Claims {
                            iat: date.timestamp() as usize,
                            exp: (date + Duration::days(14)).timestamp() as usize,
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
                            email: primary_email,
                            secondary_email: NULL_ALIAS_STRING.to_string(),
                            password,
                            access_token: access_token.unwrap(),
                            refresh_token: refresh_token.unwrap(),
                            users_id,
                        }))
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
                None => Err(StatusCode::BAD_REQUEST),
            },
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// inside login
pub async fn change_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ChangePassword>,
) -> StatusCode {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let users_id: i32 = claims.sub.parse().unwrap();
        let update_password = sqlx::query(
            "UPDATE public.users
        SET password = $1
        WHERE users_id = $2",
        )
        .bind(blake3::hash(payload.password.as_bytes()).to_string())
        .bind(users_id)
        .execute(&pool)
        .await;
        match update_password {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::UNAUTHORIZED
    }
}

// middleware
pub async fn verify_key(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    if bearer.token() == KEY_TOKEN {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
