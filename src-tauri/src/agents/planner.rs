use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::goals::MilestoneInput;

const DEFAULT_WEEKS: i64 = 4;
const MAX_WEEKS: i64 = 52;
const MAX_MILESTONES: i64 = 8;
const SESSION_MINUTES: u16 = 45;
const MAX_SESSIONS: u16 = 7;

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRequest {
    pub goal_title: String,
    pub target_date: Option<NaiveDate>,
    pub today: NaiveDate,
    pub free_hours_per_week: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PlanSource {
    Rules,
    Ai,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanProposal {
    pub milestones: Vec<MilestoneInput>,
    pub sessions_per_week: u16,
    pub session_minutes: u16,
    pub first_task: String,
    pub source: PlanSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoalKind {
    Learning,
    Building,
    Health,
    General,
}

fn classify(title: &str) -> GoalKind {
    let lower = title.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| lower.contains(word));
    if has(&[
        "ship", "build", "launch", "release", "publish", "make ", "create", "v1",
    ]) {
        GoalKind::Building
    } else if has(&["learn", "study", "master", "understand", "course", "read "]) {
        GoalKind::Learning
    } else if has(&[
        "fit", "run", "weight", "sleep", "walk", "gym", "health", "strength",
    ]) {
        GoalKind::Health
    } else {
        GoalKind::General
    }
}

fn title_for(kind: GoalKind, index: i64, count: i64) -> String {
    let last = index == count - 1;
    let step = index;
    match kind {
        GoalKind::Learning if index == 0 => "Pick one resource and build a hello world".into(),
        GoalKind::Learning if last => "Ship a small project you can show someone".into(),
        GoalKind::Learning => format!("Build something small with what you learned (part {step})"),
        GoalKind::Building if index == 0 => {
            "Write what v1 means in three bullets and set up the project".into()
        }
        GoalKind::Building if last => "Release it and tell someone".into(),
        GoalKind::Building if index == count - 2 => "Polish: fix the three biggest issues".into(),
        GoalKind::Building => format!("Ship the next visible piece (part {step})"),
        GoalKind::Health if index == 0 => {
            "Measure your baseline and pick a routine you can keep".into()
        }
        GoalKind::Health if last => "Measure again and compare with your baseline".into(),
        GoalKind::Health => format!("Keep the routine and add a little more (week {})", step + 1),
        GoalKind::General if index == 0 => "Define what done looks like".into(),
        GoalKind::General if last => "Finish and review what you learned".into(),
        GoalKind::General => format!("Make visible progress (part {step})"),
    }
}

fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

pub fn rule_plan(request: &PlanRequest) -> PlanProposal {
    let first_week = week_start(request.today);
    let weeks = request
        .target_date
        .map(|target| (target - first_week).num_days() / 7 + 1)
        .unwrap_or(DEFAULT_WEEKS)
        .clamp(1, MAX_WEEKS);
    let count = weeks.min(MAX_MILESTONES);
    let kind = classify(&request.goal_title);
    let milestones: Vec<MilestoneInput> = (0..count)
        .map(|index| MilestoneInput {
            title: title_for(kind, index, count),
            week_start: first_week + Duration::weeks(index * weeks / count),
        })
        .collect();
    let free_minutes = u16::from(request.free_hours_per_week.unwrap_or(4)) * 60;
    let sessions_per_week = (free_minutes / SESSION_MINUTES).clamp(1, MAX_SESSIONS);
    let first_task = milestones
        .first()
        .map(|milestone| format!("{SESSION_MINUTES} min: {}", milestone.title))
        .unwrap_or_default();
    PlanProposal {
        milestones,
        sessions_per_week,
        session_minutes: SESSION_MINUTES,
        first_task,
        source: PlanSource::Rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    fn request(title: &str, target: Option<NaiveDate>, hours: Option<u8>) -> PlanRequest {
        PlanRequest {
            goal_title: title.into(),
            target_date: target,
            today: date(10, 10),
            free_hours_per_week: hours,
        }
    }

    #[test]
    fn building_goals_end_with_a_release() {
        let plan = rule_plan(&request("Ship Itqan v1", Some(date(11, 1)), Some(8)));
        let titles: Vec<&str> = plan.milestones.iter().map(|m| m.title.as_str()).collect();
        assert_eq!(titles.len(), 4);
        assert!(titles[0].contains("v1 means"));
        assert_eq!(titles[2], "Polish: fix the three biggest issues");
        assert_eq!(titles[3], "Release it and tell someone");
        assert_eq!(plan.milestones[0].week_start, date(10, 5));
        assert_eq!(plan.milestones[3].week_start, date(10, 26));
    }

    #[test]
    fn long_goals_are_capped_and_spread_out() {
        let plan = rule_plan(&request(
            "Learn Rust",
            Some(date(12, 31) + Duration::days(180)),
            None,
        ));
        assert_eq!(plan.milestones.len(), 8);
        assert!(plan
            .milestones
            .windows(2)
            .all(|pair| pair[0].week_start < pair[1].week_start));
        assert!(plan.milestones[7].title.starts_with("Ship"));
    }

    #[test]
    fn sessions_follow_free_time() {
        assert_eq!(
            rule_plan(&request("Get fit", None, Some(8))).sessions_per_week,
            7
        );
        assert_eq!(
            rule_plan(&request("Get fit", None, Some(2))).sessions_per_week,
            2
        );
        assert_eq!(
            rule_plan(&request("Get fit", None, Some(0))).sessions_per_week,
            1
        );
        let plan = rule_plan(&request("Get fit", None, Some(2)));
        assert!(plan.first_task.starts_with("45 min: Measure your baseline"));
    }

    #[test]
    fn past_targets_still_give_one_milestone() {
        let plan = rule_plan(&request("Tidy the garage", Some(date(9, 1)), None));
        assert_eq!(plan.milestones.len(), 1);
    }
}
