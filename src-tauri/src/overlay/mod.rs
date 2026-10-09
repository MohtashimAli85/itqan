mod hit_areas;
mod tracker;

use tauri::{App, Manager};

use crate::error::AppError;
use crate::platform;

pub use hit_areas::{HitAreas, Rect};
pub use tracker::OverlayCursor;

pub const OVERLAY_LABEL: &str = "overlay";

pub fn setup(app: &App) -> Result<(), AppError> {
    let window = app
        .get_webview_window(OVERLAY_LABEL)
        .ok_or(AppError::WindowMissing(OVERLAY_LABEL))?;
    platform::configure_overlay(&window)?;
    window.set_ignore_cursor_events(true)?;
    app.manage(HitAreas::default());
    tracker::spawn(app.handle().clone(), window)?;
    Ok(())
}
