use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailRes {
    pub uuid: String,
    pub email: String,
    pub reference: usize,
}

#[derive(Deserialize)]
pub struct VerifyCode {
    pub uuid: String,
    pub reference: usize,
    pub code: String,
}
