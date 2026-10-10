use tauri::{AppHandle, State};

use crate::domain::app_info::{self, AppInfo};
use itqan_core::db::Database;
use itqan_core::error::CommandError;

#[tauri::command]
#[specta::specta]
pub fn get_app_info(app: AppHandle, database: State<Database>) -> Result<AppInfo, CommandError> {
    let version = app.package_info().version.to_string();
    Ok(database.with(|connection| app_info::load(connection, &version))?)
}
