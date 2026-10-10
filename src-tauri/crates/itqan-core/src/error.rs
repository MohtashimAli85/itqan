use serde::Serialize;
use specta::Type;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("state lock poisoned")]
    LockPoisoned,
    #[error("{0} not found")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("window not found: {0}")]
    WindowMissing(&'static str),
    #[error(transparent)]
    Ai(#[from] crate::ai::client::AiError),
    #[error("platform error: {0}")]
    Platform(String),
}

#[derive(Debug, Serialize, Type)]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}
