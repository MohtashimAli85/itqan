use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use rrule::{RRule, Unvalidated};

use crate::error::AppError;

pub fn validate(rule: &str, anchor: DateTime<Utc>, timezone: Tz) -> Result<(), AppError> {
    next_after(rule, anchor, timezone, anchor).map(|_| ())
}

pub fn next_after(
    rule: &str,
    anchor: DateTime<Utc>,
    timezone: Tz,
    after: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, AppError> {
    let invalid =
        |error: rrule::RRuleError| AppError::InvalidInput(format!("bad repeat rule: {error}"));
    let tz = rrule::Tz::Tz(timezone);
    let parsed: RRule<Unvalidated> = rule.parse().map_err(invalid)?;
    let set = parsed.build(anchor.with_timezone(&tz)).map_err(invalid)?;
    let result = set
        .after((after + Duration::seconds(1)).with_timezone(&tz))
        .all(1);
    Ok(result.dates.first().map(|date| date.with_timezone(&Utc)))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn karachi(day: u32, hour: u32) -> DateTime<Utc> {
        Tz::Asia__Karachi
            .with_ymd_and_hms(2026, 10, day, hour, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn daily_rule_returns_the_next_local_occurrence() {
        let next = next_after(
            "FREQ=DAILY",
            karachi(1, 21),
            Tz::Asia__Karachi,
            karachi(10, 22),
        )
        .unwrap();
        assert_eq!(next, Some(karachi(11, 21)));
    }

    #[test]
    fn weekly_rule_respects_weekdays() {
        let next = next_after(
            "FREQ=WEEKLY;BYDAY=FR",
            karachi(2, 13),
            Tz::Asia__Karachi,
            karachi(10, 9),
        )
        .unwrap();
        assert_eq!(next, Some(karachi(16, 13)));
    }

    #[test]
    fn local_time_is_kept_across_daylight_saving() {
        let london = Tz::Europe__London;
        let anchor = london
            .with_ymd_and_hms(2026, 10, 20, 8, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let after_switch = london
            .with_ymd_and_hms(2026, 10, 30, 9, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let next = next_after("FREQ=DAILY", anchor, london, after_switch)
            .unwrap()
            .unwrap();

        assert_eq!(
            next.with_timezone(&london).format("%d %H:%M").to_string(),
            "31 08:00"
        );
    }

    #[test]
    fn finished_rules_have_no_next_occurrence() {
        let next = next_after(
            "FREQ=DAILY;COUNT=2",
            karachi(1, 9),
            Tz::Asia__Karachi,
            karachi(10, 9),
        )
        .unwrap();
        assert_eq!(next, None);
    }

    #[test]
    fn invalid_rules_are_rejected() {
        assert!(validate("FREQ=SOMETIMES", karachi(1, 9), Tz::Asia__Karachi).is_err());
    }
}
