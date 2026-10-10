#![allow(clippy::needless_pass_by_value)]

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use super::{set_enabled, ModuleState, Modules};
use crate::error::{AppError, CommandError};
use crate::scheduler;

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct ModulesChanged;

#[tauri::command]
#[specta::specta]
pub fn list_modules(modules: State<Modules>) -> Result<Vec<ModuleState>, CommandError> {
    Ok(modules.states()?)
}

#[tauri::command]
#[specta::specta]
pub fn set_module_enabled(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> Result<Vec<ModuleState>, CommandError> {
    let result = set_enabled(&app, &id, enabled);
    ModulesChanged.emit(&app).map_err(AppError::from)?;
    result?;
    if let Err(error) = scheduler::refresh(&app) {
        tracing::warn!(%error, "refresh after a module change failed");
    }
    Ok(app.state::<Modules>().states()?)
}
