use crate::overlay::FollowMode;

#[derive(Debug, Clone, PartialEq)]
pub struct TrayView {
    pub title: Option<String>,
    pub next_task: Option<String>,
    pub focus_active: bool,
    pub paused: bool,
    pub resting: bool,
    pub follow_mode: FollowMode,
}

pub struct TrayInputs {
    pub focus_minutes_left: Option<i64>,
    pub tasks_left: u32,
    pub next_task: Option<String>,
    pub paused: bool,
    pub resting: bool,
    pub follow_mode: FollowMode,
}

const MAX_TASK_LENGTH: usize = 40;

fn shorten(title: &str) -> String {
    if title.chars().count() <= MAX_TASK_LENGTH {
        return title.to_owned();
    }
    let short: String = title.chars().take(MAX_TASK_LENGTH - 1).collect();
    format!("{}…", short.trim_end())
}

pub fn view(inputs: &TrayInputs) -> TrayView {
    let title = match inputs.focus_minutes_left {
        Some(minutes) => Some(format!("{}m", minutes.max(0))),
        None if inputs.tasks_left > 0 => Some(inputs.tasks_left.to_string()),
        None => None,
    };
    TrayView {
        title,
        next_task: inputs.next_task.as_deref().map(shorten),
        focus_active: inputs.focus_minutes_left.is_some(),
        paused: inputs.paused,
        resting: inputs.resting,
        follow_mode: inputs.follow_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> TrayInputs {
        TrayInputs {
            focus_minutes_left: None,
            tasks_left: 0,
            next_task: None,
            paused: false,
            resting: false,
            follow_mode: FollowMode::Follow,
        }
    }

    #[test]
    fn title_shows_focus_time_then_tasks_left() {
        assert_eq!(view(&inputs()).title, None);
        assert_eq!(
            view(&TrayInputs {
                tasks_left: 3,
                ..inputs()
            })
            .title
            .as_deref(),
            Some("3")
        );
        let focusing = TrayInputs {
            focus_minutes_left: Some(12),
            tasks_left: 3,
            ..inputs()
        };
        let shown = view(&focusing);
        assert_eq!(shown.title.as_deref(), Some("12m"));
        assert!(shown.focus_active);
    }

    #[test]
    fn long_task_titles_are_shortened() {
        let long = TrayInputs {
            next_task: Some("Write the end of year report for the whole engineering team".into()),
            ..inputs()
        };
        let next = view(&long).next_task.unwrap();
        assert!(next.chars().count() <= MAX_TASK_LENGTH);
        assert!(next.ends_with('…'));
    }
}
