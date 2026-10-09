use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::method::Parameters;
use super::solar::{Coordinates, SolarTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrayerTimes {
    pub fajr: DateTime<Utc>,
    pub sunrise: DateTime<Utc>,
    pub dhuhr: DateTime<Utc>,
    pub asr: DateTime<Utc>,
    pub maghrib: DateTime<Utc>,
    pub isha: DateTime<Utc>,
}

fn at(date: NaiveDate, hours: f64) -> Option<DateTime<Utc>> {
    if !hours.is_finite() {
        return None;
    }
    let whole_hours = hours.floor();
    let minutes = ((hours - whole_hours) * 60.0).floor();
    let seconds = ((hours - (whole_hours + minutes / 60.0)) * 3600.0).floor();
    let offset = whole_hours as i64 * 3600 + minutes as i64 * 60 + seconds as i64;
    Some(date.and_hms_opt(0, 0, 0)?.and_utc() + Duration::seconds(offset))
}

fn rounded(time: DateTime<Utc>, adjustment_minutes: i64) -> DateTime<Utc> {
    let adjusted = time + Duration::minutes(adjustment_minutes);
    let seconds = i64::from(adjusted.second());
    let offset = if seconds >= 30 {
        60 - seconds
    } else {
        -seconds
    };
    adjusted + Duration::seconds(offset)
}

pub fn calculate(
    date: NaiveDate,
    coordinates: Coordinates,
    parameters: &Parameters,
) -> Option<PrayerTimes> {
    let solar = SolarTime::new(date, coordinates);
    let tomorrow = date.succ_opt()?;
    let tomorrow_solar = SolarTime::new(tomorrow, coordinates);

    let dhuhr = at(date, solar.transit)?;
    let sunrise = at(date, solar.sunrise)?;
    let sunset = at(date, solar.sunset)?;
    let asr = at(date, solar.afternoon(parameters.madhab.shadow_length()))?;
    let tomorrow_sunrise = at(tomorrow, tomorrow_solar.sunrise)?;
    let night_seconds = (tomorrow_sunrise - sunset).num_seconds() as f64;
    let (fajr_portion, isha_portion) = parameters.night_portions();

    let safe_fajr = sunrise - Duration::seconds((fajr_portion * night_seconds) as i64);
    let fajr = match at(date, solar.hour_angle(-parameters.fajr_angle, false)) {
        Some(fajr) if fajr >= safe_fajr => fajr,
        _ => safe_fajr,
    };

    let isha = if parameters.isha_interval_minutes > 0 {
        sunset + Duration::minutes(parameters.isha_interval_minutes)
    } else {
        let safe_isha = sunset + Duration::seconds((isha_portion * night_seconds) as i64);
        match at(date, solar.hour_angle(-parameters.isha_angle, true)) {
            Some(isha) if isha <= safe_isha => isha,
            _ => safe_isha,
        }
    };

    let adjust = parameters.adjustments;
    Some(PrayerTimes {
        fajr: rounded(fajr, adjust.fajr),
        sunrise: rounded(sunrise, adjust.sunrise),
        dhuhr: rounded(dhuhr, adjust.dhuhr),
        asr: rounded(asr, adjust.asr),
        maghrib: rounded(sunset, adjust.maghrib),
        isha: rounded(isha, adjust.isha),
    })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use super::*;
    use crate::prayer::method::{HighLatitudeRule, Madhab, Method};

    fn parse(value: &str) -> DateTime<Utc> {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
            .unwrap()
            .and_utc()
    }

    fn check(fixture: &str, method: Method, madhab: Madhab) {
        let mut lines = fixture.lines();
        let header: Vec<f64> = lines
            .next()
            .unwrap()
            .split_whitespace()
            .skip(3)
            .map(|value| value.parse().unwrap())
            .collect();
        let coordinates = Coordinates {
            latitude: header[0],
            longitude: header[1],
        };
        let parameters = Parameters::new(method, madhab, HighLatitudeRule::MiddleOfTheNight);
        let mut checked = 0;
        for line in lines {
            let fields: Vec<&str> = line.split(',').collect();
            let date = NaiveDate::parse_from_str(fields[0], "%Y-%m-%d").unwrap();
            let times = calculate(date, coordinates, &parameters).unwrap();
            let actual = [
                times.fajr,
                times.sunrise,
                times.dhuhr,
                times.asr,
                times.maghrib,
                times.isha,
            ];
            for (index, expected) in fields[1..].iter().enumerate() {
                assert_eq!(
                    actual[index],
                    parse(expected),
                    "{method:?} {date} prayer {index}"
                );
            }
            checked += 1;
        }
        assert!(checked > 70);
    }

    #[test]
    fn matches_adhan_for_karachi_every_day_of_the_year() {
        check(
            include_str!("../../tests/fixtures/prayer/karachi.csv"),
            Method::Karachi,
            Madhab::Hanafi,
        );
    }

    #[test]
    fn matches_adhan_for_london() {
        check(
            include_str!("../../tests/fixtures/prayer/london.csv"),
            Method::MuslimWorldLeague,
            Madhab::Shafi,
        );
    }

    #[test]
    fn matches_adhan_for_new_york() {
        check(
            include_str!("../../tests/fixtures/prayer/new-york.csv"),
            Method::NorthAmerica,
            Madhab::Shafi,
        );
    }

    #[test]
    fn matches_adhan_for_makkah() {
        check(
            include_str!("../../tests/fixtures/prayer/makkah.csv"),
            Method::UmmAlQura,
            Madhab::Shafi,
        );
    }

    #[test]
    fn matches_adhan_for_cairo() {
        check(
            include_str!("../../tests/fixtures/prayer/cairo.csv"),
            Method::Egyptian,
            Madhab::Shafi,
        );
    }

    #[test]
    fn matches_adhan_for_dubai() {
        check(
            include_str!("../../tests/fixtures/prayer/dubai.csv"),
            Method::Dubai,
            Madhab::Shafi,
        );
    }

    #[test]
    fn polar_days_return_none_instead_of_panicking() {
        let tromso = Coordinates {
            latitude: 69.6492,
            longitude: 18.9553,
        };
        let parameters = Parameters::new(
            Method::MuslimWorldLeague,
            Madhab::Shafi,
            HighLatitudeRule::MiddleOfTheNight,
        );
        let midsummer = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        assert!(calculate(midsummer, tromso, &parameters).is_none());
    }
}
