use serde::Serialize;
use specta::Type;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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
