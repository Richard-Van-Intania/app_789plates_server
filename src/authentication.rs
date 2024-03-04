use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Debug, Serialize)]
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
