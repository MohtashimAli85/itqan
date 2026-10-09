use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::tasks as repo;
use crate::domain::categories::CategoryId;
use crate::domain::goals::GoalId;
use crate::domain::skills::SkillId;
use crate::error::AppError;

pub type TaskId = i32;

pub const MAX_TOP_THREE: u32 = 3;
const MAX_TITLE_LENGTH: usize = 500;
const MAX_PRIORITY: u8 = 3;

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
pub enum TaskStatus {
    Open,
    Done,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Done => "done",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "open" => Some(Self::Open),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub notes: Option<String>,
    pub category_id: Option<CategoryId>,
    pub kind: TaskKind,
    pub priority: u8,
    pub due_at: Option<DateTime<Utc>>,
    pub is_top_three: bool,
    pub status: TaskStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub parent_id: Option<TaskId>,
    pub goal_id: Option<GoalId>,
    pub skill_id: Option<SkillId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub title: String,
    pub notes: Option<String>,
    pub category_id: Option<CategoryId>,
    pub kind: TaskKind,
    pub priority: u8,
    pub due_at: Option<DateTime<Utc>>,
    pub parent_id: Option<TaskId>,
    pub goal_id: Option<GoalId>,
    pub skill_id: Option<SkillId>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub category_id: Option<CategoryId>,
    pub due_before: Option<DateTime<Utc>>,
}

impl TaskInput {
    fn validated(self) -> Result<Self, AppError> {
        let title = self.title.trim().to_owned();
        if title.is_empty() {
            return Err(AppError::InvalidInput("a task needs a title".into()));
        }
        if title.chars().count() > MAX_TITLE_LENGTH {
            return Err(AppError::InvalidInput("the title is too long".into()));
        }
        if self.priority > MAX_PRIORITY {
            return Err(AppError::InvalidInput("priority is 0 to 3".into()));
        }
        let notes = self
            .notes
            .map(|notes| notes.trim().to_owned())
            .filter(|notes| !notes.is_empty());
        Ok(Self {
            title,
            notes,
            ..self
        })
    }
}

pub fn list(connection: &Connection, filter: &TaskFilter) -> Result<Vec<Task>, AppError> {
    repo::list(connection, filter)
}

pub fn get(connection: &Connection, id: TaskId) -> Result<Task, AppError> {
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("task {id}")))
}

pub fn create(
    connection: &Connection,
    input: TaskInput,
    now: DateTime<Utc>,
) -> Result<Task, AppError> {
    let input = input.validated()?;
    check_parent(connection, None, input.parent_id)?;
    let id = repo::insert(connection, &input, now)?;
    get(connection, id)
}

pub fn update(
    connection: &Connection,
    id: TaskId,
    input: TaskInput,
    now: DateTime<Utc>,
) -> Result<Task, AppError> {
    let input = input.validated()?;
    get(connection, id)?;
    check_parent(connection, Some(id), input.parent_id)?;
    repo::update(connection, id, &input, now)?;
    get(connection, id)
}

pub fn set_status(
    connection: &Connection,
    id: TaskId,
    status: TaskStatus,
    now: DateTime<Utc>,
) -> Result<Task, AppError> {
    get(connection, id)?;
    let completed_at = (status == TaskStatus::Done).then_some(now);
    repo::set_status(connection, id, status, completed_at, now)?;
    get(connection, id)
}

pub fn set_top_three(
    connection: &Connection,
    id: TaskId,
    on: bool,
    now: DateTime<Utc>,
) -> Result<Task, AppError> {
    let task = get(connection, id)?;
    if on && !task.is_top_three && repo::count_open_top_three(connection)? >= MAX_TOP_THREE {
        return Err(AppError::InvalidInput(format!(
            "today's top {MAX_TOP_THREE} is already full"
        )));
    }
    repo::set_top_three(connection, id, on, now)?;
    get(connection, id)
}

