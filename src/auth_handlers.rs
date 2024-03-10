use crate::{
    authentication::{
        AddSecondaryEmail, ChangePassword, CreateNewAccount, Email, SignIn, VerificationCode,
        VerificationRes,
    },
    jwt::{Claims, Token, ACCESS_TOKEN_KEY, ISSUER, REFRESH_TOKEN_KEY},
    mailer::{send_email, MINUTES},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::PgPool;

pub async fn check_availability_email(
    State(pool): State<PgPool>,
    Json(payload): Json<Email>,
) -> Result<Json<VerificationRes>, StatusCode> {
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
                        Ok(Json(VerificationRes {
                            verification_id,
                            email,
                            reference,
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
    Json(payload): Json<VerificationCode>,
) -> StatusCode {
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

pub async fn create_new_account(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNewAccount>,
) -> Result<Json<Token>, StatusCode> {
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
        match fetch_email {
            Ok(rows) => {
                if rows.is_empty() {
                    let fetch_code: Result<Option<(i32,)>,  sqlx::Error> = sqlx::query_as(
                        "SELECT verification_id FROM public.verification WHERE verification_id = $1 AND reference = $2 AND code = $3 AND verified = true",
                    )
                    .bind(payload.verification_id)
                    .bind(payload.reference)
                    .bind(payload.code)
                    .fetch_optional(&pool)
                    .await;
                    match fetch_code {
                        Ok(_) => {
                            let date = Utc::now();
                            let insert_user: Result<(i32,), sqlx::Error> = sqlx::query_as(
                                "INSERT INTO public.users (name, primary_email, password, created_date) VALUES ($1, $2, $3, $4) RETURNING users_id",
                            )
                            .bind(*email.split("@").collect::<Vec<&str>>().get(0).unwrap())
                            .bind(&email)
                            .bind(blake3::hash(payload.password.as_bytes()).to_string())
                            .bind(date)
                            .fetch_one(&pool)
                            .await;
                            match insert_user {
                                Ok((users_id,)) => {
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
                                    let token = Token {
                                        access_token: access_token.unwrap(),
                                        refresh_token: refresh_token.unwrap(),
                                    };
                                    Ok(Json(token))
                                }
                                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                            }
                        }
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
    Json(payload): Json<SignIn>,
) -> Result<Json<Token>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    let password = payload.password;
    if valid && !password.is_empty() {
        let fetch_users_id: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
            "SELECT users_id FROM public.users WHERE (primary_email = $1 OR secondary_email = $2) AND password = $3",
        )
        .bind(&email)
        .bind(&email)
        .bind(blake3::hash(password.as_bytes()).to_string())
        .fetch_optional(&pool)
        .await;
        if let Ok(opt_users_id) = fetch_users_id {
            if let Some((users_id,)) = opt_users_id {
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
                let token = Token {
                    access_token: access_token.unwrap(),
                    refresh_token: refresh_token.unwrap(),
                };
                Ok(Json(token))
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
    Json(payload): Json<Email>,
) -> Result<Json<VerificationRes>, StatusCode> {
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
                        Ok(Json(VerificationRes {
                            verification_id,
                            email,
                            reference,
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
    Json(payload): Json<CreateNewAccount>,
) -> Result<Json<Token>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    let password = payload.password;
    if valid && !password.is_empty() {
        let date = Utc::now();
        let update_code: Result<Option<(bool,)>, sqlx::Error> = sqlx::query_as(
            "UPDATE public.verification
        SET verified = true
        WHERE verification_id = $1 AND reference = $2 AND code = $3 AND expire > $4
        RETURNING verified",
        )
        .bind(payload.verification_id)
        .bind(payload.reference)
        .bind(payload.code)
        .bind(date)
        .fetch_optional(&pool)
        .await;
        if let Ok(opt_verified) = update_code {
            if let Some((verified,)) = opt_verified {
                if verified {
                    let update_password: Result<(i32,), sqlx::Error> = sqlx::query_as(
                        "UPDATE public.users
                    SET password = $1
                    WHERE primary_email = $2 OR secondary_email = $3
                    RETURNING users_id",
                    )
                    .bind(blake3::hash(password.as_bytes()).to_string())
                    .bind(&email)
                    .bind(&email)
                    .fetch_one(&pool)
                    .await;
                    if let Ok((users_id,)) = update_password {
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
                        let token = Token {
                            access_token: access_token.unwrap(),
                            refresh_token: refresh_token.unwrap(),
                        };
                        Ok(Json(token))
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                } else {
                    Err(StatusCode::BAD_REQUEST)
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
            // check and update
            StatusCode::OK
        } else {
            StatusCode::UNAUTHORIZED
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn delete_account() -> StatusCode {
    StatusCode::OK
}
