use keyring::{Entry, Error};

use super::config::Preset;
use crate::error::AppError;

const SERVICE: &str = "dev.itqan.desktop";

fn entry(preset: Preset) -> Result<Entry, AppError> {
    Entry::new(SERVICE, preset.key_account()).map_err(|error| AppError::Platform(error.to_string()))
}

pub fn get(preset: Preset) -> Result<Option<String>, AppError> {
    match entry(preset)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(Error::NoEntry) => Ok(None),
        Err(error) => Err(AppError::Platform(error.to_string())),
    }
}

pub fn set(preset: Preset, key: &str) -> Result<(), AppError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(AppError::InvalidInput("the key is empty".into()));
    }
    entry(preset)?
        .set_password(key)
        .map_err(|error| AppError::Platform(error.to_string()))
}

pub fn delete(preset: Preset) -> Result<(), AppError> {
    match entry(preset)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(error) => Err(AppError::Platform(error.to_string())),
    }
}
