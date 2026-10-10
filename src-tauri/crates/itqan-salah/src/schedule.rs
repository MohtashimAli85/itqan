use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;
pub use itqan_contracts::{Prayer, PrayerWindow};

use super::method::Parameters;
use super::settings::PrayerSettings;
use super::solar::Coordinates;
use super::times::{self, PrayerTimes};

const JUMUAH_BEFORE_MINUTES: i64 = 30;
const JUMUAH_AFTER_MINUTES: i64 = 60;

fn coordinates(settings: &PrayerSettings) -> Option<Coordinates> {
    Some(Coordinates {
        latitude: settings.latitude?,
        longitude: settings.longitude?,
    })
}

pub fn times_on(settings: &PrayerSettings, date: NaiveDate) -> Option<PrayerTimes> {
    let parameters = Parameters::new(
        settings.method,
        settings.madhab,
        settings.high_latitude_rule,
    );
    times::calculate(date, coordinates(settings)?, &parameters)
}

pub fn windows_on(settings: &PrayerSettings, date: NaiveDate) -> Vec<PrayerWindow> {
    if !settings.enabled {
        return Vec::new();
    }
    let Some(times) = times_on(settings, date) else {
        return Vec::new();
    };
    let before = Duration::minutes(i64::from(settings.pause_before_minutes));
    let after = Duration::minutes(i64::from(settings.pause_after_minutes));
    let friday = date.weekday() == Weekday::Fri && settings.jumuah_break;
    [
        (Prayer::Fajr, times.fajr),
        (
            if friday {
                Prayer::Jumuah
            } else {
                Prayer::Dhuhr
            },
            times.dhuhr,
        ),
        (Prayer::Asr, times.asr),
        (Prayer::Maghrib, times.maghrib),
        (Prayer::Isha, times.isha),
    ]
    .into_iter()
    .map(|(prayer, at)| {
        let (before, after) = if prayer == Prayer::Jumuah {
            (
                Duration::minutes(JUMUAH_BEFORE_MINUTES).max(before),
                Duration::minutes(JUMUAH_AFTER_MINUTES).max(after),
            )
        } else {
            (before, after)
        };
        PrayerWindow {
            prayer,
            at,
            pause_from: at - before,
            pause_until: at + after,
        }
    })
    .collect()
}

pub fn windows_around(
    settings: &PrayerSettings,
    now: DateTime<Utc>,
    timezone: Tz,
) -> Vec<PrayerWindow> {
    let today = now.with_timezone(&timezone).date_naive();
    [today.pred_opt(), Some(today), today.succ_opt()]
        .into_iter()
        .flatten()
        .flat_map(|date| windows_on(settings, date))
        .collect()
}

pub fn active_window(
    settings: &PrayerSettings,
    now: DateTime<Utc>,
    timezone: Tz,
) -> Option<PrayerWindow> {
    windows_around(settings, now, timezone)
        .into_iter()
        .find(|window| window.pause_from <= now && now < window.pause_until)
}

pub fn next_prayer(
    settings: &PrayerSettings,
    now: DateTime<Utc>,
    timezone: Tz,
) -> Option<PrayerWindow> {
    windows_around(settings, now, timezone)
        .into_iter()
        .find(|window| window.at > now)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn karachi() -> PrayerSettings {
        PrayerSettings {
            enabled: true,
            city: Some("Karachi".into()),
            latitude: Some(24.8607),
            longitude: Some(67.0011),
            ..PrayerSettings::default()
        }
    }

    fn local(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Tz::Asia__Karachi
            .with_ymd_and_hms(2026, 10, day, hour, minute, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn disabled_or_unset_settings_have_no_windows() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        assert!(windows_on(&PrayerSettings::default(), date).is_empty());
        let no_city = PrayerSettings {
            enabled: true,
            ..PrayerSettings::default()
        };
        assert!(windows_on(&no_city, date).is_empty());
    }

    #[test]
    fn friday_dhuhr_becomes_a_longer_jumuah_break() {
        let friday = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        let windows = windows_on(&karachi(), friday);
        let jumuah = windows.iter().find(|w| w.prayer == Prayer::Jumuah).unwrap();
        assert_eq!(jumuah.at - jumuah.pause_from, Duration::minutes(30));
        assert_eq!(jumuah.pause_until - jumuah.at, Duration::minutes(60));

        let saturday = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        assert!(windows_on(&karachi(), saturday)
            .iter()
            .any(|w| w.prayer == Prayer::Dhuhr));
    }

    #[test]
    fn active_window_covers_the_pause_around_asr() {
        let settings = karachi();
        let asr = windows_on(&settings, NaiveDate::from_ymd_opt(2026, 10, 10).unwrap())
            .into_iter()
            .find(|w| w.prayer == Prayer::Asr)
            .unwrap();

        let during = active_window(&settings, asr.at + Duration::minutes(10), Tz::Asia__Karachi);
        assert_eq!(during.map(|w| w.prayer), Some(Prayer::Asr));
        assert!(active_window(&settings, asr.pause_until, Tz::Asia__Karachi).is_none());
    }

    #[test]
    fn next_prayer_after_isha_is_tomorrows_fajr() {
        let next = next_prayer(&karachi(), local(10, 23, 0), Tz::Asia__Karachi).unwrap();
        assert_eq!(next.prayer, Prayer::Fajr);
        assert_eq!(next.at.with_timezone(&Tz::Asia__Karachi).day(), 11);
    }
}
