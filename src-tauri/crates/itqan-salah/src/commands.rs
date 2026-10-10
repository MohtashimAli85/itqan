use chrono::Utc;
use itqan_contracts::PrayerWindow;
use itqan_core::db::Database;
use itqan_core::error::{AppError, CommandError};
use itqan_core::{scheduler, settings};
use tauri::{AppHandle, State};

use crate::repo as prayer_repo;
use crate::schedule;
use crate::settings::PrayerSettings;

#[tauri::command]
#[specta::specta]
pub fn get_prayer_settings(database: State<Database>) -> Result<PrayerSettings, CommandError> {
    Ok(database.with(prayer_repo::get)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_prayer_settings(
    app: AppHandle,
    database: State<Database>,
    settings: PrayerSettings,
) -> Result<PrayerSettings, CommandError> {
    validate(&settings)?;
    let saved = database.with(|connection| {
        prayer_repo::save(connection, &settings)?;
        prayer_repo::get(connection)
    })?;
    scheduler::refresh(&app)?;
    Ok(saved)
}

#[tauri::command]
#[specta::specta]
pub fn get_prayer_day(database: State<Database>) -> Result<Vec<PrayerWindow>, CommandError> {
    Ok(database.with(|connection| {
        let prayer = prayer_repo::get(connection)?;
        let timezone = settings::timezone(connection)?;
        let today = Utc::now().with_timezone(&timezone).date_naive();
        Ok(schedule::windows_on(&prayer, today))
    })?)
}

fn validate(settings: &PrayerSettings) -> Result<(), AppError> {
    let latitude_ok = settings
        .latitude
        .is_none_or(|value| (-90.0..=90.0).contains(&value));
    let longitude_ok = settings
        .longitude
        .is_none_or(|value| (-180.0..=180.0).contains(&value));
    if !latitude_ok || !longitude_ok {
        return Err(AppError::InvalidInput(
            "coordinates are out of range".into(),
        ));
    }
    if settings.enabled && (settings.latitude.is_none() || settings.longitude.is_none()) {
        return Err(AppError::InvalidInput(
            "choose a city before turning on prayer times".into(),
        ));
    }
    if settings.pause_before_minutes > 60 || settings.pause_after_minutes > 90 {
        return Err(AppError::InvalidInput("pause windows are too long".into()));
    }
    Ok(())
}
