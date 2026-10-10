use tauri::State;

use itqan_core::db::Database;
use itqan_core::error::CommandError;
use itqan_core::settings;

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
