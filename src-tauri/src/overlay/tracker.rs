use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, WebviewWindow};
use tauri_specta::Event;

use super::{Activity, HitAreas, OVERLAY_LABEL};
use crate::error::AppError;

const TICK: Duration = Duration::from_millis(16);
const HIDDEN_TICK: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct OverlayCursor {
    pub x: i32,
    pub y: i32,
}

#[derive(Default)]
struct Tracker {
    last_cursor: Option<PhysicalPosition<f64>>,
    monitor_origin: Option<PhysicalPosition<i32>>,
    scale_factor: f64,
    interactive: bool,
}

pub fn spawn(app: AppHandle, window: WebviewWindow) -> Result<(), AppError> {
    let mut tracker = Tracker::default();
    tracker.tick(&app, &window)?;
    window.show()?;
    thread::Builder::new()
        .name("overlay-tracker".into())
        .spawn(move || loop {
            let visible = window.is_visible().unwrap_or(false);
            if visible {
                if let Err(error) = tracker.tick(&app, &window) {
                    tracing::warn!(%error, "overlay tick failed");
                }
            }
            thread::sleep(if visible { TICK } else { HIDDEN_TICK });
        })?;
    Ok(())
}

impl Tracker {
    fn tick(&mut self, app: &AppHandle, window: &WebviewWindow) -> Result<(), AppError> {
        let cursor = app.cursor_position()?;
        if let Some(monitor) = monitor_at(window, cursor)? {
            if self.monitor_origin != Some(*monitor.position()) {
                self.fit_to(window, &monitor)?;
            }
        }
        let Some(origin) = self.monitor_origin else {
            return Ok(());
        };
        let x = (cursor.x - f64::from(origin.x)) / self.scale_factor;
        let y = (cursor.y - f64::from(origin.y)) / self.scale_factor;

        let interactive = app.state::<HitAreas>().contains(x, y)?;
        if interactive != self.interactive {
            window.set_ignore_cursor_events(!interactive)?;
            self.interactive = interactive;
        }

        if self.last_cursor != Some(cursor) {
            self.last_cursor = Some(cursor);
            app.state::<Activity>().touch();
            OverlayCursor {
                x: x.round() as i32,
                y: y.round() as i32,
            }
            .emit_to(app, OVERLAY_LABEL)?;
        }
        Ok(())
    }

    fn fit_to(&mut self, window: &WebviewWindow, monitor: &Monitor) -> Result<(), AppError> {
        window.set_position(*monitor.position())?;
        window.set_size(*monitor.size())?;
        self.monitor_origin = Some(*monitor.position());
        self.scale_factor = monitor.scale_factor();
        Ok(())
    }
}

fn monitor_at(
    window: &WebviewWindow,
    cursor: PhysicalPosition<f64>,
) -> Result<Option<Monitor>, AppError> {
    let monitors = window.available_monitors()?;
    let under_cursor = monitors.into_iter().find(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        let left = f64::from(position.x);
        let top = f64::from(position.y);
        cursor.x >= left
            && cursor.x < left + f64::from(size.width)
            && cursor.y >= top
            && cursor.y < top + f64::from(size.height)
    });
    match under_cursor {
        Some(monitor) => Ok(Some(monitor)),
        None => Ok(window.current_monitor()?),
    }
}
