pub mod commands;
pub mod drafts;
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
pub(crate) const SAID_CONFIDENCE: f64 = 0.8;
const CONFIRMED_CONFIDENCE: f64 = 0.9;
pub(crate) const NOTE: &str = "note";

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

pub fn insert(
    connection: &Connection,
    belief: &NewBelief,
    now: DateTime<Utc>,
) -> Result<BeliefId, AppError> {
    repo::insert(connection, belief, now)
}

pub fn find(connection: &Connection, id: BeliefId) -> Result<Belief, AppError> {
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("belief {id}")))
}

pub fn confirm(connection: &Connection, id: BeliefId, now: DateTime<Utc>) -> Result<(), AppError> {
    let belief = find(connection, id)?;
    if matches!(
        belief.status,
        BeliefStatus::Rejected | BeliefStatus::Archived
    ) {
        return Err(AppError::InvalidInput(format!(
            "belief {id} is no longer open"
        )));
    }
    if belief.subject != NOTE {
        if let Some(current) = repo::active_for(connection, &belief.subject)? {
            if current.id != id {
                repo::set_status(connection, current.id, BeliefStatus::Archived, now)?;
            }
        }
    }
    repo::confirm(
        connection,
        id,
        CONFIRMED_CONFIDENCE.max(belief.confidence),
        now,
    )
}

pub fn reject(connection: &Connection, id: BeliefId, now: DateTime<Utc>) -> Result<(), AppError> {
    repo::set_status(connection, id, BeliefStatus::Rejected, now)
}

pub fn active(connection: &Connection) -> Result<Vec<Belief>, AppError> {
    repo::with_status(connection, BeliefStatus::Active)
}

fn relative_weight(value: Option<&serde_json::Value>) -> Option<u8> {
    let weight = value?.get("weight")?.as_f64()?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let weight = weight.round().clamp(0.0, 100.0) as u8;
    (weight > 0).then_some(weight)
}

fn motivator_weight(belief: &Belief) -> Option<MotivatorWeight> {
    let name = belief.subject.strip_prefix(MOTIVATOR_PREFIX)?;
    let motivator = serde_json::from_value(serde_json::Value::String(name.into())).ok()?;
    Some(MotivatorWeight {
        motivator,
        weight: relative_weight(belief.value.as_ref())?,
    })
}

fn style_of(belief: &Belief) -> Option<CoachStyle> {
    serde_json::from_value(belief.value.as_ref()?.get("style")?.clone()).ok()
}

pub fn profile_parts(
    connection: &Connection,
) -> Result<(Vec<MotivatorWeight>, CoachStyle), AppError> {
    let beliefs = active(connection)?;
    let mut weights: Vec<MotivatorWeight> = beliefs.iter().filter_map(motivator_weight).collect();
    weights.sort_by_key(|weight| weight.motivator);
    let style = beliefs
        .iter()
        .find(|belief| belief.subject == COACH_STYLE)
        .and_then(style_of)
        .unwrap_or_default();
    let weights = if weights.is_empty() {
        weights
    } else {
        normalize(&weights)?
    };
    Ok((weights, style))
}

pub fn motivator_weights(connection: &Connection) -> Result<Vec<MotivatorWeight>, AppError> {
    Ok(profile_parts(connection)?.0)
}

pub fn coach_style(connection: &Connection) -> Result<CoachStyle, AppError> {
    Ok(repo::active_for(connection, COACH_STYLE)?
        .as_ref()
        .and_then(style_of)
        .unwrap_or_default())
}

pub fn motivator_statement(motivator: Motivator) -> Result<String, AppError> {
    Ok(format!(
        "{} matters to me",
        capitalised(&motivator_name(motivator)?)
    ))
}

pub fn coach_style_statement(style: CoachStyle) -> Result<String, AppError> {
    Ok(format!(
        "Coach me like a {}",
        crate::db::enums::to_text(&style)?
    ))
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
        let current = relative_weight(belief.value.as_ref());
        let unchanged = wanted
            .iter()
            .any(|(subject, weight)| *subject == belief.subject && current == Some(weight.weight));
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
                statement: motivator_statement(weight.motivator)?,
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
            statement: coach_style_statement(style)?,
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
        for belief in &beliefs {
            if let Some(weight) = motivator_weight(belief) {
                assert_eq!(
                    belief.statement,
                    motivator_statement(weight.motivator).unwrap()
                );
                assert_eq!(belief.strength, strength_for_weight(weight.weight));
            } else {
                assert_eq!(
                    belief.statement,
                    coach_style_statement(CoachStyle::Trainer).unwrap()
                );
            }
        }
    }

    #[test]
    fn a_style_saved_before_onboarding_finished_is_kept() {
        let mut connection = before_beliefs();
        connection
            .execute(
                "UPDATE profile SET coach_style = 'manager', updated_at = ?1 WHERE id = 1",
                [now()],
            )
            .unwrap();

        crate::db::migrations::run(&mut connection).unwrap();

        assert_eq!(coach_style(&connection).unwrap(), CoachStyle::Manager);
    }

    #[test]
    fn odd_weights_are_rounded_or_dropped() {
        assert_eq!(
            relative_weight(Some(&serde_json::json!({ "weight": 40.4 }))),
            Some(40)
        );
        assert_eq!(
            relative_weight(Some(&serde_json::json!({ "weight": 300 }))),
            Some(100)
        );
        assert_eq!(
            relative_weight(Some(&serde_json::json!({ "weight": 0 }))),
            None
        );
        assert_eq!(
            relative_weight(Some(&serde_json::json!({ "weight": "high" }))),
            None
        );
    }

    #[test]
    fn weights_come_back_in_motivator_order() {
        let connection = test_connection();
        say_motivators(
            &connection,
            &[weight(Motivator::Health, 50), weight(Motivator::Family, 50)],
            now(),
        )
        .unwrap();
        say_motivators(
            &connection,
            &[
                weight(Motivator::Learning, 50),
                weight(Motivator::Health, 50),
            ],
            now(),
        )
        .unwrap();

        assert_eq!(
            motivator_weights(&connection).unwrap(),
            vec![
                weight(Motivator::Learning, 50),
                weight(Motivator::Health, 50)
            ]
        );
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
