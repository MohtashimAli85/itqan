use tauri::{AppHandle, State};

use crate::error::CommandError;
use crate::overlay::{
    self, BubbleAction, BubbleOrigin, FollowMode, HitAreas, OrbState, OverlaySnapshot,
    OverlayStore, Rect,
};
use crate::scheduler;

#[tauri::command]
#[specta::specta]
pub fn set_overlay_hit_areas(
    hit_areas: State<HitAreas>,
    rects: Vec<Rect>,
) -> Result<(), CommandError> {
    Ok(hit_areas.replace(rects)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_overlay_state(store: State<OverlayStore>) -> Result<OverlaySnapshot, CommandError> {
    Ok(store.snapshot()?)
}

#[tauri::command]
#[specta::specta]
pub fn set_orb_state(
    app: AppHandle,
    store: State<OverlayStore>,
    state: OrbState,
) -> Result<(), CommandError> {
    Ok(overlay::publish(&app, store.set_orb_state(state)?)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_follow_mode(app: AppHandle, mode: FollowMode) -> Result<(), CommandError> {
    Ok(overlay::save_follow_mode(&app, mode)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_orb_progress(
    app: AppHandle,
    store: State<OverlayStore>,
    progress: Option<f32>,
) -> Result<(), CommandError> {
    Ok(overlay::publish(&app, store.set_progress(progress)?)?)
}

#[tauri::command]
#[specta::specta]
pub fn show_bubble(
    app: AppHandle,
    store: State<OverlayStore>,
    text: String,
    actions: Vec<BubbleAction>,
) -> Result<u32, CommandError> {
    let (id, snapshot) = store.show_bubble(text, actions, None)?;
    overlay::publish(&app, snapshot)?;
    Ok(id)
}

#[tauri::command]
#[specta::specta]
pub fn resolve_bubble(
    app: AppHandle,
    store: State<OverlayStore>,
    id: u32,
    action_id: Option<String>,
) -> Result<(), CommandError> {
    let Some((snapshot, origin)) = store.dismiss_bubble(id)? else {
        return Ok(());
    };
    tracing::debug!(
        bubble = id,
        action = action_id.as_deref(),
        "bubble resolved"
    );
    let panel_open = snapshot.panel_open;
    overlay::publish(&app, snapshot)?;
    let action = action_id.as_deref();
    match origin {
        Some(BubbleOrigin::Reminder { id, .. }) => scheduler::resolve_reminder(&app, id, action)?,
        Some(BubbleOrigin::Overtime) => scheduler::modes::resolve_overtime(&app, action)?,
        Some(BubbleOrigin::FocusDone) => {
            scheduler::modes::resolve_focus_done(&app, action)?;
            overlay::publish(&app, store.restore_base()?)?;
        }
        None => {}
    }
    if origin.is_some() {
        scheduler::refresh(&app)?;
    }
    if !panel_open {
        overlay::set_keyboard_focus(&app, false)?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_panel_open(
    app: AppHandle,
    store: State<OverlayStore>,
    open: bool,
) -> Result<(), CommandError> {
    let snapshot = store.set_panel_open(open)?;
    let has_bubble = snapshot.bubble.is_some();
    overlay::publish(&app, snapshot)?;
    if open || !has_bubble {
        overlay::set_keyboard_focus(&app, open)?;
    }
    Ok(())
}
