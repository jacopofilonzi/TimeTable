use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub code: String,
    pub name: String,
    pub category: String,
}
