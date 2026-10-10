mod delivery;
pub mod modes;

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use crate::agents::{self, coach, AppEvent};
use crate::tray;
use itqan_core::db::Database;
use itqan_core::error::AppError;
use itqan_core::ports::Ports;
use itqan_core::reminders;
use itqan_core::scheduler::Scheduler;

pub use delivery::resolve_reminder;
pub use modes::{ModeChanged, ModeEngine, ModeStatus};

const MAX_SLEEP: Duration = Duration::from_secs(60);

pub fn start(app: &AppHandle) {
    let wake = Arc::new(Notify::new());
    app.manage(Scheduler::new(wake.clone()));
    app.manage(ModeEngine::default());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let reminders = run_due(&app).unwrap_or_else(|error| {
                tracing::warn!(%error, "scheduler tick failed");
                MAX_SLEEP
            });
            let modes = modes::evaluate(&app).map_or_else(
                |error| {
                    tracing::warn!(%error, "mode evaluation failed");
                    MAX_SLEEP
                },
                |(_, pause)| pause,
            );
            if let Err(error) = agents::publish(&app, AppEvent::Tick) {
                tracing::warn!(%error, "agent tick failed");
            }
            if let Err(error) = coach::tick(&app, Utc::now()) {
                tracing::warn!(%error, "coach tick failed");
            }
            if let Err(error) = tray::refresh(&app) {
                tracing::warn!(%error, "tray refresh failed");
            }
            let pause = reminders.min(modes);
            tokio::select! {
                () = tokio::time::sleep(pause) => {}
                () = wake.notified() => {}
            }
        }
    });
}

fn run_due(app: &AppHandle) -> Result<Duration, AppError> {
    let database = app.state::<Database>();
    let now = Utc::now();
    let ports = app.state::<Ports>();
    let due = database.with(|connection| reminders::due(connection, &ports, now))?;
    let hold = modes::current(app)?.and_then(|status| {
        status
            .active_prayer
            .map(|prayer| prayer.pause_until)
            .or(status.focus.map(|focus| focus.ends_at))
    });
    for reminder in &due {
        if let (Some(until), false) = (hold, reminder.critical) {
            database
                .with(|connection| reminders::hold_until(connection, reminder.id, until, now))?;
            continue;
        }
        database.with(|connection| reminders::mark_fired(connection, reminder, now))?;
        delivery::deliver(app, reminder)?;
    }
    let next = database.with(|connection| reminders::next_due_at(connection, &ports))?;
    Ok(next
        .and_then(|next| (next - Utc::now()).to_std().ok())
        .map_or(MAX_SLEEP, |until| until.min(MAX_SLEEP)))
}

pub fn refresh(app: &AppHandle) -> Result<ModeStatus, AppError> {
    let (status, _) = modes::evaluate(app)?;
    tray::refresh(app)?;
    app.state::<Scheduler>().wake();
    Ok(status)
}
