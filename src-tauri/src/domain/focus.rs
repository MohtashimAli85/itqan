use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::focus as repo;
use crate::domain::tasks::TaskId;
use crate::error::AppError;
use crate::prayer::schedule::{self, PrayerWindow};

pub type FocusSessionId = i32;

pub const DEFAULT_MINUTES: u16 = 25;
const MAX_MINUTES: u16 = 240;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FocusSession {
    pub id: FocusSessionId,
    pub task_id: Option<TaskId>,
    pub started_at: DateTime<Utc>,
    pub planned_minutes: u16,
    pub ended_at: Option<DateTime<Utc>>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FocusStatus {
    pub session: FocusSession,
    pub progress: f32,
    pub ends_at: DateTime<Utc>,
    pub paused_for_prayer: bool,
}

impl FocusStatus {
    pub fn is_finished(&self) -> bool {
        self.progress >= 1.0
    }
}

pub fn active(connection: &Connection) -> Result<Option<FocusSession>, AppError> {
    repo::active(connection)
}

pub fn start(
    connection: &Connection,
    minutes: u16,
    task_id: Option<TaskId>,
    now: DateTime<Utc>,
) -> Result<FocusSession, AppError> {
    if !(1..=MAX_MINUTES).contains(&minutes) {
        return Err(AppError::InvalidInput(format!(
            "focus lasts 1 to {MAX_MINUTES} minutes"
        )));
    }
    if repo::active(connection)?.is_some() {
        return Err(AppError::InvalidInput(
            "a focus session is already running".into(),
        ));
    }
    let id = repo::insert(connection, task_id, minutes, now)?;
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("focus session {id}")))
}

pub fn finish(
    connection: &Connection,
    id: FocusSessionId,
    now: DateTime<Utc>,
    completed: bool,
) -> Result<(), AppError> {
    repo::finish(connection, id, now, completed)
}

pub fn status(session: FocusSession, now: DateTime<Utc>, windows: &[PrayerWindow]) -> FocusStatus {
    let planned = Duration::minutes(i64::from(session.planned_minutes));
    let paused = schedule::paused_overlap(windows, session.started_at, now);
    let elapsed = (now - session.started_at - paused).max(Duration::zero());
    let remaining = (planned - elapsed).max(Duration::zero());
    let paused_for_prayer = windows
        .iter()
        .any(|window| window.pause_from <= now && now < window.pause_until);
    let progress = (elapsed.num_milliseconds() as f64 / planned.num_milliseconds() as f64) as f32;
    FocusStatus {
        session,
        progress: progress.clamp(0.0, 1.0),
        ends_at: now + remaining,
        paused_for_prayer,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;
    use crate::prayer::schedule::Prayer;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, hour, minute, 0).unwrap()
    }

    #[test]
    fn only_one_session_runs_at_a_time() {
        let connection = test_connection();
        let session = start(&connection, 25, None, at(10, 0)).unwrap();
        assert!(start(&connection, 25, None, at(10, 1)).is_err());

        finish(&connection, session.id, at(10, 25), true).unwrap();
        assert!(active(&connection).unwrap().is_none());
        assert!(start(&connection, 0, None, at(11, 0)).is_err());
    }

    #[test]
    fn progress_pauses_during_prayer() {
        let connection = test_connection();
        let session = start(&connection, 30, None, at(10, 0)).unwrap();
        let asr = PrayerWindow {
            prayer: Prayer::Asr,
            at: at(10, 15),
            pause_from: at(10, 10),
            pause_until: at(10, 30),
        };

        let during = status(session.clone(), at(10, 20), &[asr]);
        assert!(during.paused_for_prayer);
        assert!((during.progress - 1.0 / 3.0).abs() < 0.001);

        let after = status(session.clone(), at(10, 40), &[asr]);
        assert!((after.progress - 2.0 / 3.0).abs() < 0.001);
        assert_eq!(after.ends_at, at(10, 50));

        assert!(status(session, at(10, 50), &[asr]).is_finished());
    }
}
