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

impl AppError {
    /// An error from the local AI engine, reworded so that no engine or model
    /// name, address or HTTP detail ever reaches the user.
    pub fn ai(msg: impl AsRef<str>) -> Self {
        let raw = msg.as_ref();
        let lower = raw.to_lowercase();
        let friendly = if ["connect", "refused", "connection", "dns", "timed out", "timeout"].iter().any(|w| lower.contains(w)) {
            "Errandly’s AI engine isn’t responding. Quit and reopen Errandly, then try again."
        } else if lower.contains("not found") || lower.contains("no such") || lower.contains("pull") {
            "Errandly’s models aren’t set up yet. Open Settings → Models and choose Refresh."
        } else if lower.contains("memory") || lower.contains("oom") || lower.contains("resource") {
            "Your Mac is short on memory right now. Close a few apps and try again."
        } else if lower.contains("space") || lower.contains("disk") {
            "There isn’t enough free disk space. Free up a few GB and try again."
        } else {
            "Errandly’s AI had trouble with that. Please try again."
        };
        AppError::Ai(friendly.to_string())
    }
}

// Commands return errors to the UI as plain messages.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
