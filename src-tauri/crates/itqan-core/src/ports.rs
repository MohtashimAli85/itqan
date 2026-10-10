use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itqan_contracts::PrayerWindow;
use rusqlite::Connection;

use crate::error::AppError;

pub trait PrayerSchedule: Send + Sync {
    fn windows_around(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Vec<PrayerWindow>, AppError>;

    fn next_prayer(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Option<PrayerWindow>, AppError>;

    fn active_window(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Option<PrayerWindow>, AppError>;
}

#[derive(Default)]
pub struct Ports {
    prayer: OnceLock<Box<dyn PrayerSchedule>>,
}

impl Ports {
    pub fn set_prayer(&self, schedule: impl PrayerSchedule + 'static) -> Result<(), AppError> {
        self.prayer
            .set(Box::new(schedule))
            .map_err(|_| AppError::InvalidInput("a prayer schedule is already registered".into()))
    }

    pub fn prayer_windows(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Vec<PrayerWindow>, AppError> {
        self.prayer.get().map_or(Ok(Vec::new()), |schedule| {
            schedule.windows_around(connection, now, timezone)
        })
    }

    pub fn next_prayer(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Option<PrayerWindow>, AppError> {
        self.prayer.get().map_or(Ok(None), |schedule| {
            schedule.next_prayer(connection, now, timezone)
        })
    }

    pub fn active_prayer(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<Option<PrayerWindow>, AppError> {
        self.prayer.get().map_or(Ok(None), |schedule| {
            schedule.active_window(connection, now, timezone)
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use itqan_contracts::Prayer;

    use super::*;
    use crate::db::test_connection;

    fn window() -> PrayerWindow {
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 11, 0, 0).unwrap();
        PrayerWindow {
            prayer: Prayer::Asr,
            at,
            pause_from: at,
            pause_until: at,
        }
    }

    struct Fixed;

    impl PrayerSchedule for Fixed {
        fn windows_around(
            &self,
            _: &Connection,
            _: DateTime<Utc>,
            _: Tz,
        ) -> Result<Vec<PrayerWindow>, AppError> {
            Ok(vec![window()])
        }

        fn next_prayer(
            &self,
            _: &Connection,
            _: DateTime<Utc>,
            _: Tz,
        ) -> Result<Option<PrayerWindow>, AppError> {
            Ok(Some(window()))
        }

        fn active_window(
            &self,
            _: &Connection,
            _: DateTime<Utc>,
            _: Tz,
        ) -> Result<Option<PrayerWindow>, AppError> {
            Ok(None)
        }
    }

    #[test]
    fn prayer_answers_are_neutral_until_a_schedule_registers() {
        let connection = test_connection();
        let ports = Ports::default();
        let now = window().at;

        assert!(ports
            .prayer_windows(&connection, now, Tz::UTC)
            .unwrap()
            .is_empty());
        assert!(ports
            .next_prayer(&connection, now, Tz::UTC)
            .unwrap()
            .is_none());

        ports.set_prayer(Fixed).unwrap();
        assert_eq!(
            ports.prayer_windows(&connection, now, Tz::UTC).unwrap(),
            vec![window()]
        );
        assert_eq!(
            ports.next_prayer(&connection, now, Tz::UTC).unwrap(),
            Some(window())
        );
        assert!(ports.set_prayer(Fixed).is_err());
    }
}
