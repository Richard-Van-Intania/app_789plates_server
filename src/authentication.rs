use serde::{Deserialize, Serialize};

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
