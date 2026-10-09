use chrono::{NaiveTime, TimeZone, Utc};
use tauri::{AppHandle, State};
use tauri_specta::Event;

use super::events::HealthChanged;
use crate::agents::{self, AppEvent};
use crate::db::Database;
use crate::domain::health::{self, HabitId, HabitKind, HealthOverview, Medicine};
use crate::domain::settings;
use crate::error::{AppError, CommandError};
use crate::scheduler::Scheduler;

fn changed<T>(app: &AppHandle, value: T) -> Result<T, CommandError> {
    HealthChanged.emit(app).map_err(AppError::from)?;
    Ok(value)
}

#[tauri::command]
#[specta::specta]
pub fn get_health_overview(database: State<Database>) -> Result<HealthOverview, CommandError> {
    Ok(database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        let today = Utc::now().with_timezone(&timezone).date_naive();
        let start = timezone
            .from_local_datetime(&today.and_time(NaiveTime::MIN))
            .earliest()
            .map_or_else(Utc::now, |start| start.with_timezone(&Utc));
        health::overview(connection, start)
    })?)
}

#[tauri::command]
#[specta::specta]
pub fn set_health_enabled(
    app: AppHandle,
    database: State<Database>,
    enabled: bool,
) -> Result<(), CommandError> {
    database.with(|connection| health::set_enabled(connection, enabled))?;
    changed(&app, ())
}

#[tauri::command]
#[specta::specta]
pub fn log_habit(
    app: AppHandle,
    database: State<Database>,
    kind: HabitKind,
) -> Result<(), CommandError> {
    database.with(|connection| health::log(connection, kind, Utc::now()))?;
    agents::publish(&app, AppEvent::HabitLogged { kind })?;
    changed(&app, ())
}

#[tauri::command]
#[specta::specta]
pub fn set_water_target(
    app: AppHandle,
    database: State<Database>,
    target: u16,
) -> Result<(), CommandError> {
    database.with(|connection| health::set_water_target(connection, target))?;
    changed(&app, ())
}

#[tauri::command]
#[specta::specta]
pub fn add_medicine(
    app: AppHandle,
    database: State<Database>,
    scheduler: State<Scheduler>,
    name: String,
    times: Vec<String>,
) -> Result<Medicine, CommandError> {
    let parsed = times
        .iter()
        .map(|time| {
            NaiveTime::parse_from_str(time, "%H:%M")
                .map_err(|_| AppError::InvalidInput(format!("{time} is not a time like 09:00")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let medicine = database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        health::add_medicine(connection, &name, &parsed, timezone, Utc::now())
    })?;
    scheduler.wake();
    changed(&app, medicine)
}

#[tauri::command]
#[specta::specta]
pub fn delete_medicine(
    app: AppHandle,
    database: State<Database>,
    scheduler: State<Scheduler>,
    id: HabitId,
) -> Result<(), CommandError> {
    database.with(|connection| health::delete_medicine(connection, id))?;
    scheduler.wake();
    changed(&app, ())
}
