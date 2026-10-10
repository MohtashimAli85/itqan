#![allow(clippy::needless_pass_by_value)]

use tauri::{AppHandle, State};

use crate::app_info::{self, AppInfo};
use crate::db::Database;
use crate::error::CommandError;

#[tauri::command]
#[specta::specta]
pub fn get_app_info(app: AppHandle, database: State<Database>) -> Result<AppInfo, CommandError> {
    let version = app.package_info().version.to_string();
    Ok(database.with(|connection| app_info::load(connection, &version))?)
}
