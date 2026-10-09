use tauri::{AppHandle, State};

use crate::db::Database;
use crate::domain::app_info::{self, AppInfo};
use crate::error::CommandError;

#[tauri::command]
#[specta::specta]
pub fn get_app_info(app: AppHandle, database: State<Database>) -> Result<AppInfo, CommandError> {
    let version = app.package_info().version.to_string();
    Ok(database.with(|connection| app_info::load(connection, &version))?)
}
