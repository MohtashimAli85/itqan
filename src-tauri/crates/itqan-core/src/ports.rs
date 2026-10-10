use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itqan_contracts::{PrayerWindow, TaskKind};
use rusqlite::Connection;
use tauri::AppHandle;

use crate::error::AppError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrayerSnapshot {
    pub windows: Vec<PrayerWindow>,
    pub next: Option<PrayerWindow>,
    pub active: Option<PrayerWindow>,
}

pub trait PrayerSchedule: Send + Sync {
    fn snapshot(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<PrayerSnapshot, AppError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderTargetKind {
    Task,
    Habit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetInfo {
    pub title: Option<String>,
    pub active: bool,
}

pub trait ReminderTarget: Send + Sync {
    fn describe(
        &self,
        connection: &Connection,
        ids: &[i32],
    ) -> Result<HashMap<i32, TargetInfo>, AppError>;

    fn complete(&self, app: &AppHandle, id: i32) -> Result<(), AppError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TodayCounts {
    pub done: u32,
    pub open: u32,
}

pub trait TaskStats: Send + Sync {
    fn today_counts(
        &self,
        connection: &Connection,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TodayCounts, AppError>;

    fn next_for_today(
        &self,
        connection: &Connection,
        end: DateTime<Utc>,
    ) -> Result<Option<String>, AppError>;

    fn completed_between(
        &self,
        connection: &Connection,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<u32, AppError>;

    fn done_count(&self, connection: &Connection, kind: TaskKind) -> Result<u32, AppError>;
}

#[derive(Default)]
pub struct Ports {
    tasks: RwLock<Option<Arc<dyn TaskStats>>>,
    prayer: RwLock<Option<Arc<dyn PrayerSchedule>>>,
    reminder_targets: RwLock<HashMap<ReminderTargetKind, Arc<dyn ReminderTarget>>>,
}

impl Ports {
    pub fn set_prayer(&self, schedule: impl PrayerSchedule + 'static) -> Result<(), AppError> {
        let mut slot = self.prayer.write().map_err(|_| AppError::LockPoisoned)?;
        if slot.is_some() {
            return Err(AppError::InvalidInput(
                "a prayer schedule is already registered".into(),
            ));
        }
        *slot = Some(Arc::new(schedule));
        Ok(())
    }

    pub fn clear_prayer(&self) -> Result<(), AppError> {
        *self.prayer.write().map_err(|_| AppError::LockPoisoned)? = None;
        Ok(())
    }

    pub fn set_task_stats(&self, stats: impl TaskStats + 'static) -> Result<(), AppError> {
        let mut slot = self.tasks.write().map_err(|_| AppError::LockPoisoned)?;
        if slot.is_some() {
            return Err(AppError::InvalidInput(
                "task stats are already registered".into(),
            ));
        }
        *slot = Some(Arc::new(stats));
        Ok(())
    }

    pub fn clear_task_stats(&self) -> Result<(), AppError> {
        *self.tasks.write().map_err(|_| AppError::LockPoisoned)? = None;
        Ok(())
    }

    fn task_stats(&self) -> Result<Option<Arc<dyn TaskStats>>, AppError> {
        Ok(self
            .tasks
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .clone())
    }

    pub fn today_task_counts(
        &self,
        connection: &Connection,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TodayCounts, AppError> {
        self.task_stats()?
            .map_or(Ok(TodayCounts::default()), |stats| {
                stats.today_counts(connection, start, end)
            })
    }

    pub fn next_task_for_today(
        &self,
        connection: &Connection,
        end: DateTime<Utc>,
    ) -> Result<Option<String>, AppError> {
        self.task_stats()?
            .map_or(Ok(None), |stats| stats.next_for_today(connection, end))
    }

    pub fn tasks_completed_between(
        &self,
        connection: &Connection,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<u32, AppError> {
        self.task_stats()?
            .map_or(Ok(0), |stats| stats.completed_between(connection, from, to))
    }

    pub fn tasks_done_count(
        &self,
        connection: &Connection,
        kind: TaskKind,
    ) -> Result<u32, AppError> {
        self.task_stats()?
            .map_or(Ok(0), |stats| stats.done_count(connection, kind))
    }

    pub fn set_reminder_target(
        &self,
        kind: ReminderTargetKind,
        target: impl ReminderTarget + 'static,
    ) -> Result<(), AppError> {
        let mut targets = self
            .reminder_targets
            .write()
            .map_err(|_| AppError::LockPoisoned)?;
        if targets.contains_key(&kind) {
            return Err(AppError::InvalidInput(format!(
                "a {kind:?} reminder target is already registered"
            )));
        }
        targets.insert(kind, Arc::new(target));
        Ok(())
    }

    pub fn clear_reminder_target(&self, kind: ReminderTargetKind) -> Result<(), AppError> {
        self.reminder_targets
            .write()
            .map_err(|_| AppError::LockPoisoned)?
            .remove(&kind);
        Ok(())
    }

    pub fn reminder_target(
        &self,
        kind: ReminderTargetKind,
    ) -> Result<Option<Arc<dyn ReminderTarget>>, AppError> {
        Ok(self
            .reminder_targets
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .get(&kind)
            .cloned())
    }

    pub fn prayer(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<PrayerSnapshot, AppError> {
        let schedule = self
            .prayer
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .clone();
        schedule.map_or(Ok(PrayerSnapshot::default()), |schedule| {
            schedule.snapshot(connection, now, timezone)
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use itqan_contracts::Prayer;

    use super::*;
    use crate::db::test_connection;

    fn window(prayer: Prayer) -> PrayerWindow {
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 11, 0, 0).unwrap();
        PrayerWindow {
            prayer,
            at,
            pause_from: at,
            pause_until: at,
        }
    }

    struct Fixed;

    impl PrayerSchedule for Fixed {
        fn snapshot(
            &self,
            _: &Connection,
            _: DateTime<Utc>,
            _: Tz,
        ) -> Result<PrayerSnapshot, AppError> {
            Ok(PrayerSnapshot {
                windows: vec![window(Prayer::Asr), window(Prayer::Maghrib)],
                next: Some(window(Prayer::Maghrib)),
                active: Some(window(Prayer::Asr)),
            })
        }
    }

    #[test]
    fn prayer_answers_are_neutral_without_a_schedule() {
        let connection = test_connection();
        let ports = Ports::default();
        let now = window(Prayer::Asr).at;

        assert_eq!(
            ports.prayer(&connection, now, Tz::UTC).unwrap(),
            PrayerSnapshot::default()
        );

        ports.set_prayer(Fixed).unwrap();
        let snapshot = ports.prayer(&connection, now, Tz::UTC).unwrap();
        assert_eq!(snapshot.windows.len(), 2);
        assert_eq!(snapshot.next, Some(window(Prayer::Maghrib)));
        assert_eq!(snapshot.active, Some(window(Prayer::Asr)));
        assert!(ports.set_prayer(Fixed).is_err());

        ports.clear_prayer().unwrap();
        assert_eq!(
            ports.prayer(&connection, now, Tz::UTC).unwrap(),
            PrayerSnapshot::default()
        );
    }

    #[test]
    fn task_answers_are_neutral_without_a_tasks_module() {
        let connection = test_connection();
        let ports = Ports::default();
        let now = window(Prayer::Asr).at;

        assert_eq!(
            ports.today_task_counts(&connection, now, now).unwrap(),
            TodayCounts::default()
        );
        assert_eq!(ports.next_task_for_today(&connection, now).unwrap(), None);
        assert_eq!(
            ports
                .tasks_completed_between(&connection, now, now)
                .unwrap(),
            0
        );
        assert_eq!(
            ports
                .tasks_done_count(&connection, TaskKind::Output)
                .unwrap(),
            0
        );
    }
}
