use rusqlite::Connection;
use serde::Serialize;
use specta::Type;

use crate::db::migrations;
use crate::error::AppError;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub schema_version: u32,
}

pub fn load(connection: &Connection, version: &str) -> Result<AppInfo, AppError> {
    Ok(AppInfo {
        name: "Itqan".to_owned(),
        version: version.to_owned(),
        schema_version: migrations::schema_version(connection)?,
    })
}
