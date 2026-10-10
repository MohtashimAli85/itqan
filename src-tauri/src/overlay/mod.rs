mod activity;
mod hit_areas;
mod state;
mod tracker;

use tauri::{App, AppHandle, Manager, WebviewWindow};
use tauri_specta::Event;

use crate::db::{settings as settings_repo, Database};
use crate::platform;
use itqan_core::error::AppError;

pub use activity::Activity;
pub use hit_areas::{HitAreas, Rect};
pub use state::{
    BubbleAction, BubbleOrigin, FollowMode, OrbState, OverlayChanged, OverlaySnapshot, OverlayStore,
};
pub use tracker::OverlayCursor;

pub const OVERLAY_LABEL: &str = "overlay";
const FOLLOW_MODE: &str = "follow_mode";

pub fn setup(app: &App) -> Result<(), AppError> {
    let window = overlay_window(app.handle())?;
    platform::configure_overlay(&window)?;
    window.set_ignore_cursor_events(true)?;
    app.manage(HitAreas::default());
    app.manage(OverlayStore::default());
    app.manage(Activity::default());
    tracker::spawn(app.handle().clone(), window)?;
    Ok(())
}

pub fn restore_follow_mode(app: &AppHandle) -> Result<(), AppError> {
    let stored = app
        .state::<Database>()
        .with(|connection| settings_repo::get(connection, FOLLOW_MODE))?;
    let Some(mode) =
        stored.and_then(|value| serde_json::from_value(serde_json::Value::String(value)).ok())
    else {
        return Ok(());
    };
    publish(app, app.state::<OverlayStore>().set_follow_mode(mode)?)
}

pub fn save_follow_mode(app: &AppHandle, mode: FollowMode) -> Result<(), AppError> {
    let value = serde_json::to_value(mode)?
        .as_str()
        .map(str::to_owned)
        .unwrap_or_default();
    app.state::<Database>()
        .with(|connection| settings_repo::set(connection, FOLLOW_MODE, &value))?;
    publish(app, app.state::<OverlayStore>().set_follow_mode(mode)?)?;
    crate::tray::refresh(app)
}

pub fn publish(app: &AppHandle, snapshot: OverlaySnapshot) -> Result<(), AppError> {
    let window = overlay_window(app)?;
    let should_show = snapshot.follow_mode != FollowMode::Hidden;
    if window.is_visible()? != should_show {
        if should_show {
            window.show()?;
        } else {
            window.hide()?;
        }
    }
    OverlayChanged(snapshot).emit_to(app, OVERLAY_LABEL)?;
    Ok(())
}

pub fn open_panel(app: &AppHandle) -> Result<(), AppError> {
    publish(app, app.state::<OverlayStore>().set_panel_open(true)?)?;
    set_keyboard_focus(app, true)
}

pub fn set_keyboard_focus(app: &AppHandle, focused: bool) -> Result<(), AppError> {
    platform::focus_overlay(&overlay_window(app)?, focused)
}

fn overlay_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    app.get_webview_window(OVERLAY_LABEL)
        .ok_or(AppError::WindowMissing(OVERLAY_LABEL))
}
