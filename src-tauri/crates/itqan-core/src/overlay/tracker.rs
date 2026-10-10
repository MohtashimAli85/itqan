use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, WebviewWindow};
use tauri_specta::Event;

use super::{publish, Activity, HitAreas, OverlayStore, OVERLAY_LABEL};
use crate::error::AppError;
use crate::platform;

const ACTIVE_TICK: Duration = Duration::from_millis(16);
const STILL_TICK: Duration = Duration::from_millis(100);
const HIDDEN_TICK: Duration = Duration::from_millis(500);
const STILL_AFTER: Duration = Duration::from_secs(2);
const INPUT_IS_ACTIVITY: Duration = Duration::from_secs(1);
const TYPING_STARTS: Duration = Duration::from_millis(800);
const TYPING_ENDS: Duration = Duration::from_secs(3);

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
    hit_version: Option<u64>,
    still_for: Duration,
}

pub fn typing_state(since_key: Option<Duration>, typing: bool) -> bool {
    match since_key {
        Some(since) if since < TYPING_STARTS => true,
        Some(since) if since > TYPING_ENDS => false,
        Some(_) => typing,
        None => false,
    }
}

pub fn pause_for(visible: bool, still_for: Duration) -> Duration {
    if !visible {
        HIDDEN_TICK
    } else if still_for >= STILL_AFTER {
        STILL_TICK
    } else {
        ACTIVE_TICK
    }
}

pub fn spawn(app: AppHandle, window: WebviewWindow) -> Result<(), AppError> {
    let mut tracker = Tracker::default();
    tracker.tick(&app, &window, ACTIVE_TICK)?;
    window.show()?;
    thread::Builder::new()
        .name("overlay-tracker".into())
        .spawn(move || {
            let mut pause = ACTIVE_TICK;
            loop {
                let visible = window.is_visible().unwrap_or(false);
                track_input(&app);
                if visible {
                    if let Err(error) = tracker.tick(&app, &window, pause) {
                        tracing::warn!(%error, "overlay tick failed");
                    }
                }
                pause = pause_for(visible, tracker.still_for);
                thread::sleep(pause);
            }
        })?;
    Ok(())
}

fn track_input(app: &AppHandle) {
    if platform::since_last_input().is_some_and(|since| since < INPUT_IS_ACTIVITY) {
        app.state::<Activity>().touch();
    }
    let store = app.state::<OverlayStore>();
    let Ok(snapshot) = store.snapshot() else {
        return;
    };
    let typing = !snapshot.panel_open && typing_state(platform::since_last_key(), snapshot.docked);
    if let Ok(Some(changed)) = store.set_typing(typing) {
        if let Err(error) = publish(app, changed) {
            tracing::warn!(%error, "typing dock failed");
        }
    }
}

impl Tracker {
    fn tick(
        &mut self,
        app: &AppHandle,
        window: &WebviewWindow,
        elapsed: Duration,
    ) -> Result<(), AppError> {
        let cursor = app.cursor_position()?;
        let moved = self.last_cursor != Some(cursor);
        if moved || self.monitor_origin.is_none() {
            if let Some(monitor) = monitor_at(window, cursor)? {
                if self.monitor_origin != Some(*monitor.position()) {
                    self.fit_to(window, &monitor)?;
                }
            }
        }
        let Some(origin) = self.monitor_origin else {
            return Ok(());
        };
        let x = (cursor.x - f64::from(origin.x)) / self.scale_factor;
        let y = (cursor.y - f64::from(origin.y)) / self.scale_factor;

        let hit_areas = app.state::<HitAreas>();
        let version = hit_areas.version();
        if moved || self.hit_version != Some(version) {
            self.hit_version = Some(version);
            let interactive = hit_areas.contains(x, y)?;
            if interactive != self.interactive {
                window.set_ignore_cursor_events(!interactive)?;
                self.interactive = interactive;
            }
        }

        if moved {
            self.still_for = Duration::ZERO;
            self.last_cursor = Some(cursor);
            app.state::<Activity>().touch();
            OverlayCursor {
                x: x.round() as i32,
                y: y.round() as i32,
            }
            .emit_to(app, OVERLAY_LABEL)?;
        } else {
            self.still_for += elapsed;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polling_slows_down_when_the_cursor_rests() {
        assert_eq!(pause_for(true, Duration::ZERO), ACTIVE_TICK);
        assert_eq!(pause_for(true, Duration::from_secs(3)), STILL_TICK);
        assert_eq!(pause_for(false, Duration::ZERO), HIDDEN_TICK);
    }

    #[test]
    fn typing_has_hysteresis() {
        assert!(typing_state(Some(Duration::from_millis(200)), false));
        assert!(typing_state(Some(Duration::from_secs(2)), true));
        assert!(!typing_state(Some(Duration::from_secs(2)), false));
        assert!(!typing_state(Some(Duration::from_secs(4)), true));
        assert!(!typing_state(None, true));
    }
}
