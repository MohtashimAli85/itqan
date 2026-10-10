#![allow(clippy::needless_pass_by_value)]

use tauri::State;

use crate::db::Database;
use crate::error::CommandError;
use crate::settings;

#[tauri::command]
#[specta::specta]
pub fn get_timezone(database: State<Database>) -> Result<String, CommandError> {
    Ok(database.with(|connection| Ok(settings::timezone(connection)?.name().to_owned()))?)
}

#[tauri::command]
#[specta::specta]
pub fn set_timezone(database: State<Database>, timezone: String) -> Result<String, CommandError> {
    Ok(database.with(|connection| {
        Ok(settings::set_timezone(connection, &timezone)?
            .name()
            .to_owned())
    })?)
}