pub fn completed_between(
    connection: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<u32, AppError> {
    repo::count_completed_between(connection, from, to)
}

pub fn delete(connection: &Connection, id: TaskId) -> Result<(), AppError> {
    get(connection, id)?;
    repo::delete(connection, id)
}

fn check_parent(
    connection: &Connection,
    id: Option<TaskId>,
    parent_id: Option<TaskId>,
) -> Result<(), AppError> {
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    if Some(parent_id) == id {
        return Err(AppError::InvalidInput(
            "a task cannot be its own parent".into(),
        ));
    }
    if get(connection, parent_id)?.parent_id.is_some() {
        return Err(AppError::InvalidInput(
            "subtasks cannot have subtasks".into(),
        ));
    }
    if let Some(id) = id {
        if repo::has_subtasks(connection, id)? {
            return Err(AppError::InvalidInput(
                "a task with subtasks cannot become a subtask".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap()
    }

    fn input(title: &str) -> TaskInput {
        TaskInput {
            title: title.into(),
            ..TaskInput::default()
        }
    }

    #[test]
    fn create_trims_and_round_trips_every_field() {
        let connection = test_connection();
        let due = Utc.with_ymd_and_hms(2026, 10, 10, 14, 0, 0).unwrap();
        let task = create(
            &connection,
            TaskInput {
                title: "  Ship the parser  ".into(),
                notes: Some("   ".into()),
                category_id: Some(5),
                kind: TaskKind::DeepWork,
                priority: 2,
                due_at: Some(due),
                parent_id: None,
                goal_id: None,
                skill_id: None,
            },
            now(),
        )
        .unwrap();

        assert_eq!(task.title, "Ship the parser");
        assert_eq!(task.notes, None);
        assert_eq!(task.category_id, Some(5));
        assert_eq!(task.kind, TaskKind::DeepWork);
        assert_eq!(task.priority, 2);
        assert_eq!(task.due_at, Some(due));
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.created_at, now());
    }

    #[test]
    fn create_rejects_blank_titles_and_bad_priority() {
        let connection = test_connection();

        assert!(create(&connection, input("   "), now()).is_err());
        let loud = TaskInput {
            priority: 4,
            ..input("x")
        };
        assert!(create(&connection, loud, now()).is_err());
    }

    #[test]
    fn completing_and_reopening_tracks_completed_at() {
        let connection = test_connection();
        let task = create(&connection, input("Buy dahi"), now()).unwrap();

        let done = set_status(&connection, task.id, TaskStatus::Done, now()).unwrap();
        assert_eq!(done.completed_at, Some(now()));

        let open = set_status(&connection, task.id, TaskStatus::Open, now()).unwrap();
        assert_eq!(open.completed_at, None);
    }

    #[test]
    fn completed_between_counts_tasks_finished_in_the_window() {
        let connection = test_connection();
        let first = create(&connection, input("a"), now()).unwrap();
        let second = create(&connection, input("b"), now()).unwrap();
        create(&connection, input("c"), now()).unwrap();
        set_status(&connection, first.id, TaskStatus::Done, now()).unwrap();
        set_status(
            &connection,
            second.id,
            TaskStatus::Done,
            now() + chrono::Duration::days(1),
        )
        .unwrap();

        let count = completed_between(
            &connection,
            now() - chrono::Duration::hours(1),
            now() + chrono::Duration::hours(1),
        )
        .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn top_three_is_limited_to_open_tasks() {
        let connection = test_connection();
        let ids: Vec<TaskId> = (0..4)
            .map(|index| {
                create(&connection, input(&format!("t{index}")), now())
                    .unwrap()
                    .id
            })
            .collect();
        for id in &ids[..3] {
            set_top_three(&connection, *id, true, now()).unwrap();
        }

        assert!(set_top_three(&connection, ids[3], true, now()).is_err());

        set_status(&connection, ids[0], TaskStatus::Done, now()).unwrap();
        assert!(set_top_three(&connection, ids[3], true, now()).is_ok());
    }

    #[test]
    fn subtasks_are_one_level_deep_and_deleted_with_their_parent() {
        let connection = test_connection();
        let parent = create(&connection, input("Ship v1"), now()).unwrap();
        let child = create(
            &connection,
            TaskInput {
                parent_id: Some(parent.id),
                ..input("Write README")
            },
            now(),
        )
        .unwrap();

        let grandchild = TaskInput {
            parent_id: Some(child.id),
            ..input("too deep")
        };
        assert!(create(&connection, grandchild, now()).is_err());

        delete(&connection, parent.id).unwrap();
        assert!(get(&connection, child.id).is_err());
    }

    #[test]
    fn list_filters_by_status_and_category() {
        let connection = test_connection();
        let work = create(
            &connection,
            TaskInput {
                category_id: Some(1),
                ..input("Review MR")
            },
            now(),
        )
        .unwrap();
        let other = create(&connection, input("Call home"), now()).unwrap();
        set_status(&connection, other.id, TaskStatus::Done, now()).unwrap();

        let open = list(
            &connection,
            &TaskFilter {
                status: Some(TaskStatus::Open),
                ..TaskFilter::default()
            },
        )
        .unwrap();
        assert_eq!(open.len(), 1);

        let in_work = list(
            &connection,
            &TaskFilter {
                category_id: Some(1),
                ..TaskFilter::default()
            },
        )
        .unwrap();
        assert_eq!(
            in_work.iter().map(|task| task.id).collect::<Vec<_>>(),
            vec![work.id]
        );
    }
}
