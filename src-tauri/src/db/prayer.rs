use rusqlite::{params, Connection, Row};

use crate::error::AppError;
use crate::prayer::method::{HighLatitudeRule, Madhab, Method};
use crate::prayer::settings::PrayerSettings;

fn text<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_value(value)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| AppError::InvalidInput("expected a text value".into()))
}

fn parse<T: serde::de::DeserializeOwned>(row: &Row, index: usize) -> rusqlite::Result<T> {
    let value: String = row.get(index)?;
    serde_json::from_value(serde_json::Value::String(value)).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, error.into())
    })
}

pub fn get(connection: &Connection) -> Result<PrayerSettings, AppError> {
    Ok(connection.query_row(
        "SELECT enabled, city, latitude, longitude, method, madhab, high_latitude_rule,
                pause_before_minutes, pause_after_minutes, jumuah_break
         FROM prayer_settings WHERE id = 1",
        [],
        |row| {
            Ok(PrayerSettings {
                enabled: row.get(0)?,
                city: row.get(1)?,
                latitude: row.get(2)?,
                longitude: row.get(3)?,
                method: parse::<Method>(row, 4)?,
                madhab: parse::<Madhab>(row, 5)?,
                high_latitude_rule: parse::<HighLatitudeRule>(row, 6)?,
                pause_before_minutes: row.get(7)?,
                pause_after_minutes: row.get(8)?,
                jumuah_break: row.get(9)?,
            })
        },
    )?)
}

pub fn save(connection: &Connection, settings: &PrayerSettings) -> Result<(), AppError> {
    connection.execute(
        "UPDATE prayer_settings SET enabled = ?1, city = ?2, latitude = ?3, longitude = ?4,
                method = ?5, madhab = ?6, high_latitude_rule = ?7, pause_before_minutes = ?8,
                pause_after_minutes = ?9, jumuah_break = ?10
         WHERE id = 1",
        params![
            settings.enabled,
            settings.city,
            settings.latitude,
            settings.longitude,
            text(&settings.method)?,
            text(&settings.madhab)?,
            text(&settings.high_latitude_rule)?,
            settings.pause_before_minutes,
            settings.pause_after_minutes,
            settings.jumuah_break,
        ],
    )?;
    Ok(())
}
