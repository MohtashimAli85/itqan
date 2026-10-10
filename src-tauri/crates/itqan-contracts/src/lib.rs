use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

pub type TaskId = i32;
pub type GoalId = i32;
pub type MilestoneId = i32;
pub type FocusSessionId = i32;
pub type HabitId = i32;
pub type SkillId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Work,
    Evening,
    Rest,
    Focus,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaskKind {
    #[default]
    Output,
    Learning,
    DeepWork,
    Habit,
}

impl TaskKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Output => "output",
            Self::Learning => "learning",
            Self::DeepWork => "deepWork",
            Self::Habit => "habit",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "output" => Some(Self::Output),
            "learning" => Some(Self::Learning),
            "deepWork" => Some(Self::DeepWork),
            "habit" => Some(Self::Habit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HabitKind {
    Stretch,
    EyeRest,
    Water,
    Medicine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Prayer {
    Fajr,
    Dhuhr,
    Jumuah,
    Asr,
    Maghrib,
    Isha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrayerWindow {
    pub prayer: Prayer,
    pub at: DateTime<Utc>,
    pub pause_from: DateTime<Utc>,
    pub pause_until: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppEvent {
    Tick,
    TaskCompleted {
        task_id: TaskId,
        kind: TaskKind,
        skill_id: Option<SkillId>,
    },
    FocusCompleted {
        session_id: FocusSessionId,
        minutes: u16,
    },
    MilestoneCompleted {
        milestone_id: MilestoneId,
    },
    GoalCompleted {
        goal_id: GoalId,
    },
    HabitLogged {
        kind: HabitKind,
    },
    ModeChanged {
        from: Option<Mode>,
        to: Mode,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> Vec<AppEvent> {
        vec![
            AppEvent::Tick,
            AppEvent::TaskCompleted {
                task_id: 7,
                kind: TaskKind::Learning,
                skill_id: Some(2),
            },
            AppEvent::FocusCompleted {
                session_id: 3,
                minutes: 25,
            },
            AppEvent::MilestoneCompleted { milestone_id: 2 },
            AppEvent::GoalCompleted { goal_id: 1 },
            AppEvent::HabitLogged {
                kind: HabitKind::Water,
            },
            AppEvent::ModeChanged {
                from: Some(Mode::Work),
                to: Mode::Evening,
            },
        ]
    }

    const VARIANTS: usize = 7;

    fn variant(event: &AppEvent) -> usize {
        match event {
            AppEvent::Tick => 0,
            AppEvent::TaskCompleted { .. } => 1,
            AppEvent::FocusCompleted { .. } => 2,
            AppEvent::MilestoneCompleted { .. } => 3,
            AppEvent::GoalCompleted { .. } => 4,
            AppEvent::HabitLogged { .. } => 5,
            AppEvent::ModeChanged { .. } => 6,
        }
    }

    #[test]
    fn samples_cover_every_variant() {
        let mut seen: Vec<usize> = samples().iter().map(variant).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen, (0..VARIANTS).collect::<Vec<_>>());
    }

    #[test]
    fn every_event_round_trips_through_serde() {
        for event in samples() {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(serde_json::from_str::<AppEvent>(&json).unwrap(), event);
        }
    }
}
