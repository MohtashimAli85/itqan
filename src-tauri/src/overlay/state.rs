use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;

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
pub struct BubbleAction {
    pub id: String,
    pub label: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct OverlayChanged(pub OverlaySnapshot);

#[derive(Debug, Default)]
struct Inner {
    snapshot: OverlaySnapshot,
    next_bubble_id: u32,
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

    pub fn set_progress(&self, progress: Option<f32>) -> Result<OverlaySnapshot, AppError> {
        let progress = progress.map(|value| value.clamp(0.0, 1.0));
        self.update(|snapshot| snapshot.progress = progress)
    }

    pub fn show_bubble(
        &self,
        text: String,
        actions: Vec<BubbleAction>,
    ) -> Result<(u32, OverlaySnapshot), AppError> {
        if actions.len() > MAX_BUBBLE_ACTIONS {
            return Err(AppError::InvalidInput(format!(
                "a bubble has at most {MAX_BUBBLE_ACTIONS} actions"
            )));
        }
        let mut inner = self.lock()?;
        inner.next_bubble_id = inner.next_bubble_id.wrapping_add(1);
        let id = inner.next_bubble_id;
        inner.snapshot.bubble = Some(Bubble { id, text, actions });
        Ok((id, inner.snapshot.clone()))
    }

    pub fn dismiss_bubble(&self, id: u32) -> Result<Option<OverlaySnapshot>, AppError> {
        let mut inner = self.lock()?;
        if inner.snapshot.bubble.as_ref().map(|bubble| bubble.id) != Some(id) {
            return Ok(None);
        }
        inner.snapshot.bubble = None;
        Ok(Some(inner.snapshot.clone()))
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

        assert!(store.show_bubble("hi".into(), actions).is_err());
    }

    #[test]
    fn dismiss_only_clears_the_current_bubble() {
        let store = OverlayStore::default();
        let (first, _) = store.show_bubble("first".into(), vec![]).unwrap();
        let (second, _) = store.show_bubble("second".into(), vec![]).unwrap();

        assert!(store.dismiss_bubble(first).unwrap().is_none());
        assert!(store.dismiss_bubble(second).unwrap().is_some());
        assert!(store.snapshot().unwrap().bubble.is_none());
    }

    #[test]
    fn progress_is_clamped() {
        let store = OverlayStore::default();

        assert_eq!(store.set_progress(Some(1.4)).unwrap().progress, Some(1.0));
        assert_eq!(store.set_progress(Some(-0.2)).unwrap().progress, Some(0.0));
    }
}
