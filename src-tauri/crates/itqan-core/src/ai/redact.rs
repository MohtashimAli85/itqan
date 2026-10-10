use std::sync::LazyLock;

use regex::Regex;

struct Pattern {
    label: &'static str,
    regex: Regex,
}

#[allow(clippy::expect_used)]
static PATTERNS: LazyLock<Vec<Pattern>> = LazyLock::new(|| {
    [
        (
            "KEY",
            r"\b(?:sk-[A-Za-z0-9_-]{16,}|gsk_[A-Za-z0-9]{16,}|gh[pousr]_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|xox[abprs]-[A-Za-z0-9-]{10,})",
        ),
        ("EMAIL", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
        ("CNIC", r"\b\d{5}-\d{7}-\d\b"),
        ("CARD", r"\b(?:\d[ -]?){12,18}\d\b"),
        ("OTP", r"(?i)\b(?:otp|code|pin|passcode)\b\D{0,12}\d{4,8}\b"),
        ("PHONE", r"(?:\+|\b)\d[\d\s-]{8,14}\d\b"),
    ]
    .into_iter()
    .map(|(label, pattern)| Pattern {
        label,
        regex: Regex::new(pattern).expect("redaction patterns are valid"),
    })
    .collect()
});

#[derive(Debug, Default)]
pub struct Redaction {
    pub text: String,
    replacements: Vec<(String, String)>,
}

impl Redaction {
    pub fn restore(&self, text: &str) -> String {
        self.replacements
            .iter()
            .fold(text.to_owned(), |text, (placeholder, original)| {
                text.replace(placeholder, original)
            })
    }
}

pub fn redact(input: &str) -> Redaction {
    let mut text = input.to_owned();
    let mut replacements: Vec<(String, String)> = Vec::new();
    for pattern in PATTERNS.iter() {
        let mut counter = 0;
        text = pattern
            .regex
            .replace_all(&text, |captures: &regex::Captures| {
                let original = captures[0].to_owned();
                if let Some((placeholder, _)) =
                    replacements.iter().find(|(_, value)| *value == original)
                {
                    return placeholder.clone();
                }
                counter += 1;
                let placeholder = format!("[[{}_{counter}]]", pattern.label);
                replacements.push((placeholder.clone(), original));
                placeholder
            })
            .into_owned();
    }
    Redaction { text, replacements }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_replaced_and_restored() {
        let input = "Email ravi@example.com or call +92 300 1234567. CNIC 42101-1234567-1. \
                     Card 4111 1111 1111 1111. Your OTP is 482913. Key gsk_abcdefghijklmnop1234.";
        let redaction = redact(input);
        for secret in [
            "ravi@example.com",
            "+92 300 1234567",
            "42101-1234567-1",
            "4111 1111 1111 1111",
            "482913",
            "gsk_abcdefghijklmnop1234",
        ] {
            assert!(
                !redaction.text.contains(secret),
                "{secret} leaked: {}",
                redaction.text
            );
        }
        assert!(redaction.text.contains("[[EMAIL_1]]"));
        assert_eq!(redaction.restore(&redaction.text), input);
    }

    #[test]
    fn ordinary_text_is_untouched() {
        let input = "Ship the parser by Friday and review 3 MRs";
        assert_eq!(redact(input).text, input);
    }

    #[test]
    fn repeated_values_share_one_placeholder() {
        let redaction = redact("a@b.co then a@b.co");
        assert_eq!(redaction.text, "[[EMAIL_1]] then [[EMAIL_1]]");
    }
}
