pub mod commands;

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;

use crate::db::settings as repo;
use crate::error::AppError;

const TIMEZONE: &str = "timezone";

pub fn timezone(connection: &Connection) -> Result<Tz, AppError> {
    let stored = repo::get(connection, TIMEZONE)?;
    Ok(stored
        .as_deref()
        .and_then(|name| name.parse().ok())
        .unwrap_or_else(system_timezone))
}

pub fn set_timezone(connection: &Connection, timezone: &str) -> Result<Tz, AppError> {
    let parsed: Tz = timezone
        .parse()
        .map_err(|_| AppError::InvalidInput(format!("unknown timezone {timezone}")))?;
    repo::set(connection, TIMEZONE, parsed.name())?;
    Ok(parsed)
}

pub fn system_timezone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

pub fn day_bounds(timezone: Tz, date: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let at = |date: NaiveDate| {
        timezone
            .from_local_datetime(&date.and_time(NaiveTime::MIN))
            .earliest()
            .map_or_else(Utc::now, |start| start.with_timezone(&Utc))
    };
    (at(date), at(date + Duration::days(1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;

    #[test]
    fn stored_timezone_wins_and_bad_names_are_rejected() {
        let connection = test_connection();

        set_timezone(&connection, "Asia/Karachi").unwrap();
        assert_eq!(timezone(&connection).unwrap(), Tz::Asia__Karachi);
        assert!(set_timezone(&connection, "Mars/Olympus").is_err());
    }
}
