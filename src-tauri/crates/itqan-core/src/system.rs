use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::coach;
use crate::db::Database;
use crate::error::{AppError, CommandError};
use crate::tray;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CoachSettings {
    pub budget_per_hour: u32,
    pub paused_until: Option<DateTime<Utc>>,
}

#[tauri::command]
#[specta::specta]
pub fn get_autostart(app: AppHandle) -> Result<bool, CommandError> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| AppError::Platform(error.to_string()).into())
}

#[tauri::command]
#[specta::specta]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, CommandError> {
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|error| AppError::Platform(error.to_string()))?;
    get_autostart(app)
}

#[tauri::command]
#[specta::specta]
pub fn get_coach_settings(database: State<Database>) -> Result<CoachSettings, CommandError> {
    Ok(database.with(|connection| {
        Ok(CoachSettings {
            budget_per_hour: coach::budget(connection)?,
            paused_until: coach::paused_until(connection)?.filter(|until| *until > Utc::now()),
        })
    })?)
}

#[tauri::command]
#[specta::specta]
pub fn set_nudges_paused(
    app: AppHandle,
    database: State<Database>,
    minutes: Option<u16>,
) -> Result<CoachSettings, CommandError> {
    let until = minutes.map(|minutes| Utc::now() + Duration::minutes(i64::from(minutes)));
    database.with(|connection| coach::set_paused_until(connection, until))?;
    tray::refresh(&app)?;
    get_coach_settings(database)
}
