use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itqan_contracts::PrayerWindow;
use rusqlite::Connection;

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

#[derive(Default)]
pub struct Ports {
    prayer: RwLock<Option<Arc<dyn PrayerSchedule>>>,
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
}
