use chrono::Utc;
use tauri::State;

use crate::agents::rewards;
use crate::db::Database;
use crate::domain::rewards::{self as progress, ProgressSummary};
use crate::domain::settings;
use crate::error::CommandError;

const PROGRESS_DAYS: u32 = 35;

#[tauri::command]
#[specta::specta]
pub fn get_progress(database: State<Database>) -> Result<ProgressSummary, CommandError> {
    Ok(database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        progress::summary(connection, Utc::now(), timezone, PROGRESS_DAYS)
    })?)
}

#[tauri::command]
#[specta::specta]
pub fn get_reward_sound(database: State<Database>) -> Result<bool, CommandError> {
    Ok(database.with(rewards::sound_enabled)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_reward_sound(database: State<Database>, enabled: bool) -> Result<(), CommandError> {
    Ok(database.with(|connection| rewards::set_sound(connection, enabled))?)
}
