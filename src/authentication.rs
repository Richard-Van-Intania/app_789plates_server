use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Email {
    pub email: String,
}
