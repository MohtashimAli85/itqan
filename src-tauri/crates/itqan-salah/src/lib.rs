mod astronomy;
pub mod commands;
pub mod method;
pub mod repo;
pub mod schedule;
pub mod settings;
pub mod solar;
pub mod times;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itqan_core::error::AppError;
use itqan_core::module::{Migration, Module};
use itqan_core::ports::{Ports, PrayerSchedule, PrayerSnapshot};
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "salah_settings",
    sql: include_str!("../migrations/0001_salah_settings.sql"),
}];

pub struct SalahModule;

impl Module for SalahModule {
    fn id(&self) -> &'static str {
        "salah"
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn setup(&self, app: &AppHandle) -> Result<(), AppError> {
        app.state::<Ports>().set_prayer(Schedule)
    }
}

struct Schedule;

impl PrayerSchedule for Schedule {
    fn snapshot(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
        timezone: Tz,
    ) -> Result<PrayerSnapshot, AppError> {
        let windows = schedule::windows_around(&repo::get(connection)?, now, timezone);
        Ok(PrayerSnapshot {
            next: schedule::next_in(&windows, now),
            active: schedule::active_in(&windows, now),
            windows,
        })
    }
}

#[cfg(test)]
mod tests {
    use itqan_core::db::{migrations, test_connection_with};
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn existing_prayer_settings_survive_the_rename() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrations::run(&mut connection).unwrap();
        connection
            .execute(
                "UPDATE prayer_settings SET enabled = 1, city = 'Karachi', latitude = 24.86,
                     longitude = 67.0, pause_after_minutes = 30 WHERE id = 1",
                [],
            )
            .unwrap();

        migrations::run_modules(&mut connection, &[&SalahModule]).unwrap();

        let saved = repo::get(&connection).unwrap();
        assert!(saved.enabled);
        assert_eq!(saved.city.as_deref(), Some("Karachi"));
        assert_eq!(saved.pause_after_minutes, 30);
        let old_table: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'prayer_settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old_table, 0);
    }

    #[test]
    fn a_fresh_database_gets_default_settings() {
        let connection = test_connection_with(&[&SalahModule]);
        assert!(!repo::get(&connection).unwrap().enabled);
    }
}
