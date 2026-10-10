mod repo;

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;
use crate::profile::{normalize, CoachStyle, Motivator, MotivatorWeight};

pub type BeliefId = i32;

const MOTIVATOR_PREFIX: &str = "motivator.";
const COACH_STYLE: &str = "coach.style";
const SAID_CONFIDENCE: f64 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BeliefKind {
    Motivator,
    Preference,
    Pattern,
    Constraint,
    Goal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Strength {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BeliefSource {
    Said,
    Observed,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BeliefStatus {
    Active,
    Proposed,
    Rejected,
    Archived,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Belief {
    pub id: BeliefId,
    pub statement: String,
    pub kind: BeliefKind,
    pub subject: String,
    pub value: Option<serde_json::Value>,
    pub strength: Strength,
    pub confidence: f64,
    pub source: BeliefSource,
    pub evidence: Vec<String>,
    pub status: BeliefStatus,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewBelief {
    pub statement: String,
    pub kind: BeliefKind,
    pub subject: String,
    pub value: Option<serde_json::Value>,
    pub strength: Strength,
    pub confidence: f64,
    pub source: BeliefSource,
    pub status: BeliefStatus,
}

pub fn strength_for_weight(weight: u8) -> Strength {
    match weight {
        30.. => Strength::High,
        15.. => Strength::Medium,
        _ => Strength::Low,
    }
}

fn motivator_name(motivator: Motivator) -> Result<String, AppError> {
    crate::db::enums::to_text(&motivator)
}

fn motivator_subject(motivator: Motivator) -> Result<String, AppError> {
    Ok(format!("{MOTIVATOR_PREFIX}{}", motivator_name(motivator)?))
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

pub fn active(connection: &Connection) -> Result<Vec<Belief>, AppError> {
    repo::with_status(connection, BeliefStatus::Active)
}

pub fn motivator_weights(connection: &Connection) -> Result<Vec<MotivatorWeight>, AppError> {
    let weights: Vec<MotivatorWeight> = active(connection)?
        .into_iter()
        .filter_map(|belief| {
            let name = belief.subject.strip_prefix(MOTIVATOR_PREFIX)?;
            let motivator = serde_json::from_value(serde_json::Value::String(name.into())).ok()?;
            let weight = belief.value?.get("weight")?.as_u64()?;
            Some(MotivatorWeight {
                motivator,
                weight: u8::try_from(weight).unwrap_or(u8::MAX),
            })
        })
        .filter(|weight| weight.weight > 0)
        .collect();
    if weights.is_empty() {
        return Ok(weights);
    }
    normalize(&weights)
}

pub fn coach_style(connection: &Connection) -> Result<CoachStyle, AppError> {
    Ok(repo::active_for(connection, COACH_STYLE)?
        .and_then(|belief| belief.value?.get("style").cloned())
        .and_then(|style| serde_json::from_value(style).ok())
        .unwrap_or_default())
}

pub fn say_motivators(
    connection: &Connection,
    weights: &[MotivatorWeight],
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let mut wanted = Vec::new();
    for weight in weights.iter().filter(|weight| weight.weight > 0) {
        wanted.push((motivator_subject(weight.motivator)?, *weight));
    }
    for belief in active(connection)? {
        if !belief.subject.starts_with(MOTIVATOR_PREFIX) {
            continue;
        }
        let unchanged = wanted.iter().any(|(subject, weight)| {
            *subject == belief.subject
                && belief
                    .value
                    .as_ref()
                    .and_then(|value| value.get("weight"))
                    .and_then(serde_json::Value::as_u64)
                    == Some(u64::from(weight.weight))
        });
        if unchanged {
            wanted.retain(|(subject, _)| *subject != belief.subject);
        } else {
            repo::set_status(connection, belief.id, BeliefStatus::Archived, now)?;
        }
    }
    for (subject, weight) in wanted {
        repo::insert(
            connection,
            &NewBelief {
                statement: format!(
                    "{} matters to me",
                    capitalised(&motivator_name(weight.motivator)?)
                ),
                kind: BeliefKind::Motivator,
                subject,
                value: Some(serde_json::json!({ "weight": weight.weight })),
                strength: strength_for_weight(weight.weight),
                confidence: SAID_CONFIDENCE,
                source: BeliefSource::Said,
                status: BeliefStatus::Active,
            },
            now,
        )?;
    }
    Ok(())
}

pub fn say_coach_style(
    connection: &Connection,
    style: CoachStyle,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let name = crate::db::enums::to_text(&style)?;
    if let Some(current) = repo::active_for(connection, COACH_STYLE)? {
        let same = current
            .value
            .as_ref()
            .and_then(|value| value.get("style"))
            .and_then(serde_json::Value::as_str)
            == Some(name.as_str());
        if same {
            return Ok(());
        }
        repo::set_status(connection, current.id, BeliefStatus::Archived, now)?;
    }
    repo::insert(
        connection,
        &NewBelief {
            statement: format!("Coach me like a {name}"),
            kind: BeliefKind::Preference,
            subject: COACH_STYLE.into(),
            value: Some(serde_json::json!({ "style": name })),
            strength: Strength::Medium,
            confidence: SAID_CONFIDENCE,
            source: BeliefSource::Said,
            status: BeliefStatus::Active,
        },
        now,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap()
    }

    fn weight(motivator: Motivator, weight: u8) -> MotivatorWeight {
        MotivatorWeight { motivator, weight }
    }

    fn before_beliefs() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        crate::db::migrations::run_until(&mut connection, 11).unwrap();
        connection
    }

    #[test]
    fn the_sliders_become_said_beliefs_and_the_profile_reads_the_same() {
        let mut connection = before_beliefs();
        connection
            .execute(
                "UPDATE profile SET motivators = ?1, coach_style = 'trainer', updated_at = ?2 WHERE id = 1",
                rusqlite::params![
                    r#"[{"motivator":"learning","weight":40},{"motivator":"building","weight":35},{"motivator":"health","weight":25}]"#,
                    now(),
                ],
            )
            .unwrap();
        crate::db::settings::set(&connection, "onboarding_completed", "true").unwrap();

        crate::db::migrations::run(&mut connection).unwrap();

        let profile = crate::profile::get(&connection).unwrap();
        assert_eq!(
            profile.motivators,
            vec![
                weight(Motivator::Learning, 40),
                weight(Motivator::Building, 35),
                weight(Motivator::Health, 25),
            ]
        );
        assert_eq!(profile.coach_style, CoachStyle::Trainer);
        let beliefs = active(&connection).unwrap();
        assert_eq!(beliefs.len(), 4);
        assert!(beliefs
            .iter()
            .all(|belief| belief.source == BeliefSource::Said
                && (belief.confidence - 0.8).abs() < f64::EPSILON
                && belief.created_at == now()));
        let strengths: Vec<(String, Strength)> = beliefs
            .iter()
            .map(|belief| (belief.subject.clone(), belief.strength))
            .collect();
        assert!(strengths.contains(&("motivator.learning".into(), Strength::High)));
        assert!(strengths.contains(&("motivator.health".into(), Strength::Medium)));
        assert_eq!(beliefs[0].statement, "Learning matters to me");
    }

    #[test]
    fn a_profile_nobody_filled_in_migrates_to_nothing() {
        let mut connection = before_beliefs();

        crate::db::migrations::run(&mut connection).unwrap();

        assert!(active(&connection).unwrap().is_empty());
        let profile = crate::profile::get(&connection).unwrap();
        assert!(profile.motivators.is_empty());
        assert_eq!(profile.coach_style, CoachStyle::Mentor);
    }

    #[test]
    fn nothing_said_means_no_weights_and_the_mentor_style() {
        let connection = test_connection();
        assert!(motivator_weights(&connection).unwrap().is_empty());
        assert_eq!(coach_style(&connection).unwrap(), CoachStyle::Mentor);
    }

    #[test]
    fn saying_motivators_replaces_the_whole_set() {
        let connection = test_connection();
        say_motivators(
            &connection,
            &[
                weight(Motivator::Building, 60),
                weight(Motivator::Learning, 40),
            ],
            now(),
        )
        .unwrap();
        say_motivators(&connection, &[weight(Motivator::Building, 100)], now()).unwrap();

        assert_eq!(
            motivator_weights(&connection).unwrap(),
            vec![weight(Motivator::Building, 100)]
        );
        let archived = repo::with_status(&connection, BeliefStatus::Archived).unwrap();
        assert_eq!(archived.len(), 2);
    }

    #[test]
    fn an_unchanged_weight_keeps_its_belief() {
        let connection = test_connection();
        let set = [weight(Motivator::Health, 50), weight(Motivator::Family, 50)];
        say_motivators(&connection, &set, now()).unwrap();
        say_motivators(&connection, &set, now()).unwrap();

        assert_eq!(active(&connection).unwrap().len(), 2);
        assert!(repo::with_status(&connection, BeliefStatus::Archived)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn relative_weights_are_normalised_to_one_hundred() {
        let connection = test_connection();
        for (motivator, relative) in [(Motivator::Learning, 3), (Motivator::Health, 1)] {
            repo::insert(
                &connection,
                &NewBelief {
                    statement: "x".into(),
                    kind: BeliefKind::Motivator,
                    subject: motivator_subject(motivator).unwrap(),
                    value: Some(serde_json::json!({ "weight": relative })),
                    strength: Strength::Medium,
                    confidence: 0.8,
                    source: BeliefSource::Said,
                    status: BeliefStatus::Active,
                },
                now(),
            )
            .unwrap();
        }

        assert_eq!(
            motivator_weights(&connection).unwrap(),
            vec![
                weight(Motivator::Learning, 75),
                weight(Motivator::Health, 25)
            ]
        );
    }

    #[test]
    fn one_active_belief_per_subject() {
        let connection = test_connection();
        say_coach_style(&connection, CoachStyle::Trainer, now()).unwrap();
        say_coach_style(&connection, CoachStyle::Manager, now()).unwrap();

        assert_eq!(coach_style(&connection).unwrap(), CoachStyle::Manager);
        let duplicate = repo::insert(
            &connection,
            &NewBelief {
                statement: "x".into(),
                kind: BeliefKind::Preference,
                subject: COACH_STYLE.into(),
                value: None,
                strength: Strength::Low,
                confidence: 0.5,
                source: BeliefSource::Said,
                status: BeliefStatus::Active,
            },
            now(),
        );
        assert!(duplicate.is_err());
    }
}
