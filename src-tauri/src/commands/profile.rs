use chrono::Utc;
use tauri::{AppHandle, Manager, State};

use crate::db::Database;
use crate::domain::profile::{self, Profile};
use crate::error::CommandError;
use crate::overlay::{self, OverlayStore};

#[tauri::command]
#[specta::specta]
pub fn get_profile(database: State<Database>) -> Result<Profile, CommandError> {
    Ok(database.with(profile::get)?)
}

#[tauri::command]
#[specta::specta]
pub fn save_profile(database: State<Database>, profile: Profile) -> Result<Profile, CommandError> {
    Ok(database.with(|connection| profile::save(connection, profile, Utc::now()))?)
}

#[tauri::command]
#[specta::specta]
pub fn is_onboarded(database: State<Database>) -> Result<bool, CommandError> {
    Ok(database.with(profile::is_onboarded)?)
}

#[tauri::command]
#[specta::specta]
pub fn complete_onboarding(app: AppHandle, database: State<Database>) -> Result<(), CommandError> {
    let name = database.with(|connection| {
        profile::complete_onboarding(connection)?;
        Ok(profile::get(connection)?.name)
    })?;
    let greeting = match name {
        Some(name) => {
            format!("Assalamu alaikum, {name}. I'm here when you need me. Click me any time.")
        }
        None => "Assalamu alaikum. I'm here when you need me. Click me any time.".to_owned(),
    };
    let store = app.state::<OverlayStore>();
    let (_, snapshot) = store.show_bubble(greeting, Vec::new(), None)?;
    overlay::publish(&app, snapshot)?;
    Ok(())
}
