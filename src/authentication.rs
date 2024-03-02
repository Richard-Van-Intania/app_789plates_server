use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyRef {
    pub uuid: String,
    pub email: String,
    pub reference: u8,
}

#[derive(Deserialize)]
pub struct VerifyCode {
    pub uuid: String,
    pub reference: u8,
    pub code: String,
}
