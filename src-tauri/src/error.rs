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
    #[error("database lock poisoned")]
    LockPoisoned,
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
