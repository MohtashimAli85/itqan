use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::Deserialize;
use specta::Type;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Type)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Debug, Default)]
pub struct HitAreas {
    rects: Mutex<Vec<Rect>>,
    version: AtomicU64,
}

impl HitAreas {
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    pub fn replace(&self, rects: Vec<Rect>) -> Result<(), AppError> {
        *self.rects.lock().map_err(|_| AppError::LockPoisoned)? = rects;
        self.version.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn contains(&self, x: f64, y: f64) -> Result<bool, AppError> {
        let rects = self.rects.lock().map_err(|_| AppError::LockPoisoned)?;
        Ok(rects.iter().any(|rect| rect.contains(x, y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORB: Rect = Rect {
        x: 100.0,
        y: 50.0,
        width: 56.0,
        height: 56.0,
    };

    #[test]
    fn rect_contains_points_inside_and_excludes_far_edges() {
        assert!(ORB.contains(100.0, 50.0));
        assert!(ORB.contains(155.9, 105.9));
        assert!(!ORB.contains(156.0, 60.0));
        assert!(!ORB.contains(99.9, 60.0));
    }

    #[test]
    fn hit_areas_start_empty_and_follow_replacements() {
        let hit_areas = HitAreas::default();
        assert!(!hit_areas.contains(120.0, 70.0).unwrap());

        hit_areas.replace(vec![ORB]).unwrap();
        assert!(hit_areas.contains(120.0, 70.0).unwrap());

        hit_areas.replace(Vec::new()).unwrap();
        assert!(!hit_areas.contains(120.0, 70.0).unwrap());
    }
}
