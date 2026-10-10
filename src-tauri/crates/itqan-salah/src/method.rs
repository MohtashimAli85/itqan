use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    MuslimWorldLeague,
    Egyptian,
    #[default]
    Karachi,
    UmmAlQura,
    Dubai,
    NorthAmerica,
    Kuwait,
    Qatar,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Madhab {
    Shafi,
    #[default]
    Hanafi,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HighLatitudeRule {
    #[default]
    MiddleOfTheNight,
    SeventhOfTheNight,
    TwilightAngle,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Adjustments {
    pub fajr: i64,
    pub sunrise: i64,
    pub dhuhr: i64,
    pub asr: i64,
    pub maghrib: i64,
    pub isha: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameters {
    pub fajr_angle: f64,
    pub isha_angle: f64,
    pub isha_interval_minutes: i64,
    pub madhab: Madhab,
    pub high_latitude_rule: HighLatitudeRule,
    pub adjustments: Adjustments,
}

impl Madhab {
    pub fn shadow_length(self) -> f64 {
        match self {
            Self::Shafi => 1.0,
            Self::Hanafi => 2.0,
        }
    }
}

impl Parameters {
    pub fn new(method: Method, madhab: Madhab, high_latitude_rule: HighLatitudeRule) -> Self {
        let dhuhr_one = Adjustments {
            dhuhr: 1,
            ..Adjustments::default()
        };
        let (fajr_angle, isha_angle, isha_interval_minutes, adjustments) = match method {
            Method::MuslimWorldLeague => (18.0, 17.0, 0, dhuhr_one),
            Method::Egyptian => (19.5, 17.5, 0, dhuhr_one),
            Method::Karachi => (18.0, 18.0, 0, dhuhr_one),
            Method::UmmAlQura => (18.5, 0.0, 90, Adjustments::default()),
            Method::Dubai => (
                18.2,
                18.2,
                0,
                Adjustments {
                    sunrise: -3,
                    dhuhr: 3,
                    asr: 3,
                    maghrib: 3,
                    ..Adjustments::default()
                },
            ),
            Method::NorthAmerica => (15.0, 15.0, 0, dhuhr_one),
            Method::Kuwait => (18.0, 17.5, 0, Adjustments::default()),
            Method::Qatar => (18.0, 0.0, 90, Adjustments::default()),
        };
        Self {
            fajr_angle,
            isha_angle,
            isha_interval_minutes,
            madhab,
            high_latitude_rule,
            adjustments,
        }
    }

    pub fn night_portions(&self) -> (f64, f64) {
        match self.high_latitude_rule {
            HighLatitudeRule::MiddleOfTheNight => (0.5, 0.5),
            HighLatitudeRule::SeventhOfTheNight => (1.0 / 7.0, 1.0 / 7.0),
            HighLatitudeRule::TwilightAngle => (self.fajr_angle / 60.0, self.isha_angle / 60.0),
        }
    }
}
