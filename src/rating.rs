use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Rating {
    pub rating_id: i32,
    pub users_id: i32,
    pub store_id: i32,
    pub score: f64,
    pub review: String,
    pub add_date: String,
}

// fetch_rating
// add_new_rating
