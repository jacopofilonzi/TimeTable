use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub error: String,
    pub message: Option<String>,
    pub fault: ErrorFault,
    pub http_code: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorFault {
    User,
    Internal,
    External,
}
