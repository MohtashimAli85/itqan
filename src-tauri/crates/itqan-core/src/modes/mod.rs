pub mod commands;
mod work_hours;

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modes::work_hours as repo;

use crate::db::settings as settings_repo;
use crate::error::AppError;

pub use itqan_contracts::Mode;

pub const MORNING_START_MINUTE: u16 = 6 * 60;
const DEFAULT_EVENING_END_MINUTE: u16 = 23 * 60;
const EVENING_END: &str = "evening_end_minute";
const REST_UNTIL: &str = "rest_until";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkDay {
    pub weekday: u8,
    pub enabled: bool,
    pub start_minute: u16,
    pub end_minute: u16,
}

pub fn scheduled_mode(days: &[WorkDay], evening_end_minute: u16, local: NaiveDateTime) -> Mode {
    let weekday = u8::try_from(local.weekday().num_days_from_monday()).unwrap_or(0);
    let minute = u16::try_from(local.hour() * 60 + local.minute()).unwrap_or(0);
    let today = days
        .iter()
        .find(|day| day.weekday == weekday && day.enabled);
    match today {
        Some(day) if (day.start_minute..day.end_minute).contains(&minute) => Mode::Work,
        _ if minute >= evening_end_minute || minute < MORNING_START_MINUTE => Mode::Rest,
        Some(_) => Mode::Evening,
        None => Mode::Rest,
    }
}

pub fn work_hours(connection: &Connection) -> Result<Vec<WorkDay>, AppError> {
    repo::list(connection)
}

pub fn set_work_hours(
    connection: &Connection,
    days: Vec<WorkDay>,
) -> Result<Vec<WorkDay>, AppError> {
    let mut seen = [false; 7];
    for day in &days {
        let index = usize::from(day.weekday);
        if index > 6 || seen[index] {
            return Err(AppError::InvalidInput(
                "each weekday appears once, 0 to 6".into(),
            ));
        }
        seen[index] = true;
        if day.start_minute >= day.end_minute || day.end_minute > 24 * 60 {
            return Err(AppError::InvalidInput(
                "work must end after it starts".into(),
            ));
        }
    }
    repo::save(connection, &days)?;
    work_hours(connection)
}

pub fn evening_end_minute(connection: &Connection) -> Result<u16, AppError> {
    Ok(settings_repo::get(connection, EVENING_END)?
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_EVENING_END_MINUTE))
}

pub fn rest_until(connection: &Connection) -> Result<Option<DateTime<Utc>>, AppError> {
    Ok(settings_repo::get(connection, REST_UNTIL)?
        .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
        .map(|value| value.with_timezone(&Utc)))
}

pub fn set_rest_until(
    connection: &Connection,
    until: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    let value = until.map(|until| until.to_rfc3339()).unwrap_or_default();
    settings_repo::set(connection, REST_UNTIL, &value)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::db::test_connection;

    fn at(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, day)
            .unwrap()
            .and_hms_opt(hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn default_week_is_nine_to_five_on_weekdays() {
        let connection = test_connection();
        let days = work_hours(&connection).unwrap();
        let end = evening_end_minute(&connection).unwrap();

        assert_eq!(scheduled_mode(&days, end, at(12, 10, 0)), Mode::Work);
        assert_eq!(scheduled_mode(&days, end, at(12, 17, 0)), Mode::Evening);
        assert_eq!(scheduled_mode(&days, end, at(12, 7, 30)), Mode::Evening);
        assert_eq!(scheduled_mode(&days, end, at(12, 23, 30)), Mode::Rest);
        assert_eq!(scheduled_mode(&days, end, at(12, 3, 0)), Mode::Rest);
        assert_eq!(scheduled_mode(&days, end, at(10, 12, 0)), Mode::Rest);
    }

    #[test]
    fn work_hours_validate_each_day_once() {
        let connection = test_connection();
        let mut days = work_hours(&connection).unwrap();
        days[0].end_minute = days[0].start_minute;
        assert!(set_work_hours(&connection, days.clone()).is_err());

        days[0].end_minute = 18 * 60;
        days[1].weekday = 0;
        assert!(set_work_hours(&connection, days).is_err());
    }

    #[test]
    fn rest_until_round_trips_and_clears() {
        let connection = test_connection();
        let until = DateTime::parse_from_rfc3339("2026-10-10T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        set_rest_until(&connection, Some(until)).unwrap();
        assert_eq!(rest_until(&connection).unwrap(), Some(until));

        set_rest_until(&connection, None).unwrap();
        assert_eq!(rest_until(&connection).unwrap(), None);
    }
}
