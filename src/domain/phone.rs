//! Phone number domain logic.
//!
//! This module encapsulates all business rules related to phone number
//! detection, validation, and format generation.

use super::PatternMatcher;
use once_cell::sync::Lazy;
use regex::Regex;

/// American phone number pattern matcher.
///
/// Supports various North American Numbering Plan (NANP) formats:
/// - (555) 123-4567
/// - 555-123-4567
/// - 555.123.4567
/// - +1 555 123 4567
#[derive(Debug, Clone)]
pub struct PhoneNumberMatcher;

impl PhoneNumberMatcher {
    /// Creates a new phone number matcher.
    pub fn new() -> Self {
        Self
    }

    /// Returns the regex pattern for NANP phone numbers.
    fn regex() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?:\+?\s*1[-.\s]?)?\(?\s*([2-9]\d{2})\s*\)?[-.\s]?\s*(\d{3})[-.\s]?\s*(\d{4})\b",
            )
            .expect("Valid phone number regex")
        });
        &PATTERN
    }

    /// Validates that a phone number follows NANP rules.
    ///
    /// # Rules
    /// - Area code (NXX): First digit 2-9, remaining digits 0-9
    /// - Exchange code: First digit 2-9, remaining digits 0-9
    /// - Subscriber number: Any 4 digits
    pub fn validate(area: &str, exchange: &str, subscriber: &str) -> bool {
        area.len() == 3
            && exchange.len() == 3
            && subscriber.len() == 4
            && area
                .chars()
                .next()
                .is_some_and(|c| ('2'..='9').contains(&c))
            && exchange
                .chars()
                .next()
                .is_some_and(|c| ('2'..='9').contains(&c))
    }
}

impl Default for PhoneNumberMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternMatcher for PhoneNumberMatcher {
    fn pattern(&self) -> &Regex {
        Self::regex()
    }

    fn extract_all<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.pattern().find_iter(text).map(|m| m.as_str()).collect()
    }

    fn normalize(&self, text: &str) -> Option<String> {
        // Find first match in text (not just from start)
        self.pattern().captures(text).and_then(|caps| {
            let area_str = caps.get(1)?.as_str();
            let exchange_str = caps.get(2)?.as_str();
            let subscriber_str = caps.get(3)?.as_str();

            if Self::validate(area_str, exchange_str, subscriber_str) {
                Some(format!("{}{}{}", area_str, exchange_str, subscriber_str))
            } else {
                None
            }
        })
    }

    fn generate_variants(&self, normalized: &str) -> Vec<String> {
        if normalized.len() != 10 {
            return vec![normalized.to_string()];
        }

        let area = &normalized[0..3];
        let exchange = &normalized[3..6];
        let subscriber = &normalized[6..10];

        vec![
            normalized.to_string(),                             // 5551234567
            format!("{}-{}-{}", area, exchange, subscriber),    // 555-123-4567
            format!("({}) {}-{}", area, exchange, subscriber),  // (555) 123-4567
            format!("{}.{}.{}", area, exchange, subscriber),    // 555.123.4567
            format!("+1 {} {} {}", area, exchange, subscriber), // +1 555 123 4567
            format!("+1-{}-{}-{}", area, exchange, subscriber), // +1-555-123-4567
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_extraction() {
        let matcher = PhoneNumberMatcher::new();
        let text = "Call us at (555) 123-4567 or 555-987-6543";
        let numbers = matcher.extract_all(text);
        assert_eq!(numbers.len(), 2);
    }

    #[test]
    fn test_phone_normalization() {
        let matcher = PhoneNumberMatcher::new();
        // Use valid NANP number (exchange must start with 2-9)
        assert_eq!(
            matcher.normalize("(555) 234-5678"),
            Some("5552345678".to_string())
        );
    }

    #[test]
    fn test_phone_variants() {
        let matcher = PhoneNumberMatcher::new();
        let variants = matcher.generate_variants("5552345678");
        assert!(variants.contains(&"555-234-5678".to_string()));
        assert!(variants.contains(&"(555) 234-5678".to_string()));
    }

    #[test]
    fn test_invalid_area_code() {
        // Area code cannot start with 0 or 1
        assert!(!PhoneNumberMatcher::validate("155", "234", "5678"));
        assert!(!PhoneNumberMatcher::validate("055", "234", "5678"));
    }
}
