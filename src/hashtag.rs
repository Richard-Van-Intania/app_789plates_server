use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hashtag {
    pub hashtag_id: i32,
    pub tag: String,
    pub add_date: String,
}

// add_new_hashtag

// add_hashtag_to_plates
