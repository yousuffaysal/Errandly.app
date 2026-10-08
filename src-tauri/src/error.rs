use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("file system error: {0}")]
    Io(#[from] std::io::Error),
    #[error("local AI error: {0}")]
    Ai(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("invalid plan: {0}")]
    InvalidPlan(String),
    #[error("{0}")]
    Invalid(String),
    #[error("stopped")]
    Cancelled,
    #[error("not found: {0}")]
    NotFound(String),
}

// Commands return errors to the UI as plain messages.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
