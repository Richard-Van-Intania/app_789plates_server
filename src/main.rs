use std::time;

use app_789plates_server::{
    authentication::{CreateNewAccount, Email, SignIn, VerificationCode, VerificationRes},
    graceful_shutdown::shutdown_signal,
    jwt::{verify_signature, Claims, Token, ACCESS_TOKEN_KEY, ISSUER, REFRESH_TOKEN_KEY},
    mailer::{send_email, MINUTES},
};
use axum::{
    extract::{Request, State},
    handler::Handler,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Duration, Local, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{random, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use sqlx::{FromRow, PgPool};
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/app789plates")
        .await
        .unwrap();
    let app = Router::new()
        .route("/", get(|| async {}))
        .route("/checkavailabilityemail", post(check_availability_email))
        .route("/checkverificationcode", post(check_verification_code))
        .route("/createnewaccount", post(create_new_account))
        .route("/signin", get(sign_in))
        .route("/forgotpassword", post(forgot_password))
        .route("/resetpassword", put(reset_password))
        .route("/addsecondaryemail", post(add_secondary_email))
        .route("/deleteaccount", delete(delete_account))
        .route("/editname", put(edit_name))
        .route("/editprofilepicture", put(edit_profile_picture))
        .route("/editinformation", put(edit_information))
        .route(
            "/search",
            get(search.layer(middleware::from_fn(verify_signature))),
        )
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(time::Duration::from_secs(30)))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8700").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn root() -> Result<impl IntoResponse, StatusCode> {
    Ok(())
}

async fn check_availability_email(
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

async fn check_verification_code(
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

async fn create_new_account(
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

async fn sign_in(
    State(pool): State<PgPool>,
    Json(payload): Json<SignIn>,
) -> Result<Json<Token>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let valid = EmailAddress::is_valid(&email);
    if valid {
        let fetch_email: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
            "SELECT users_id FROM public.users WHERE primary_email = $1 OR secondary_email = $2",
        )
        .bind(&email)
        .bind(&email)
        .fetch_optional(&pool)
        .await;
        if let Ok(row) = fetch_email {
            match row {
                Some((users_id,)) => {
                    let sign_in: Result<Option<(i32,)>, sqlx::Error> = sqlx::query_as(
                        "SELECT users_id FROM public.users WHERE users_id = $1 AND password = $2",
                    )
                    .bind(users_id)
                    .bind(blake3::hash(payload.password.as_bytes()).to_string())
                    .fetch_optional(&pool)
                    .await;
                    if let Ok(opt) = sign_in {
                        match opt {
                            Some((users_id,)) => {
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
                            }
                            None => Err(StatusCode::UNAUTHORIZED),
                        }
                    } else {
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
                None => Err(StatusCode::BAD_REQUEST),
            }
        } else {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// user
async fn forgot_password() -> impl IntoResponse {}
async fn edit_name() -> impl IntoResponse {}
async fn reset_password() -> impl IntoResponse {}
async fn add_secondary_email() -> impl IntoResponse {}
async fn delete_account() -> impl IntoResponse {}
async fn edit_profile_picture() -> impl IntoResponse {}
async fn edit_information() -> impl IntoResponse {}
async fn search() -> impl IntoResponse {}

//  plates
// add edit delete transfer show hide

// search search with condition

// home feed fetch timeline

// like plate, like profile
