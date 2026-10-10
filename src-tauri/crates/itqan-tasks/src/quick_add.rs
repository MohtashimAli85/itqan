use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::categories;
use crate::tasks::{self, Task, TaskInput};
use itqan_core::error::AppError;
use itqan_core::reminders::{self, ReminderInput};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QuickAddOutcome {
    pub task: Task,
    pub notice: Option<String>,
}

pub fn create<Tz: TimeZone>(
    connection: &Connection,
    text: &str,
    now: DateTime<Tz>,
    timezone: chrono_tz::Tz,
) -> Result<QuickAddOutcome, AppError> {
    let parsed = parse(text, now.clone());
    let category_id = match &parsed.category {
        Some(name) => categories::find_by_name(connection, name)?.map(|category| category.id),
        None => None,
    };
    let mut title = parsed.title;
    if let (Some(name), None) = (&parsed.category, category_id) {
        title = format!("{title} #{name}");
    }
    let now = now.with_timezone(&Utc);
    let task = tasks::create(
        connection,
        TaskInput {
            title,
            category_id,
            priority: parsed.priority,
            due_at: parsed.due_at,
            ..TaskInput::default()
        },
        now,
    )?;
    if let (true, Some(at)) = (parsed.has_time, task.due_at) {
        reminders::insert(
            connection,
            ReminderInput {
                task_id: Some(task.id),
                title: None,
                at,
                rrule: None,
                critical: false,
            },
            timezone,
            now,
        )?;
    }
    if !parsed.top_three {
        return Ok(QuickAddOutcome { task, notice: None });
    }
    match tasks::set_top_three(connection, task.id, true, now) {
        Ok(task) => Ok(QuickAddOutcome { task, notice: None }),
        Err(AppError::InvalidInput(message)) => Ok(QuickAddOutcome {
            task,
            notice: Some(message),
        }),
        Err(error) => Err(error),
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QuickAdd {
    pub title: String,
    pub due_at: Option<DateTime<Utc>>,
    pub category: Option<String>,
    pub priority: u8,
    pub top_three: bool,
    pub has_time: bool,
}

const DEFAULT_TIME: (u32, u32) = (9, 0);
const TONIGHT: (u32, u32) = (20, 0);
const DANGLING: &[&str] = &["at", "on", "by", "in"];

#[derive(Default)]
struct Found {
    date: Option<NaiveDate>,
    time: Option<NaiveTime>,
    offset: Option<Duration>,
}

pub fn parse<Tz: TimeZone>(text: &str, now: DateTime<Tz>) -> QuickAdd {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let today = now.date_naive();
    let mut result = QuickAdd::default();
    let mut found = Found::default();
    let mut kept: Vec<&str> = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let token = tokens[index];
        let lower = token.to_lowercase();
        let next = tokens.get(index + 1).map(|value| value.to_lowercase());

        if let Some(name) = token.strip_prefix('#').filter(|name| !name.is_empty()) {
            result.category = Some(name.to_owned());
        } else if !lower.is_empty() && lower.chars().all(|c| c == '!') && lower.len() <= 3 {
            result.priority = u8::try_from(lower.len()).unwrap_or(3);
        } else if lower == "*" {
            result.top_three = true;
        } else if lower == "today" {
            found.date = Some(today);
        } else if lower == "tonight" {
            found.date = Some(today);
            found.time = found
                .time
                .or(NaiveTime::from_hms_opt(TONIGHT.0, TONIGHT.1, 0));
        } else if lower == "tomorrow" {
            found.date = today.succ_opt();
        } else if let Some(weekday) = parse_weekday(&lower) {
            found.date = Some(next_weekday(today, weekday));
        } else if lower == "in" {
            if let Some((offset, used)) = parse_offset(&tokens[index + 1..]) {
                found.offset = Some(offset);
                index += 1 + used;
                continue;
            }
            kept.push(token);
        } else if let Some((time, used)) = parse_time(&lower, next.as_deref()) {
            found.time = Some(time);
            index += used;
            continue;
        } else {
            kept.push(token);
        }
        index += 1;
    }

    while kept
        .last()
        .is_some_and(|word| DANGLING.contains(&word.to_lowercase().as_str()))
    {
        kept.pop();
    }
    result.title = kept.join(" ");
    result.has_time = found.time.is_some() || found.offset.is_some();
    result.due_at = resolve(&now, found);
    result
}

fn resolve<Tz: TimeZone>(now: &DateTime<Tz>, found: Found) -> Option<DateTime<Utc>> {
    if let Some(offset) = found.offset {
        return Some(now.with_timezone(&Utc) + offset);
    }
    let today = now.date_naive();
    let date = match (found.date, found.time) {
        (None, None) => return None,
        (Some(date), _) => date,
        (None, Some(time)) if time <= now.time() => today.succ_opt()?,
        (None, Some(_)) => today,
    };
    let time = found
        .time
        .or(NaiveTime::from_hms_opt(DEFAULT_TIME.0, DEFAULT_TIME.1, 0))?;
    now.timezone()
        .from_local_datetime(&date.and_time(time))
        .earliest()
        .map(|local| local.with_timezone(&Utc))
}

fn parse_weekday(word: &str) -> Option<Weekday> {
    let weekday = match word {
        "monday" | "mon" => Weekday::Mon,
        "tuesday" | "tue" | "tues" => Weekday::Tue,
        "wednesday" | "wed" => Weekday::Wed,
        "thursday" | "thu" | "thurs" => Weekday::Thu,
        "friday" | "fri" => Weekday::Fri,
        "saturday" | "sat" => Weekday::Sat,
        "sunday" | "sun" => Weekday::Sun,
        _ => return None,
    };
    Some(weekday)
}

fn next_weekday(from: NaiveDate, weekday: Weekday) -> NaiveDate {
    let ahead = (7 + weekday.num_days_from_monday() - from.weekday().num_days_from_monday()) % 7;
    from + Duration::days(i64::from(ahead))
}

fn parse_offset(tokens: &[&str]) -> Option<(Duration, usize)> {
    let first = tokens.first()?.to_lowercase();
    let split = first
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(first.len());
    let amount: i64 = first[..split].parse().ok()?;
    let (unit, used) = if split < first.len() {
        (first[split..].to_owned(), 1)
    } else {
        (tokens.get(1)?.to_lowercase(), 2)
    };
    let offset = match unit.as_str() {
        "m" | "min" | "mins" | "minute" | "minutes" => Duration::minutes(amount),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::hours(amount),
        "d" | "day" | "days" => Duration::days(amount),
        _ => return None,
    };
    Some((offset, used))
}

fn parse_time(word: &str, next: Option<&str>) -> Option<(NaiveTime, usize)> {
    match word {
        "noon" => return Some((NaiveTime::from_hms_opt(12, 0, 0)?, 1)),
        "midnight" => return Some((NaiveTime::from_hms_opt(23, 59, 0)?, 1)),
        _ => {}
    }
    let (clock, suffix, used) = if let Some(clock) = word.strip_suffix("am") {
        (clock, Some(false), 1)
    } else if let Some(clock) = word.strip_suffix("pm") {
        (clock, Some(true), 1)
    } else {
        match next {
            Some("am") => (word, Some(false), 2),
            Some("pm") => (word, Some(true), 2),
            _ => (word, None, 1),
        }
    };
    let (hour, minute) = match clock.split_once(':') {
        Some((hour, minute)) => (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?),
        None if suffix.is_some() => (clock.parse::<u32>().ok()?, 0),
        None => return None,
    };
    let hour = match suffix {
        Some(_) if !(1..=12).contains(&hour) => return None,
        Some(true) if hour != 12 => hour + 12,
        Some(false) if hour == 12 => 0,
        _ => hour,
    };
    Some((NaiveTime::from_hms_opt(hour, minute, 0)?, used))
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;
    use crate::test_connection;

    fn karachi() -> FixedOffset {
        FixedOffset::east_opt(5 * 3600).unwrap()
    }

    fn now() -> DateTime<FixedOffset> {
        karachi().with_ymd_and_hms(2026, 10, 10, 16, 30, 0).unwrap()
    }

    fn local(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        karachi()
            .with_ymd_and_hms(2026, 10, day, hour, minute, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn plain_text_is_just_a_title() {
        let parsed = parse("  Read the Rust book  ", now());
        assert_eq!(parsed.title, "Read the Rust book");
        assert_eq!(parsed.due_at, None);
    }

    #[test]
    fn at_seven_pm_is_today_in_local_time() {
        let parsed = parse("buy dahi at 7pm", now());
        assert_eq!(parsed.title, "buy dahi");
        assert_eq!(parsed.due_at, Some(local(10, 19, 0)));
    }

    #[test]
    fn a_time_that_already_passed_moves_to_tomorrow() {
        let parsed = parse("call ammi 9:15 am", now());
        assert_eq!(parsed.title, "call ammi");
        assert_eq!(parsed.due_at, Some(local(11, 9, 15)));
    }

    #[test]
    fn twenty_four_hour_times_work() {
        assert_eq!(
            parse("standup 17:45", now()).due_at,
            Some(local(10, 17, 45))
        );
    }

    #[test]
    fn tomorrow_without_a_time_defaults_to_morning() {
        let parsed = parse("tomorrow submit invoice", now());
        assert_eq!(parsed.title, "submit invoice");
        assert_eq!(parsed.due_at, Some(local(11, 9, 0)));
    }

    #[test]
    fn weekdays_pick_the_next_occurrence() {
        let parsed = parse("gym on monday at 6am", now());
        assert_eq!(parsed.title, "gym");
        assert_eq!(parsed.due_at, Some(local(12, 6, 0)));
    }

    #[test]
    fn tonight_defaults_to_eight() {
        assert_eq!(
            parse("ship parser tonight", now()).due_at,
            Some(local(10, 20, 0))
        );
    }

    #[test]
    fn relative_offsets_work_with_and_without_a_space() {
        assert_eq!(
            parse("stretch in 30 min", now()).due_at,
            Some(local(10, 17, 0))
        );
        assert_eq!(parse("water in 2h", now()).due_at, Some(local(10, 18, 30)));
    }

    #[test]
    fn tags_priority_and_top_three_are_extracted() {
        let parsed = parse("Review MR #work !! *", now());
        assert_eq!(parsed.title, "Review MR");
        assert_eq!(parsed.category.as_deref(), Some("work"));
        assert_eq!(parsed.priority, 2);
        assert!(parsed.top_three);
    }

    #[test]
    fn create_resolves_known_categories_and_keeps_unknown_tags() {
        let connection = test_connection();

        let known = create(
            &connection,
            "Review MR #work",
            now(),
            chrono_tz::Tz::Asia__Karachi,
        )
        .unwrap();
        assert_eq!(known.task.category_id, Some(1));
        assert_eq!(known.task.title, "Review MR");

        let unknown = create(
            &connection,
            "Plan trip #travel",
            now(),
            chrono_tz::Tz::Asia__Karachi,
        )
        .unwrap();
        assert_eq!(unknown.task.category_id, None);
        assert_eq!(unknown.task.title, "Plan trip #travel");
    }

    #[test]
    fn create_reports_a_full_top_three_without_failing() {
        let connection = test_connection();
        for index in 0..3 {
            create(
                &connection,
                &format!("task {index} *"),
                now(),
                chrono_tz::Tz::Asia__Karachi,
            )
            .unwrap();
        }

        let fourth = create(
            &connection,
            "one more *",
            now(),
            chrono_tz::Tz::Asia__Karachi,
        )
        .unwrap();
        assert!(!fourth.task.is_top_three);
        assert!(fourth.notice.is_some());
    }

    #[test]
    fn create_adds_a_reminder_only_for_explicit_times() {
        let connection = test_connection();
        let zone = chrono_tz::Tz::Asia__Karachi;

        let timed = create(&connection, "buy dahi at 7pm", now(), zone).unwrap();
        let ports = itqan_core::ports::Ports::default();
        crate::targets::register(&ports).unwrap();
        let reminders = reminders::list_for_task(&connection, &ports, timed.task.id).unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].next_at, Some(local(10, 19, 0)));

        let dated = create(&connection, "submit invoice tomorrow", now(), zone).unwrap();
        assert!(reminders::list_for_task(&connection, &ports, dated.task.id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn invalid_times_stay_in_the_title() {
        let parsed = parse("room 13pm", now());
        assert_eq!(parsed.title, "room 13pm");
        assert_eq!(parsed.due_at, None);
    }
}
