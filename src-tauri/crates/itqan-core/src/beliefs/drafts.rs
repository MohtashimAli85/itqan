use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{BeliefKind, BeliefSource, BeliefStatus, NewBelief, Strength, NOTE, SAID_CONFIDENCE};
use crate::error::AppError;
use crate::profile::{CoachStyle, Motivator};

const MAX_DRAFTS: usize = 12;
const MAX_STATEMENT: usize = 200;
pub const MAX_ANSWER: usize = 1000;

pub const SYSTEM: &str = "You help a personal productivity coach understand its user. \
You get the user's answers to a few onboarding questions, in English or Roman Urdu. \
Turn them into short beliefs about the user. Write each statement in the user's own words and language style, first person, at most one sentence. \
Reply with JSON only: {\"beliefs\": [{\"statement\": string, \"kind\": \"motivator\"|\"preference\"|\"pattern\"|\"constraint\"|\"goal\", \"subject\": string, \"strength\": \"low\"|\"medium\"|\"high\", \"style\": string}]}. \
subject is one of: motivator.learning, motivator.building, motivator.health, motivator.money, motivator.recognition, motivator.family, motivator.freedom, motivator.status, coach.style, note. \
Use coach.style only for how the user wants to be spoken to, with style set to mentor (supportive), manager (direct, deadlines) or trainer (no excuses). \
Use note for everything else. Never invent facts the user did not say. At most 12 beliefs.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DraftSubject {
    Motivator { motivator: Motivator },
    CoachStyle { style: CoachStyle },
    Note,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DraftBelief {
    pub statement: String,
    pub kind: BeliefKind,
    pub subject: DraftSubject,
    pub strength: Strength,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub question: String,
    pub answer: String,
}

#[derive(Deserialize)]
struct Reply {
    beliefs: Vec<RawDraft>,
}

#[derive(Deserialize)]
struct RawDraft {
    statement: Option<String>,
    kind: Option<String>,
    subject: Option<String>,
    strength: Option<String>,
    style: Option<String>,
}

fn parse_text<T: serde::de::DeserializeOwned>(text: Option<&str>) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(text?.trim().to_owned())).ok()
}

