use rusqlite::{params, Connection};

use itqan_core::db::enums::{column, to_text};

use crate::method::{HighLatitudeRule, Madhab, Method};
use crate::settings::PrayerSettings;
use itqan_core::error::AppError;

pub fn get(connection: &Connection) -> Result<PrayerSettings, AppError> {
    Ok(connection.query_row(
        "SELECT enabled, city, latitude, longitude, method, madhab, high_latitude_rule,
                pause_before_minutes, pause_after_minutes, jumuah_break
         FROM salah_settings WHERE id = 1",
        [],
        |row| {
            Ok(PrayerSettings {
                enabled: row.get(0)?,
                city: row.get(1)?,
                latitude: row.get(2)?,
                longitude: row.get(3)?,
                method: column::<Method>(row, 4)?,
                madhab: column::<Madhab>(row, 5)?,
                high_latitude_rule: column::<HighLatitudeRule>(row, 6)?,
                pause_before_minutes: row.get(7)?,
                pause_after_minutes: row.get(8)?,
                jumuah_break: row.get(9)?,
            })
        },
    )?)
}

pub fn save(connection: &Connection, settings: &PrayerSettings) -> Result<(), AppError> {
    connection.execute(
        "UPDATE salah_settings SET enabled = ?1, city = ?2, latitude = ?3, longitude = ?4,
                method = ?5, madhab = ?6, high_latitude_rule = ?7, pause_before_minutes = ?8,
                pause_after_minutes = ?9, jumuah_break = ?10
         WHERE id = 1",
        params![
            settings.enabled,
            settings.city,
            settings.latitude,
            settings.longitude,
            to_text(&settings.method)?,
            to_text(&settings.madhab)?,
            to_text(&settings.high_latitude_rule)?,
            settings.pause_before_minutes,
            settings.pause_after_minutes,
            settings.jumuah_break,
        ],
    )?;
    Ok(())
}
