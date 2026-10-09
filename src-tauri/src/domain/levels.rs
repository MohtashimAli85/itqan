use serde::{Deserialize, Serialize};
use specta::Type;

const STEP: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LevelProgress {
    pub level: u32,
    pub xp_into_level: u32,
    pub xp_for_next: u32,
}

pub fn xp_to_reach(level: u32) -> u32 {
    let completed = level.saturating_sub(1);
    STEP * completed * (completed + 1) / 2
}

pub fn progress(xp: u32) -> LevelProgress {
    let mut level = 1;
    while xp >= xp_to_reach(level + 1) {
        level += 1;
    }
    LevelProgress {
        level,
        xp_into_level: xp - xp_to_reach(level),
        xp_for_next: xp_to_reach(level + 1) - xp_to_reach(level),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_get_steadily_longer() {
        assert_eq!(progress(0).level, 1);
        assert_eq!(progress(99).level, 1);
        assert_eq!(progress(100).level, 2);
        assert_eq!(progress(300).level, 3);
        assert_eq!(
            progress(350),
            LevelProgress {
                level: 3,
                xp_into_level: 50,
                xp_for_next: 300,
            }
        );
    }
}