fn clean(statement: &str) -> String {
    statement
        .trim()
        .chars()
        .take(MAX_STATEMENT)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn subject_of(raw: &RawDraft) -> DraftSubject {
    let subject = raw.subject.as_deref().unwrap_or_default().trim();
    if let Some(name) = subject.strip_prefix("motivator.") {
        if let Some(motivator) = parse_text(Some(name)) {
            return DraftSubject::Motivator { motivator };
        }
    }
    if subject == "coach.style" {
        if let Some(style) = parse_text(raw.style.as_deref()) {
            return DraftSubject::CoachStyle { style };
        }
    }
    DraftSubject::Note
}

pub fn parse(reply: &str) -> Vec<DraftBelief> {
    let Some(json) = reply
        .find('{')
        .zip(reply.rfind('}'))
        .and_then(|(start, end)| reply.get(start..=end))
    else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<Reply>(json) else {
        return Vec::new();
    };
    let mut drafts: Vec<DraftBelief> = Vec::new();
    for raw in parsed.beliefs {
        let statement = clean(raw.statement.as_deref().unwrap_or_default());
        if statement.is_empty() {
            continue;
        }
        let subject = subject_of(&raw);
        let duplicate =
            subject != DraftSubject::Note && drafts.iter().any(|draft| draft.subject == subject);
        let style_twice = matches!(subject, DraftSubject::CoachStyle { .. })
            && drafts
                .iter()
                .any(|draft| matches!(draft.subject, DraftSubject::CoachStyle { .. }));
        if duplicate || style_twice {
            continue;
        }
        let kind = match subject {
            DraftSubject::Motivator { .. } => BeliefKind::Motivator,
            DraftSubject::CoachStyle { .. } => BeliefKind::Preference,
            DraftSubject::Note => parse_text(raw.kind.as_deref()).unwrap_or(BeliefKind::Pattern),
        };
        drafts.push(DraftBelief {
            statement,
            kind,
            subject,
            strength: parse_text(raw.strength.as_deref()).unwrap_or(Strength::Medium),
        });
        if drafts.len() == MAX_DRAFTS {
            break;
        }
    }
    drafts
}

pub fn prompt(answers: &[Answer]) -> String {
    answers
        .iter()
        .filter(|answer| !answer.answer.trim().is_empty())
        .map(|answer| {
            let text: String = answer.answer.trim().chars().take(MAX_ANSWER).collect();
            let redacted = crate::ai::redact::redact(&text);
            format!("Q: {}\nA: <<<{}>>>", answer.question.trim(), redacted.text)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn save_notes(
    connection: &Connection,
    drafts: &[DraftBelief],
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    for draft in drafts {
        if draft.subject != DraftSubject::Note {
            continue;
        }
        let statement = clean(&draft.statement);
        if statement.is_empty() {
            continue;
        }
        super::repo::insert(
            connection,
            &NewBelief {
                statement,
                kind: draft.kind,
                subject: NOTE.into(),
                value: None,
                strength: draft.strength,
                confidence: SAID_CONFIDENCE,
                source: BeliefSource::Said,
                status: BeliefStatus::Active,
            },
            now,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;

    #[test]
    fn known_subjects_are_typed_and_the_rest_become_notes() {
        let reply = r#"Sure! {"beliefs": [
            {"statement": " Getting good at Rust matters a lot ", "kind": "motivator", "subject": "motivator.learning", "strength": "high"},
            {"statement": "Be direct with me", "kind": "preference", "subject": "coach.style", "style": "manager", "strength": "medium"},
            {"statement": "I lose evenings to YouTube", "kind": "pattern", "subject": "distraction.youtube", "strength": "medium"},
            {"statement": "Wants to be rich", "kind": "motivator", "subject": "motivator.yachts", "strength": "loud"},
            {"statement": "", "kind": "goal", "subject": "note"},
            {"statement": "Learning again", "kind": "motivator", "subject": "motivator.learning", "strength": "low"}
        ]} thanks"#;

        let drafts = parse(reply);

        assert_eq!(drafts.len(), 4);
        assert_eq!(drafts[0].statement, "Getting good at Rust matters a lot");
        assert_eq!(
            drafts[0].subject,
            DraftSubject::Motivator {
                motivator: Motivator::Learning
            }
        );
        assert_eq!(drafts[0].strength, Strength::High);
        assert_eq!(
            drafts[1].subject,
            DraftSubject::CoachStyle {
                style: CoachStyle::Manager
            }
        );
        assert_eq!(drafts[1].kind, BeliefKind::Preference);
        assert_eq!(drafts[2].subject, DraftSubject::Note);
        assert_eq!(drafts[2].kind, BeliefKind::Pattern);
        assert_eq!(drafts[3].subject, DraftSubject::Note);
        assert_eq!(drafts[3].strength, Strength::Medium);
    }

    #[test]
    fn garbage_gives_no_drafts() {
        assert!(parse("I can't help with that").is_empty());
        assert!(parse("{\"beliefs\": 3}").is_empty());
    }

    #[test]
    fn answers_are_redacted_before_they_leave() {
        let text = prompt(&[
            Answer {
                question: "What pulls you off track?".into(),
                answer: "Calls from boss@example.com".into(),
            },
            Answer {
                question: "Skipped".into(),
                answer: "   ".into(),
            },
        ]);
        assert!(!text.contains("boss@example.com"));
        assert!(!text.contains("Skipped"));
    }

    #[test]
    fn only_notes_are_saved_here() {
        let connection = test_connection();
        let drafts = parse(
            r#"{"beliefs": [
                {"statement": "Learning matters", "subject": "motivator.learning"},
                {"statement": "I focus best before noon", "kind": "pattern", "subject": "note"}
            ]}"#,
        );

        save_notes(&connection, &drafts, Utc::now()).unwrap();

        let active = super::super::active(&connection).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].statement, "I focus best before noon");
        assert_eq!(active[0].subject, NOTE);
    }
}
