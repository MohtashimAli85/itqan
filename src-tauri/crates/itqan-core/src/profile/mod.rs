use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

mod repo;

use crate::db::settings as settings_repo;
use crate::error::AppError;

const ONBOARDED: &str = "onboarding_completed";
const MAX_NAME_LENGTH: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Motivator {
    Learning,
    Building,
    Health,
    Money,
    Recognition,
    Family,
    Freedom,
    Status,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CoachStyle {
    #[default]
    Mentor,
    Manager,
    Trainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MotivatorWeight {
    pub motivator: Motivator,
    pub weight: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: Option<String>,
    pub motivators: Vec<MotivatorWeight>,
    pub situation: Option<String>,
    pub free_hours_per_week: Option<u8>,
    pub age: Option<u8>,
    pub coach_style: CoachStyle,
    pub family_start_minute: Option<u16>,
    pub family_end_minute: Option<u16>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub fn normalize(weights: &[MotivatorWeight]) -> Result<Vec<MotivatorWeight>, AppError> {
    let mut seen = Vec::new();
    for weight in weights {
        if seen.contains(&weight.motivator) {
            return Err(AppError::InvalidInput("each motivator appears once".into()));
        }
        seen.push(weight.motivator);
    }
    let total: u32 = weights.iter().map(|weight| u32::from(weight.weight)).sum();
    if total == 0 {
        return Err(AppError::InvalidInput(
            "give at least one motivator some weight".into(),
        ));
    }
    let mut shares: Vec<(Motivator, u32, u32)> = weights
        .iter()
        .map(|weight| {
            let scaled = u32::from(weight.weight) * 100;
            (weight.motivator, scaled / total, scaled % total)
        })
        .collect();
    let assigned: u32 = shares.iter().map(|share| share.1).sum();
    let mut by_remainder: Vec<usize> = (0..shares.len()).collect();
    by_remainder.sort_by(|a, b| shares[*b].2.cmp(&shares[*a].2));
    for index in by_remainder.into_iter().take((100 - assigned) as usize) {
        shares[index].1 += 1;
    }
    Ok(shares
        .into_iter()
        .filter(|share| share.1 > 0)
        .map(|(motivator, weight, _)| MotivatorWeight {
            motivator,
            weight: u8::try_from(weight).unwrap_or(100),
        })
        .collect())
}

pub fn get(connection: &Connection) -> Result<Profile, AppError> {
    repo::get(connection)
}

pub fn save(
    connection: &Connection,
    profile: Profile,
    now: DateTime<Utc>,
) -> Result<Profile, AppError> {
    let name = profile
        .name
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty());
    if name
        .as_ref()
        .is_some_and(|name| name.chars().count() > MAX_NAME_LENGTH)
    {
        return Err(AppError::InvalidInput("that name is too long".into()));
    }
    if profile.age.is_some_and(|age| !(13..=110).contains(&age)) {
        return Err(AppError::InvalidInput("age is 13 to 110".into()));
    }
    if profile.free_hours_per_week.is_some_and(|hours| hours > 168) {
        return Err(AppError::InvalidInput("a week has 168 hours".into()));
    }
    match (profile.family_start_minute, profile.family_end_minute) {
        (Some(start), Some(end)) if start >= end || end > 1440 => {
            return Err(AppError::InvalidInput(
                "family time must end after it starts".into(),
            ));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(AppError::InvalidInput(
                "family time needs a start and an end".into(),
            ));
        }
        _ => {}
    }
    let profile = Profile {
        name,
        motivators: normalize(&profile.motivators)?,
        situation: profile
            .situation
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty()),
        updated_at: Some(now),
        ..profile
    };
    repo::save(connection, &profile)?;
    get(connection)
}

pub fn is_onboarded(connection: &Connection) -> Result<bool, AppError> {
    Ok(settings_repo::get(connection, ONBOARDED)?.as_deref() == Some("true"))
}

pub fn complete_onboarding(connection: &Connection) -> Result<(), AppError> {
    settings_repo::set(connection, ONBOARDED, "true")
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;

    fn weight(motivator: Motivator, weight: u8) -> MotivatorWeight {
        MotivatorWeight { motivator, weight }
    }

    #[test]
    fn weights_are_normalized_to_one_hundred() {
        let normalized = normalize(&[
            weight(Motivator::Learning, 40),
            weight(Motivator::Building, 35),
            weight(Motivator::Health, 25),
            weight(Motivator::Money, 0),
        ])
        .unwrap();
        assert_eq!(
            normalized,
            vec![
                weight(Motivator::Learning, 40),
                weight(Motivator::Building, 35),
                weight(Motivator::Health, 25),
            ]
        );

        let thirds = normalize(&[
            weight(Motivator::Learning, 1),
            weight(Motivator::Building, 1),
            weight(Motivator::Health, 1),
        ])
        .unwrap();
        assert_eq!(thirds.iter().map(|w| u32::from(w.weight)).sum::<u32>(), 100);
    }

    #[test]
    fn empty_or_duplicate_weights_are_rejected() {
        assert!(normalize(&[weight(Motivator::Money, 0)]).is_err());
        assert!(normalize(&[weight(Motivator::Money, 5), weight(Motivator::Money, 5)]).is_err());
    }

    #[test]
    fn save_round_trips_and_marks_onboarding() {
        let connection = test_connection();
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        let saved = save(
            &connection,
            Profile {
                name: Some("  Mohtashim ".into()),
                motivators: vec![
                    weight(Motivator::Learning, 2),
                    weight(Motivator::Building, 2),
                ],
                free_hours_per_week: Some(8),
                coach_style: CoachStyle::Manager,
                family_start_minute: Some(19 * 60),
                family_end_minute: Some(21 * 60),
                ..Profile::default()
            },
            now,
        )
        .unwrap();

        assert_eq!(saved.name.as_deref(), Some("Mohtashim"));
        assert_eq!(saved.motivators[0].weight, 50);
        assert_eq!(saved.coach_style, CoachStyle::Manager);
        assert_eq!(saved.updated_at, Some(now));

        assert!(!is_onboarded(&connection).unwrap());
        complete_onboarding(&connection).unwrap();
        assert!(is_onboarded(&connection).unwrap());
    }

    #[test]
    fn half_set_family_time_is_rejected() {
        let connection = test_connection();
        let profile = Profile {
            motivators: vec![weight(Motivator::Health, 1)],
            family_start_minute: Some(19 * 60),
            ..Profile::default()
        };
        assert!(save(&connection, profile, Utc::now()).is_err());
    }
}
