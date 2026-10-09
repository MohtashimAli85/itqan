mod delivery;

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use crate::db::Database;
use crate::domain::reminders;
use crate::error::AppError;

pub use delivery::resolve_reminder;

const MAX_SLEEP: Duration = Duration::from_secs(60);

pub struct Scheduler {
    wake: Arc<Notify>,
}

impl Scheduler {
    pub fn wake(&self) {
        self.wake.notify_one();
    }
}

pub fn start(app: &AppHandle) {
    let wake = Arc::new(Notify::new());
    app.manage(Scheduler { wake: wake.clone() });
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let pause = run_due(&app).unwrap_or_else(|error| {
                tracing::warn!(%error, "scheduler tick failed");
                MAX_SLEEP
            });
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
    let due = database.with(|connection| reminders::due(connection, now))?;
    for reminder in &due {
        database.with(|connection| reminders::mark_fired(connection, reminder, now))?;
        delivery::deliver(app, reminder)?;
    }
    let next = database.with(reminders::next_due_at)?;
    Ok(next
        .and_then(|next| (next - Utc::now()).to_std().ok())
        .map_or(MAX_SLEEP, |until| until.min(MAX_SLEEP)))
}
