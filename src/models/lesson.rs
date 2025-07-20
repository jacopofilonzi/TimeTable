use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Lesson {
    pub starts_at: String,
    pub ends_at: String,
    pub subject: String,
    pub teacher: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}
