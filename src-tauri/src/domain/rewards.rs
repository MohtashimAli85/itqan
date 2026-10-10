use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::rewards as repo;
use crate::domain::levels::{self, LevelProgress};
use crate::domain::profile::{Motivator, Profile};
use crate::domain::skills::SkillId;
use crate::domain::tasks::{TaskId, TaskKind};
use itqan_core::error::AppError;

const MAX_FREEZES: u8 = 2;
const FREEZE_EVERY: u32 = 7;
const MAX_GAP_DAYS: i64 = 60;
pub const MILESTONE_XP: u32 = 40;
pub const GOAL_XP: u32 = 100;
pub const HABIT_XP: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum XpSource {
    Task,
    Focus,
    Milestone,
    Goal,
    Habit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Streak {
    pub current: u32,
    pub best: u32,
    pub freezes: u8,
    pub last_active_day: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Award {
    pub source: XpSource,
    pub amount: u32,
    pub task_id: Option<TaskId>,
    pub skill_id: Option<SkillId>,
    pub reference_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RewardOutcome {
    pub amount: u32,
    pub total_xp: u32,
    pub level: LevelProgress,
    pub leveled_up: bool,
    pub streak: Streak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BadgeId {
    FirstShip,
    TenShips,
    FirstFocus,
    FocusFifty,
    WeekStreak,
    MonthStreak,
    Learner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: BadgeId,
    pub title: String,
    pub description: String,
    pub earned: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    pub date: NaiveDate,
    pub xp: u32,
    pub focus_minutes: u32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSummary {
    pub total_xp: u32,
    pub level: LevelProgress,
    pub streak: Streak,
    pub today_xp: u32,
    pub days: Vec<DaySummary>,
    pub badges: Vec<Badge>,
}

fn weight(profile: &Profile, motivator: Motivator) -> Option<u8> {
    if profile.motivators.is_empty() {
        return None;
    }
    Some(
        profile
            .motivators
            .iter()
            .find(|weight| weight.motivator == motivator)
            .map_or(0, |weight| weight.weight),
    )
}

pub fn multiplier(profile: &Profile, motivators: &[Motivator]) -> f64 {
    let weights: Vec<f64> = motivators
        .iter()
        .filter_map(|motivator| weight(profile, *motivator))
        .map(f64::from)
        .collect();
    if weights.is_empty() {
        return 1.0;
    }
    let average = weights.iter().sum::<f64>() / weights.len() as f64;
    0.5 + 1.5 * average / 100.0
}

pub fn task_xp(kind: TaskKind, profile: &Profile) -> u32 {
    let (base, motivators): (f64, &[Motivator]) = match kind {
        TaskKind::Output => (30.0, &[Motivator::Building]),
        TaskKind::DeepWork => (20.0, &[Motivator::Building, Motivator::Learning]),
        TaskKind::Learning => (15.0, &[Motivator::Learning]),
        TaskKind::Habit => (5.0, &[Motivator::Health]),
    };
    ((base * multiplier(profile, motivators)).round() as u32).max(1)
}

pub fn focus_xp(minutes: u16) -> u32 {
    (u32::from(minutes) * 15 / 25).max(5)
}

pub fn record_active_day(
    streak: Streak,
    today: NaiveDate,
    is_rest_day: impl Fn(NaiveDate) -> bool,
) -> Streak {
    let mut next = streak;
    match streak.last_active_day {
        Some(last) if last >= today => return streak,
        Some(last) if (today - last).num_days() > MAX_GAP_DAYS => next.current = 1,
        Some(last) => {
            let missed = (1..(today - last).num_days())
                .map(|offset| last + Duration::days(offset))
                .filter(|day| !is_rest_day(*day))
                .count();
            let missed = u8::try_from(missed).unwrap_or(u8::MAX);
            if missed <= streak.freezes {
                next.freezes -= missed;
                next.current += 1;
            } else {
                next.current = 1;
            }
        }
        None => next.current = 1,
    }
    if next.current > 0 && next.current.is_multiple_of(FREEZE_EVERY) {
        next.freezes = (next.freezes + 1).min(MAX_FREEZES);
    }
    next.best = next.best.max(next.current);
    next.last_active_day = Some(today);
    next
}

pub fn award(
    connection: &Connection,
    award: Award,
    now: DateTime<Utc>,
    today: NaiveDate,
    is_rest_day: impl Fn(NaiveDate) -> bool,
) -> Result<Option<RewardOutcome>, AppError> {
    let before = repo::total_xp(connection)?;
    if !repo::insert_event(connection, &award, now)? {
        return Ok(None);
    }
    if let Some(skill_id) = award.skill_id {
        repo::add_skill_xp(connection, skill_id, award.amount)?;
    }
    let mut streak = repo::streak(connection)?;
    if award.source != XpSource::Habit {
        streak = record_active_day(streak, today, is_rest_day);
        repo::save_streak(connection, &streak)?;
    }
    let total_xp = before + award.amount;
    let level = levels::progress(total_xp);
    Ok(Some(RewardOutcome {
        amount: award.amount,
        total_xp,
        level,
        leveled_up: level.level > levels::progress(before).level,
        streak,
    }))
}

fn badge(id: BadgeId, title: &str, description: &str, earned: bool) -> Badge {
    Badge {
        id,
        title: title.into(),
        description: description.into(),
        earned,
    }
}

pub fn badges(connection: &Connection, streak: &Streak) -> Result<Vec<Badge>, AppError> {
    let ships = repo::count_done_tasks(connection, TaskKind::Output)?;
    let learning = repo::count_done_tasks(connection, TaskKind::Learning)?;
    let focus = repo::count_completed_focus(connection)?;
    Ok(vec![
        badge(
            BadgeId::FirstShip,
            "First ship",
            "Finish an output task",
            ships >= 1,
        ),
        badge(
            BadgeId::TenShips,
            "Shipper",
            "Finish ten output tasks",
            ships >= 10,
        ),
        badge(
            BadgeId::FirstFocus,
            "In the zone",
            "Complete a focus session",
            focus >= 1,
        ),
        badge(
            BadgeId::FocusFifty,
            "Deep worker",
            "Complete fifty focus sessions",
            focus >= 50,
        ),
        badge(
            BadgeId::WeekStreak,
            "Steady",
            "Reach a seven day streak",
            streak.best >= 7,
        ),
        badge(
            BadgeId::MonthStreak,
            "Consistent",
            "Reach a thirty day streak",
            streak.best >= 30,
        ),
        badge(
            BadgeId::Learner,
            "Learner",
            "Finish five learning tasks",
            learning >= 5,
        ),
    ])
}

pub fn summary(
    connection: &Connection,
    now: DateTime<Utc>,
    timezone: Tz,
    days: u32,
) -> Result<ProgressSummary, AppError> {
    let today = now.with_timezone(&timezone).date_naive();
    let first = today - Duration::days(i64::from(days.saturating_sub(1)));
    let since = now - Duration::days(i64::from(days) + 1);
    let mut summaries: Vec<DaySummary> = (0..i64::from(days))
        .map(|offset| DaySummary {
            date: first + Duration::days(offset),
            xp: 0,
            focus_minutes: 0,
            active: false,
        })
        .collect();
    let mut add = |at: DateTime<Utc>, change: &dyn Fn(&mut DaySummary)| {
        let date = at.with_timezone(&timezone).date_naive();
        if let Some(day) = summaries.iter_mut().find(|day| day.date == date) {
            change(day);
        }
    };
    for (at, amount, source) in repo::events_since(connection, since)? {
        add(at, &|day| {
            day.xp += amount;
            day.active |= source != XpSource::Habit;
        });
    }
    for (at, minutes) in repo::focus_since(connection, since)? {
        add(at, &|day| day.focus_minutes += u32::from(minutes));
    }
    let total_xp = repo::total_xp(connection)?;
    let streak = repo::streak(connection)?;
    Ok(ProgressSummary {
        total_xp,
        level: levels::progress(total_xp),
        today_xp: summaries.last().map_or(0, |day| day.xp),
        badges: badges(connection, &streak)?,
        streak,
        days: summaries,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, TimeZone, Weekday};

    use super::*;
    use crate::domain::profile::MotivatorWeight;
    use itqan_core::db::test_connection;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    fn no_rest(_: NaiveDate) -> bool {
        false
    }

    fn weekends(day: NaiveDate) -> bool {
        matches!(day.weekday(), Weekday::Sat | Weekday::Sun)
    }

    fn builder() -> Profile {
        Profile {
            motivators: vec![
                MotivatorWeight {
                    motivator: Motivator::Learning,
                    weight: 40,
                },
                MotivatorWeight {
                    motivator: Motivator::Building,
                    weight: 35,
                },
                MotivatorWeight {
                    motivator: Motivator::Health,
                    weight: 25,
                },
            ],
            ..Profile::default()
        }
    }

    #[test]
    fn outputs_earn_more_than_inputs_and_follow_the_profile() {
        let profile = builder();
        let output = task_xp(TaskKind::Output, &profile);
        let learning = task_xp(TaskKind::Learning, &profile);
        assert!(output > learning);
        assert_eq!(task_xp(TaskKind::Output, &Profile::default()), 30);

        let money_only = Profile {
            motivators: vec![MotivatorWeight {
                motivator: Motivator::Money,
                weight: 100,
            }],
            ..Profile::default()
        };
        assert_eq!(task_xp(TaskKind::Output, &money_only), 15);
    }

    #[test]
    fn streaks_grow_daily_and_restart_without_losing_the_best() {
        let mut streak = Streak {
            freezes: 0,
            ..Streak::default()
        };
        for day in 1..=5 {
            streak = record_active_day(streak, date(day), no_rest);
        }
        assert_eq!(streak.current, 5);
        assert_eq!(record_active_day(streak, date(5), no_rest), streak);

        let restarted = record_active_day(streak, date(9), no_rest);
        assert_eq!(restarted.current, 1);
        assert_eq!(restarted.best, 5);
    }

    #[test]
    fn freezes_cover_missed_days_and_rest_days_never_count() {
        let streak = Streak {
            current: 4,
            best: 4,
            freezes: 1,
            last_active_day: Some(date(6)),
        };
        let covered = record_active_day(streak, date(8), no_rest);
        assert_eq!(covered.current, 5);
        assert_eq!(covered.freezes, 0);

        let friday = Streak {
            last_active_day: Some(date(9)),
            freezes: 0,
            ..streak
        };
        let monday = record_active_day(friday, date(12), weekends);
        assert_eq!(monday.current, 5);
    }

    #[test]
    fn a_week_earns_a_freeze_up_to_two() {
        let mut streak = Streak::default();
        for day in 1..=21 {
            streak = record_active_day(streak, date(day), no_rest);
        }
        assert_eq!(streak.current, 21);
        assert_eq!(streak.freezes, MAX_FREEZES);
    }

    #[test]
    fn awards_count_once_feed_skills_and_level_up() {
        let connection = test_connection();
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        let skill = crate::domain::skills::create(&connection, "Rust", now).unwrap();
        let first = Award {
            source: XpSource::Goal,
            amount: GOAL_XP,
            task_id: None,
            skill_id: Some(skill.id),
            reference_id: Some(1),
        };

        let outcome = award(&connection, first, now, date(10), no_rest)
            .unwrap()
            .unwrap();
        assert_eq!(outcome.total_xp, 100);
        assert!(outcome.leveled_up);
        assert_eq!(outcome.streak.current, 1);
        assert!(award(&connection, first, now, date(10), no_rest)
            .unwrap()
            .is_none());

        let skills = crate::domain::skills::list(&connection).unwrap();
        assert_eq!(skills[0].xp, 100);

        let water = Award {
            source: XpSource::Habit,
            amount: HABIT_XP,
            task_id: None,
            skill_id: None,
            reference_id: None,
        };
        assert!(award(&connection, water, now, date(10), no_rest)
            .unwrap()
            .is_some());
        assert!(award(&connection, water, now, date(10), no_rest)
            .unwrap()
            .is_some());

        let progress = summary(&connection, now, Tz::UTC, 7).unwrap();
        assert_eq!(progress.total_xp, 106);
        assert_eq!(progress.today_xp, 106);
        assert!(progress.days.last().unwrap().active);
        assert_eq!(progress.days.len(), 7);
    }
}
