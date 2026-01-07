//! Account number domain logic.
//!
//! This module provides business logic for detecting and handling
//! account numbers, with special support for Verizon's 9-5 format.

use super::PatternMatcher;
use once_cell::sync::Lazy;
use regex::Regex;

/// Verizon account number pattern matcher.
///
/// Verizon uses a 14-digit format, typically displayed as 9-5 (XXXXXXXXX-XXXXX).
/// This matcher handles various representations and prioritizes context-aware matches.
#[derive(Debug, Clone)]
pub struct VerizonAccountMatcher;

impl VerizonAccountMatcher {
    /// Creates a new Verizon account matcher.
    pub fn new() -> Self {
        Self
    }

    /// Extracts the most likely account number from text.
    ///
    /// Uses a priority system:
    /// 1. 9-5 format with "account" keyword nearby
    /// 2. Any 9-5 format
    /// 3. 14 consecutive digits with "account" keyword
    /// 4. Any 14 consecutive digits
    /// 5. Generic account numbers (10-15 digits)
    pub fn find_account_number(text: &str) -> Option<String> {
        let mut candidates = Vec::new();

        // Priority 1: 9-5 format with account keywords
        if let Some(caps) = Self::pattern_9_5_with_context().captures(text) {
            if let (Some(prefix), Some(suffix)) = (caps.get(1), caps.get(2)) {
                candidates.push((0, format!("{}{}", prefix.as_str(), suffix.as_str())));
            }
        }

        // Priority 2: Any 9-5 format
        for cap in Self::pattern_9_5().captures_iter(text) {
            if let (Some(prefix), Some(suffix)) = (cap.get(1), cap.get(2)) {
                candidates.push((1, format!("{}{}", prefix.as_str(), suffix.as_str())));
            }
        }

        // Priority 3: 14 consecutive digits with context
        if let Some(caps) = Self::pattern_14_with_context().captures(text) {
            if let Some(matched) = caps.get(1) {
                candidates.push((2, matched.as_str().to_string()));
            }
        }

        // Priority 4: 14 consecutive digits
        for cap in Self::pattern_14().captures_iter(text) {
            if let Some(matched) = cap.get(1) {
                candidates.push((3, matched.as_str().to_string()));
            }
        }

        // Priority 5: Generic account number
        for cap in Self::pattern_generic().captures_iter(text) {
            if let Some(matched) = cap.get(1) {
                let digits: String = matched
                    .as_str()
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if digits.len() >= 10 && digits.len() <= 15 {
                    candidates.push((4, digits));
                }
            }
        }

        // Remove duplicates while preserving priority order
        candidates.sort_by_key(|(priority, _)| *priority);
        candidates.dedup_by(|(_, a), (_, b)| a == b);

        // Prefer 14-digit candidates
        candidates
            .iter()
            .find(|(_, num)| num.len() == 14)
            .map(|(_, num)| num.clone())
            .or_else(|| candidates.first().map(|(_, num)| num.clone()))
    }

    // Pattern helper methods (cached via Lazy)

    fn pattern_9_5_with_context() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)(?:account|acct)(?:\s*(?:number|num|no|#))?\s*:?\s*(\d{9})-(\d{5})")
                .expect("Valid regex")
        });
        &PATTERN
    }

    fn pattern_9_5() -> &'static Regex {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\d{9})-(\d{5})").expect("Valid regex"));
        &PATTERN
    }

    fn pattern_14_with_context() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)(?:account|acct)(?:\s*(?:number|num|no|#))?\s*:?\s*(\d{14})")
                .expect("Valid regex")
        });
        &PATTERN
    }

    fn pattern_14() -> &'static Regex {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\b(\d{14})\b").expect("Valid regex"));
        &PATTERN
    }

    fn pattern_generic() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)(?:account|acct)(?:\s*(?:number|num|no|#))?\s*:?\s*([\d\s\-]{10,20})")
                .expect("Valid regex")
        });
        &PATTERN
    }
}

impl Default for VerizonAccountMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternMatcher for VerizonAccountMatcher {
    fn pattern(&self) -> &Regex {
        Self::pattern_14()
    }

    fn extract_all<'a>(&self, text: &'a str) -> Vec<&'a str> {
        Self::pattern_9_5()
            .find_iter(text)
            .map(|m| m.as_str())
            .collect()
    }

    fn normalize(&self, text: &str) -> Option<String> {
        Self::find_account_number(text)
    }

    fn generate_variants(&self, normalized: &str) -> Vec<String> {
        let len = normalized.len();
        let mut variants = vec![normalized.to_string()];

        match len {
            14 => {
                // Verizon 9-5 format
                let prefix = &normalized[0..9];
                let suffix = &normalized[9..14];
                variants.push(format!("{}-{}", prefix, suffix));
                variants.push(format!("{} {}", prefix, suffix));
            }
            12 => {
                // Alternative formats
                variants.push(format!(
                    "{}-{}-{}",
                    &normalized[0..4],
                    &normalized[4..8],
                    &normalized[8..12]
                ));
                variants.push(format!("{}-{}", &normalized[0..6], &normalized[6..12]));
            }
            len if len >= 10 => {
                // Generic split
                let mid = len / 2;
                variants.push(format!("{}-{}", &normalized[0..mid], &normalized[mid..]));
                variants.push(format!("{} {}", &normalized[0..mid], &normalized[mid..]));
            }
            _ => {}
        }

        variants
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verizon_account_extraction() {
        let text = "Account Number: 123456789-00001";
        let account = VerizonAccountMatcher::find_account_number(text);
        assert_eq!(account, Some("12345678900001".to_string()));
    }

    #[test]
    fn test_account_priority() {
        // Should prefer account with context over standalone
        let text = "Random: 999999999-99999 Account: 123456789-00001";
        let account = VerizonAccountMatcher::find_account_number(text);
        assert_eq!(account, Some("12345678900001".to_string()));
    }

    #[test]
    fn test_account_variants() {
        let matcher = VerizonAccountMatcher::new();
        let variants = matcher.generate_variants("12345678900001");
        assert!(variants.contains(&"12345678900001".to_string()));
        assert!(variants.contains(&"123456789-00001".to_string()));
        assert!(variants.contains(&"123456789 00001".to_string()));
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn test_no_account_found() {
        let text = "This document has no account number";
        let account = VerizonAccountMatcher::find_account_number(text);
        assert_eq!(account, None);
    }
}
