use chrono::Utc;
use tauri::State;

use crate::agent as rewards;
use crate::rewards::{self as progress, ProgressSummary};
use itqan_core::db::Database;
use itqan_core::error::CommandError;
use itqan_core::ports::Ports;
use itqan_core::settings;

const PROGRESS_DAYS: u32 = 35;

#[tauri::command]
#[specta::specta]
pub fn get_progress(
    database: State<Database>,
    ports: State<Ports>,
) -> Result<ProgressSummary, CommandError> {
    Ok(database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        progress::summary(connection, &ports, Utc::now(), timezone, PROGRESS_DAYS)
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
