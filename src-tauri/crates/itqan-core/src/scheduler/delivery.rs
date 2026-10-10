use chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::db::Database;
use crate::error::AppError;
use crate::overlay::action;
use crate::overlay::{self, BubbleAction, BubbleOrigin, FollowMode, OrbState, OverlayStore};
use crate::ports::Ports;
use crate::reminders::{self, Reminder, ReminderId, SNOOZE_MINUTES};
use crate::scheduler::Scheduler;

pub const DONE: &str = "done";
pub const SNOOZE: &str = "snooze";

fn actions() -> Vec<BubbleAction> {
    vec![
        action(DONE, "Done"),
        action(SNOOZE, &format!("In {SNOOZE_MINUTES} min")),
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
            let ports = app.state::<Ports>();
            database.with(|connection| reminders::snooze(connection, &ports, id, now))?;
            app.state::<Scheduler>().wake();
        }
        Some(DONE) => {
            if let Some((kind, target)) =
                database.with(|connection| reminders::target_of(connection, id))?
            {
                if let Some(owner) = app.state::<Ports>().reminder_target(kind)? {
                    owner.complete(app, target)?;
                }
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
