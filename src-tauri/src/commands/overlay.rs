use tauri::State;

use crate::error::CommandError;
use crate::overlay::{HitAreas, Rect};

#[tauri::command]
#[specta::specta]
pub fn set_overlay_hit_areas(
    hit_areas: State<HitAreas>,
    rects: Vec<Rect>,
) -> Result<(), CommandError> {
    Ok(hit_areas.replace(rects)?)
}
