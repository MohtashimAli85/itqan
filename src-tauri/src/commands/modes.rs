use chrono::{Duration, Utc};
use tauri::{AppHandle, State};

use crate::db::{prayer as prayer_repo, Database};
use crate::domain::focus;
use crate::domain::modes::{self, WorkDay};
use crate::domain::tasks::TaskId;
use crate::prayer::schedule::{self, PrayerWindow};
use crate::prayer::settings::PrayerSettings;
use crate::scheduler::{self, ModeStatus};
use itqan_core::error::{AppError, CommandError};
use itqan_core::settings;

#[tauri::command]
#[specta::specta]
pub fn get_mode_status(app: AppHandle) -> Result<ModeStatus, CommandError> {
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_rest(
    app: AppHandle,
    database: State<Database>,
    minutes: Option<u16>,
) -> Result<ModeStatus, CommandError> {
    let until = minutes.map(|minutes| Utc::now() + Duration::minutes(i64::from(minutes)));
    database.with(|connection| modes::set_rest_until(connection, until))?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn start_focus(
    app: AppHandle,
    database: State<Database>,
    minutes: u16,
    task_id: Option<TaskId>,
) -> Result<ModeStatus, CommandError> {
    database.with(|connection| {
        modes::set_rest_until(connection, None)?;
        focus::start(connection, minutes, task_id, Utc::now())
    })?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn stop_focus(app: AppHandle, database: State<Database>) -> Result<ModeStatus, CommandError> {
    database.with(|connection| match focus::active(connection)? {
        Some(session) => focus::finish(connection, session.id, Utc::now(), false),
        None => Ok(()),
    })?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_work_hours(database: State<Database>) -> Result<Vec<WorkDay>, CommandError> {
    Ok(database.with(modes::work_hours)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_work_hours(
    app: AppHandle,
    database: State<Database>,
    days: Vec<WorkDay>,
) -> Result<Vec<WorkDay>, CommandError> {
    let days = database.with(|connection| modes::set_work_hours(connection, days))?;
    scheduler::refresh(&app)?;
    Ok(days)
}

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
