use serde::{Deserialize, Serialize};
use specta::Type;

use super::method::{HighLatitudeRule, Madhab, Method};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrayerSettings {
    pub enabled: bool,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub method: Method,
    pub madhab: Madhab,
    pub high_latitude_rule: HighLatitudeRule,
    pub pause_before_minutes: u16,
    pub pause_after_minutes: u16,
    pub jumuah_break: bool,
}

impl Default for PrayerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            city: None,
            latitude: None,
            longitude: None,
            method: Method::default(),
            madhab: Madhab::default(),
            high_latitude_rule: HighLatitudeRule::default(),
            pause_before_minutes: 5,
            pause_after_minutes: 20,
            jumuah_break: true,
        }
    }
}
