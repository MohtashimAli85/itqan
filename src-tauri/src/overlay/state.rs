use std::collections::VecDeque;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use specta::Type;

pub use itqan_core::overlay::BubbleAction;

use itqan_core::error::AppError;

pub const MAX_BUBBLE_ACTIONS: usize = 3;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OrbState {
    #[default]
    Idle,
    Happy,
    Alert,
    Focus,
    Resting,
    Critical,
    Listening,
    Evening,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FollowMode {
    #[default]
    Follow,
    Corner,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Bubble {
    pub id: u32,
    pub text: String,
    pub actions: Vec<BubbleAction>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySnapshot {
    pub orb_state: OrbState,
    pub follow_mode: FollowMode,
    pub bubble: Option<Bubble>,
    pub progress: Option<f32>,
    pub panel_open: bool,
    pub docked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct OverlayChanged(pub OverlaySnapshot);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleOrigin {
    Reminder { id: i32, critical: bool },
    Overtime,
    FocusDone,
    Nudge { id: i32, critical: bool },
}

impl BubbleOrigin {
    pub fn is_critical(self) -> bool {
        match self {
            Self::Reminder { critical, .. } | Self::Nudge { critical, .. } => critical,
            Self::Overtime | Self::FocusDone => false,
        }
    }
}

#[derive(Debug)]
struct Queued {
    bubble: Bubble,
    origin: Option<BubbleOrigin>,
}

#[derive(Debug, Default)]
struct Inner {
    snapshot: OverlaySnapshot,
    next_bubble_id: u32,
    queue: VecDeque<Queued>,
    base: OrbState,
    focus_docked: bool,
    typing: bool,
}

impl Inner {
    fn sync_bubble(&mut self) {
        self.snapshot.bubble = self.queue.front().map(|queued| queued.bubble.clone());
    }
}

#[derive(Debug, Default)]
pub struct OverlayStore(Mutex<Inner>);

impl OverlayStore {
    pub fn snapshot(&self) -> Result<OverlaySnapshot, AppError> {
        Ok(self.lock()?.snapshot.clone())
    }

    pub fn set_orb_state(&self, state: OrbState) -> Result<OverlaySnapshot, AppError> {
        self.update(|snapshot| snapshot.orb_state = state)
    }

    pub fn set_follow_mode(&self, mode: FollowMode) -> Result<OverlaySnapshot, AppError> {
        self.update(|snapshot| snapshot.follow_mode = mode)
    }

    pub fn set_appearance(
        &self,
        base: OrbState,
        docked: bool,
    ) -> Result<OverlaySnapshot, AppError> {
        let mut inner = self.lock()?;
        inner.base = base;
        inner.focus_docked = docked;
        inner.snapshot.docked = docked || inner.typing;
        if !matches!(
            inner.snapshot.orb_state,
            OrbState::Critical | OrbState::Happy
        ) {
            inner.snapshot.orb_state = base;
        }
        Ok(inner.snapshot.clone())
    }

    pub fn set_typing(&self, typing: bool) -> Result<Option<OverlaySnapshot>, AppError> {
        let mut inner = self.lock()?;
        if inner.typing == typing {
            return Ok(None);
        }
        inner.typing = typing;
        inner.snapshot.docked = inner.focus_docked || typing;
        Ok(Some(inner.snapshot.clone()))
    }

    pub fn restore_base(&self) -> Result<OverlaySnapshot, AppError> {
        let mut inner = self.lock()?;
        inner.snapshot.orb_state = inner.base;
        Ok(inner.snapshot.clone())
    }

    pub fn dismiss_origin(
        &self,
        origin: BubbleOrigin,
    ) -> Result<Option<OverlaySnapshot>, AppError> {
        let mut inner = self.lock()?;
        let before = inner.queue.len();
        inner.queue.retain(|queued| queued.origin != Some(origin));
        if inner.queue.len() == before {
            return Ok(None);
        }
        inner.sync_bubble();
        Ok(Some(inner.snapshot.clone()))
    }

    pub fn set_panel_open(&self, open: bool) -> Result<OverlaySnapshot, AppError> {
        self.update(|snapshot| snapshot.panel_open = open)
    }

    pub fn set_progress(&self, progress: Option<f32>) -> Result<OverlaySnapshot, AppError> {
        let progress = progress.map(|value| value.clamp(0.0, 1.0));
        self.update(|snapshot| snapshot.progress = progress)
    }

    pub fn show_bubble(
        &self,
        text: String,
        actions: Vec<BubbleAction>,
        origin: Option<BubbleOrigin>,
    ) -> Result<(u32, OverlaySnapshot), AppError> {
        if actions.len() > MAX_BUBBLE_ACTIONS {
            return Err(AppError::InvalidInput(format!(
                "a bubble has at most {MAX_BUBBLE_ACTIONS} actions"
            )));
        }
        let mut inner = self.lock()?;
        inner.next_bubble_id = inner.next_bubble_id.wrapping_add(1);
        let id = inner.next_bubble_id;
        inner.queue.push_back(Queued {
            bubble: Bubble { id, text, actions },
            origin,
        });
        inner.sync_bubble();
        Ok((id, inner.snapshot.clone()))
    }

    pub fn dismiss_bubble(
        &self,
        id: u32,
    ) -> Result<Option<(OverlaySnapshot, Option<BubbleOrigin>)>, AppError> {
        let mut inner = self.lock()?;
        let Some(position) = inner.queue.iter().position(|queued| queued.bubble.id == id) else {
            return Ok(None);
        };
        let origin = inner
            .queue
            .remove(position)
            .and_then(|queued| queued.origin);
        inner.sync_bubble();
        Ok(Some((inner.snapshot.clone(), origin)))
    }

    pub fn has_origin(&self, origin: BubbleOrigin) -> Result<bool, AppError> {
        Ok(self
            .lock()?
            .queue
            .iter()
            .any(|queued| queued.origin == Some(origin)))
    }

    pub fn has_critical(&self) -> Result<bool, AppError> {
        Ok(self
            .lock()?
            .queue
            .iter()
            .filter_map(|queued| queued.origin)
            .any(BubbleOrigin::is_critical))
    }

    fn update(
        &self,
        change: impl FnOnce(&mut OverlaySnapshot),
    ) -> Result<OverlaySnapshot, AppError> {
        let mut inner = self.lock()?;
        change(&mut inner.snapshot);
        Ok(inner.snapshot.clone())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, AppError> {
        self.0.lock().map_err(|_| AppError::LockPoisoned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(id: &str) -> BubbleAction {
        BubbleAction {
            id: id.to_owned(),
            label: id.to_owned(),
        }
    }

    #[test]
    fn show_bubble_rejects_more_than_three_actions() {
        let store = OverlayStore::default();
        let actions = vec![action("a"), action("b"), action("c"), action("d")];

        assert!(store.show_bubble("hi".into(), actions, None).is_err());
    }

    #[test]
    fn bubbles_queue_and_show_one_at_a_time() {
        let store = OverlayStore::default();
        let (first, _) = store.show_bubble("first".into(), vec![], None).unwrap();
        let (second, snapshot) = store
            .show_bubble(
                "second".into(),
                vec![],
                Some(BubbleOrigin::Reminder {
                    id: 7,
                    critical: true,
                }),
            )
            .unwrap();
        assert_eq!(snapshot.bubble.map(|bubble| bubble.id), Some(first));

        let (snapshot, origin) = store.dismiss_bubble(first).unwrap().unwrap();
        assert_eq!(origin, None);
        assert_eq!(snapshot.bubble.map(|bubble| bubble.id), Some(second));
        assert!(store
            .has_origin(BubbleOrigin::Reminder {
                id: 7,
                critical: true
            })
            .unwrap());
        assert!(store.has_critical().unwrap());

        let (snapshot, origin) = store.dismiss_bubble(second).unwrap().unwrap();
        assert_eq!(
            origin,
            Some(BubbleOrigin::Reminder {
                id: 7,
                critical: true
            })
        );
        assert!(snapshot.bubble.is_none());
        assert!(store.dismiss_bubble(second).unwrap().is_none());
    }

    #[test]
    fn appearance_keeps_critical_until_restored() {
        let store = OverlayStore::default();
        store.set_orb_state(OrbState::Critical).unwrap();

        let snapshot = store.set_appearance(OrbState::Focus, true).unwrap();
        assert_eq!(snapshot.orb_state, OrbState::Critical);
        assert!(snapshot.docked);

        assert_eq!(store.restore_base().unwrap().orb_state, OrbState::Focus);
    }

    #[test]
    fn typing_docks_without_losing_the_focus_dock() {
        let store = OverlayStore::default();
        assert!(store.set_typing(true).unwrap().unwrap().docked);
        assert!(store.set_typing(true).unwrap().is_none());
        store.set_appearance(OrbState::Focus, true).unwrap();
        assert!(store.set_typing(false).unwrap().unwrap().docked);
        assert!(!store.set_appearance(OrbState::Idle, false).unwrap().docked);
    }

    #[test]
    fn progress_is_clamped() {
        let store = OverlayStore::default();

        assert_eq!(store.set_progress(Some(1.4)).unwrap().progress, Some(1.0));
        assert_eq!(store.set_progress(Some(-0.2)).unwrap().progress, Some(0.0));
    }
}
