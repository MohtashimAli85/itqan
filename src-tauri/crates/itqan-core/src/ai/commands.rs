use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};

use crate::ai::{self, ask, config, keys};
use crate::db::Database;
use crate::error::CommandError;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub settings: config::AiSettings,
    pub has_key: bool,
    pub needs_key: bool,
}

fn status(settings: config::AiSettings) -> Result<AiStatus, CommandError> {
    Ok(AiStatus {
        has_key: keys::get(settings.preset)?.is_some(),
        needs_key: settings.preset.defaults().needs_key,
        settings,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_ai_status(database: State<Database>) -> Result<AiStatus, CommandError> {
    status(database.with(config::load)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_ai_preset_defaults(preset: config::Preset) -> config::AiSettings {
    let defaults = preset.defaults();
    config::AiSettings {
        enabled: true,
        preset,
        base_url: defaults.base_url.into(),
        fast_model: defaults.fast_model.into(),
        smart_model: defaults.smart_model.into(),
        local_fallback: false,
    }
}

#[tauri::command]
#[specta::specta]
pub fn save_ai_settings(
    database: State<Database>,
    settings: config::AiSettings,
) -> Result<AiStatus, CommandError> {
    database.with(|connection| config::save(connection, &settings))?;
    status(database.with(config::load)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_ai_key(database: State<Database>, key: String) -> Result<AiStatus, CommandError> {
    let settings = database.with(config::load)?;
    keys::set(settings.preset, &key)?;
    status(settings)
}

#[tauri::command]
#[specta::specta]
pub fn delete_ai_key(database: State<Database>) -> Result<AiStatus, CommandError> {
    let settings = database.with(config::load)?;
    keys::delete(settings.preset)?;
    status(settings)
}

#[tauri::command]
#[specta::specta]
pub async fn test_ai_connection(app: AppHandle) -> Result<(), CommandError> {
    Ok(ai::test_connection(&app).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn ask_itqan(app: AppHandle, question: String) -> Result<String, CommandError> {
    Ok(ask::ask(&app, &question).await?)
}
