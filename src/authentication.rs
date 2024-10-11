use crate::{
    app_state::AppState,
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
use rand::{random, rngs::SmallRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Authentication {
    pub verification_id: i32,
    pub reference: i32,
    pub code: i32,
    pub email: String,
    pub password: String,
    pub access_token: String,
    pub refresh_token: String,
    pub users_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn create_verification(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let rand: u8 = random();
    let reference: i32 = rand as i32;
    let mut small_rng = SmallRng::from_entropy();
    let code: i32 = small_rng.gen_range(999..999999);
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
                code: NULL_ALIAS_INT,
                email: payload.email,
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

pub async fn validate_verification(
    State(AppState { pool, client: _ }): State<AppState>,
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

pub async fn create_new_account(
    State(AppState { pool, client: _ }): State<AppState>,
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

pub async fn sign_in(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email;
    let password = payload.password;
    let fetch: Result<Option<(i32,)>, sqlx::Error> =
        sqlx::query_as("SELECT users_id FROM public.users WHERE (email = $1 AND password = $2)")
            .bind(&email)
            .bind(blake3::hash(password.as_bytes()).to_string())
            .fetch_optional(&pool)
            .await;
    if let Ok(ok) = fetch {
        if let Some((users_id,)) = ok {
            let date = Utc::now();
            let update =
                sqlx::query("UPDATE public.users SET latest_sign_in = $1 WHERE users_id = $2")
                    .bind(date)
                    .bind(users_id)
                    .execute(&pool)
                    .await;
            match update {
                Ok(_) => {
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
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn create_verification_forgot(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let email = payload.email;
    let fetch = sqlx::query("SELECT users_id FROM public.users WHERE email = $1")
        .bind(&email)
        .fetch_all(&pool)
        .await;
    if let Ok(rows) = fetch {
        if !rows.is_empty() {
            let rand: u8 = random();
            let reference: i32 = rand as i32;
            let mut small_rng = SmallRng::from_entropy();
            let code: i32 = small_rng.gen_range(999..999999);
            let expire: DateTime<Utc> = Utc::now() + Duration::minutes(MINUTES);
            let insert: Result<(i32,), sqlx::Error> = sqlx::query_as("INSERT INTO public.verification(reference, code, expire) VALUES ($1, $2, $3) RETURNING verification_id")
                .bind(reference)
                .bind(code)
                .bind(expire)
                .fetch_one(&pool)
                .await;
            if let Ok((verification_id,)) = insert {
                let sent = send_email(&email, reference, code);
                if let Ok(_) = sent {
                    Ok(Json(Authentication {
                        verification_id,
                        reference,
                        code: NULL_ALIAS_INT,
                        email,
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
}

pub async fn reset_password(
    State(AppState { pool, client: _ }): State<AppState>,
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
                let update: Result<(i32,), sqlx::Error> = sqlx::query_as("UPDATE public.users SET password = $1, latest_sign_in = $2 WHERE email = $3 RETURNING users_id")
                    .bind(blake3::hash(password.as_bytes()).to_string())
                    .bind(date)
                    .bind(&email)
                    .fetch_one(&pool)
                    .await;
                if let Ok((users_id,)) = update {
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
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            None => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn renew_token(
    Json(payload): Json<Authentication>,
) -> Result<Json<Authentication>, StatusCode> {
    let token = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(REFRESH_TOKEN_KEY.as_ref()),
        &Validation::default(),
    );
    if let Ok(TokenData { header: _, claims }) = token {
        let date = Utc::now();
        let access_claims = Claims {
            iat: date.timestamp() as usize,
            exp: (date + Duration::minutes(EXP_MIN)).timestamp() as usize,
            iss: ISSUER.to_string(),
            sub: claims.sub.to_string(),
        };
        let refresh_claims = Claims {
            iat: date.timestamp() as usize,
            exp: (date + Duration::days(EXP_DAY)).timestamp() as usize,
            iss: ISSUER.to_string(),
            sub: claims.sub.to_string(),
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
            email: NULL_ALIAS_STRING.to_string(),
            password: NULL_ALIAS_STRING.to_string(),
            access_token: access_token.unwrap(),
            refresh_token: refresh_token.unwrap(),
            users_id: NULL_ALIAS_INT,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn change_password(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    let update = sqlx::query("UPDATE public.users SET password = $1 WHERE users_id = $2")
        .bind(blake3::hash(payload.password.as_bytes()).to_string())
        .bind(payload.users_id)
        .execute(&pool)
        .await;
    match update {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn delete_account(
    State(AppState { pool, client: _ }): State<AppState>,
    Json(payload): Json<Authentication>,
) -> StatusCode {
    let delete: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as("DELETE FROM public.users WHERE (users_id = $1 AND email = $2 AND password = $3) RETURNING users_id")
        .bind(payload.users_id)
        .bind(payload.email)
        .bind(blake3::hash(payload.password.as_bytes()).to_string())
        .fetch_optional(&pool)
        .await;
    match delete {
        Ok(ok) => match ok {
            Some(_) => StatusCode::OK,
            None => StatusCode::BAD_REQUEST,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
