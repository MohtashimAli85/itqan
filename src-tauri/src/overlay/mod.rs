mod activity;
mod hit_areas;
mod state;
mod tracker;

use tauri::{App, AppHandle, Manager, WebviewWindow};
use tauri_specta::Event;

use crate::error::AppError;
use crate::platform;

pub use activity::Activity;
pub use hit_areas::{HitAreas, Rect};
pub use state::{
    BubbleAction, BubbleOrigin, FollowMode, OrbState, OverlayChanged, OverlaySnapshot, OverlayStore,
};
pub use tracker::OverlayCursor;

pub const OVERLAY_LABEL: &str = "overlay";

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

pub fn set_keyboard_focus(app: &AppHandle, focused: bool) -> Result<(), AppError> {
    platform::focus_overlay(&overlay_window(app)?, focused)
}

fn overlay_window(app: &AppHandle) -> Result<WebviewWindow, AppError> {
    app.get_webview_window(OVERLAY_LABEL)
        .ok_or(AppError::WindowMissing(OVERLAY_LABEL))
}
