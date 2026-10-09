use chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;

use super::Scheduler;
use crate::commands::events::TasksChanged;
use crate::db::Database;
use crate::domain::reminders::{self, Reminder, ReminderId, SNOOZE_MINUTES};
use crate::domain::tasks::{self, TaskStatus};
use crate::error::AppError;
use crate::overlay::{self, BubbleAction, BubbleOrigin, FollowMode, OrbState, OverlayStore};

pub const DONE: &str = "done";
pub const SNOOZE: &str = "snooze";

fn actions() -> Vec<BubbleAction> {
    vec![
        BubbleAction {
            id: DONE.into(),
            label: "Done".into(),
        },
        BubbleAction {
            id: SNOOZE.into(),
            label: format!("In {SNOOZE_MINUTES} min"),
        },
    ]
}

pub fn deliver(app: &AppHandle, reminder: &Reminder) -> Result<(), AppError> {
    let store = app.state::<OverlayStore>();
    let origin = BubbleOrigin::Reminder {
        id: reminder.id,
        critical: reminder.critical,
    };
    if store.has_origin(origin)? {
        return Ok(());
    }
    if store.snapshot()?.follow_mode == FollowMode::Hidden {
        notify(app, reminder)?;
        if !reminder.critical {
            return Ok(());
        }
    }
    tracing::info!(
        reminder = reminder.id,
        critical = reminder.critical,
        "reminder delivered"
    );
    let (_, mut snapshot) = store.show_bubble(reminder.title.clone(), actions(), Some(origin))?;
    if reminder.critical {
        snapshot = store.set_orb_state(OrbState::Critical)?;
    }
    overlay::publish(app, snapshot)
}

fn notify(app: &AppHandle, reminder: &Reminder) -> Result<(), AppError> {
    app.notification()
        .builder()
        .title("Itqan")
        .body(&reminder.title)
        .show()
        .map_err(|error| AppError::Platform(error.to_string()))
}

pub fn resolve_reminder(
    app: &AppHandle,
    id: ReminderId,
    action: Option<&str>,
) -> Result<(), AppError> {
    let database = app.state::<Database>();
    let now = Utc::now();
    match action {
        Some(SNOOZE) => {
            database.with(|connection| reminders::snooze(connection, id, now))?;
            app.state::<Scheduler>().wake();
        }
        Some(DONE) => {
            let reminder = database.with(|connection| reminders::get(connection, id))?;
            if let Some(task_id) = reminder.task_id {
                database.with(|connection| {
                    tasks::set_status(connection, task_id, TaskStatus::Done, now)
                })?;
                TasksChanged.emit(app)?;
            }
        }
        _ => {}
    }
    let store = app.state::<OverlayStore>();
    if store.snapshot()?.orb_state == OrbState::Critical && !store.has_critical()? {
        overlay::publish(app, store.restore_base()?)?;
    }
    Ok(())
}
