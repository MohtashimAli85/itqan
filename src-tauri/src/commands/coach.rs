use tauri::State;

use crate::agents::coach;
use itqan_core::db::Database;
use itqan_core::error::CommandError;
use itqan_core::nudges::{self, Nudge};

#[tauri::command]
#[specta::specta]
pub fn get_nudge_budget(database: State<Database>) -> Result<u32, CommandError> {
    Ok(database.with(coach::budget)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_nudge_budget(database: State<Database>, per_hour: u32) -> Result<u32, CommandError> {
    Ok(database.with(|connection| coach::set_budget(connection, per_hour))?)
}

#[tauri::command]
#[specta::specta]
pub fn list_recent_nudges(
    database: State<Database>,
    limit: u32,
) -> Result<Vec<Nudge>, CommandError> {
    Ok(database.with(|connection| nudges::recent(connection, limit.min(200)))?)
}
