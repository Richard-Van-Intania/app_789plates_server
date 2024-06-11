use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hashtag {
    pub hashtag_id: i32,
    pub tag: String,
    pub add_date: String,
}
