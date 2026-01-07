//! Verizon call detail column detection and redaction.
//!
//! This module handles detection of call detail tables in Verizon bills
//! and generates patterns to redact time, origination, and destination columns.

use super::PatternMatcher;
use once_cell::sync::Lazy;
use regex::Regex;

/// Matcher for Verizon call detail columns (time, origination, destination).
///
/// Verizon bills typically have a call detail section with columns:
/// - Date
/// - Time (e.g., "10:26 PM", "2:30 PM")
/// - Number (phone number - handled by PhoneNumberMatcher)
/// - Origination (location, e.g., "New York, NY")
/// - Destination (location or type, e.g., "Incoming, CL" or "Nwyrcyzn15, NY")
#[derive(Debug, Clone, Default)]
pub struct VerizonCallDetailsMatcher;

impl VerizonCallDetailsMatcher {
    /// Creates a new Verizon call details matcher.
    pub fn new() -> Self {
        Self
    }

    /// Time pattern: matches times like "10:26 PM", "2:30 AM", etc.
    pub fn time_pattern() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b\d{1,2}:\d{2}\s*(?:AM|PM|am|pm)\b").expect("Valid time regex pattern")
        });
        &PATTERN
    }

    /// Origination pattern: matches location patterns like "New York, NY" or "Los Angeles, CA"
    pub fn origination_pattern() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b[A-Z][a-zA-Z\s]+,\s*[A-Z]{2}\b")
                .expect("Valid origination regex pattern")
        });
        &PATTERN
    }

    /// Destination pattern: matches both location patterns and call types
    /// like "Incoming, CL", "Nwyrcyzn15, NY", or standalone locations
    pub fn destination_pattern() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b(?:[A-Z][a-zA-Z\s]+,\s*[A-Z]{2}|[A-Z][a-z]+,\s*[A-Z]{2}|Incoming,\s*[A-Z]{2})\b")
                .expect("Valid destination regex pattern")
        });
        &PATTERN
    }

    /// Combined pattern for all call detail columns we want to redact
    pub fn combined_pattern() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            // Match: time OR location patterns
            Regex::new(
                r"(?x)
                \b\d{1,2}:\d{2}\s*(?:AM|PM|am|pm)\b  # Time pattern
                |
                \b[A-Z][a-zA-Z\s]+,\s*[A-Z]{2}\b     # Location pattern (City, ST)
                |
                \bIncoming,\s*[A-Z]{2}\b              # Incoming call type
                |
                \b[A-Z][a-z]{3,}[a-z0-9]*,\s*[A-Z]{2}\b  # Other destination patterns
            ",
            )
            .expect("Valid combined regex pattern")
        });
        &PATTERN
    }

    /// Extract all time values from text
    pub fn extract_times<'a>(&self, text: &'a str) -> Vec<&'a str> {
        Self::time_pattern()
            .find_iter(text)
            .map(|m| m.as_str())
            .collect()
    }

    /// Extract all origination values from text
    pub fn extract_originations<'a>(&self, text: &'a str) -> Vec<&'a str> {
        Self::origination_pattern()
            .find_iter(text)
            .map(|m| m.as_str())
            .collect()
    }

    /// Extract all destination values from text
    pub fn extract_destinations<'a>(&self, text: &'a str) -> Vec<&'a str> {
        Self::destination_pattern()
            .find_iter(text)
            .map(|m| m.as_str())
            .collect()
    }

    /// Check if text contains a call detail table header
    pub fn has_call_detail_table(text: &str) -> bool {
        // Look for the table header with Time, Origination, Destination
        let header_pattern =
            Regex::new(r"(?i)Date\s+Time\s+Number\s+Origination\s+Destination").unwrap();
        header_pattern.is_match(text)
    }

    /// Extract all call detail column values (time, origination, destination)
    /// from text that contains a call detail table
    pub fn extract_all_call_details(&self, text: &str) -> Vec<String> {
        let mut details = Vec::new();

        // Extract times
        for time in self.extract_times(text) {
            details.push(time.to_string());
        }

        // Extract originations (deduplicate)
        let mut originations = self.extract_originations(text);
        originations.sort();
        originations.dedup();
        for orig in originations {
            details.push(orig.to_string());
        }

        // Extract destinations (deduplicate)
        let mut destinations = self.extract_destinations(text);
        destinations.sort();
        destinations.dedup();
        for dest in destinations {
            details.push(dest.to_string());
        }

        details
    }
}

impl PatternMatcher for VerizonCallDetailsMatcher {
    fn pattern(&self) -> &Regex {
        Self::combined_pattern()
    }

    fn extract_all<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.pattern().find_iter(text).map(|m| m.as_str()).collect()
    }

    fn normalize(&self, text: &str) -> Option<String> {
        // For call details, we don't need normalization - return as-is if it matches
        if self.pattern().is_match(text) {
            Some(text.to_string())
        } else {
            None
        }
    }

    fn generate_variants(&self, text: &str) -> Vec<String> {
        // For call detail values, we generate minimal variants
        // The values should match exactly as they appear
        vec![text.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_pattern() {
        let text = "Jul 11  3:45 PM  555-234-1111  Miami,  FL  Incoming,  CL";
        let matcher = VerizonCallDetailsMatcher::new();
        let times = matcher.extract_times(text);
        assert_eq!(times.len(), 1);
        assert_eq!(times[0], "3:45 PM");
    }

    #[test]
    fn test_time_pattern_variations() {
        let matcher = VerizonCallDetailsMatcher::new();

        assert_eq!(matcher.extract_times("2:30 PM")[0], "2:30 PM");
        assert_eq!(matcher.extract_times("11:46 PM")[0], "11:46 PM");
        assert_eq!(matcher.extract_times("9:15 AM")[0], "9:15 AM");
    }

    #[test]
    fn test_origination_pattern() {
        let matcher = VerizonCallDetailsMatcher::new();
        let text = "Miami, FL  Incoming, CL";
        let origs = matcher.extract_originations(text);
        // May extract multiple matches depending on pattern overlaps
        assert!(!origs.is_empty(), "Should extract at least one origination");
        assert!(
            origs.iter().any(|o| o.contains("Miami")),
            "Should contain 'Miami'"
        );
    }

    #[test]
    fn test_destination_pattern() {
        let matcher = VerizonCallDetailsMatcher::new();

        let text1 = "Miami, FL  Incoming, CL";
        let dests1 = matcher.extract_destinations(text1);
        assert!(dests1.iter().any(|d| d.contains("Incoming")));

        let text2 = "Miami, FL  Orlando, FL";
        let dests2 = matcher.extract_destinations(text2);
        assert!(!dests2.is_empty());
    }

    #[test]
    fn test_has_call_detail_table() {
        let text_with_table =
            "Date  Time  Number  Origination  Destination  Min.  Airtime  Charges";
        assert!(VerizonCallDetailsMatcher::has_call_detail_table(
            text_with_table
        ));

        let text_without_table = "This is just regular text";
        assert!(!VerizonCallDetailsMatcher::has_call_detail_table(
            text_without_table
        ));
    }

    #[test]
    fn test_extract_all_call_details() {
        let text = r#"
Date  Time  Number  Origination  Destination  Min.  Airtime  Charges  LD/Other  Charges  Total 
Jul 11  3:45 PM  555-234-1111  Miami,  FL  Incoming,  CL  2  --  --  -- 
Jul 12  9:15 AM  555-345-2222  Miami,  FL  Incoming,  CL  1  --  --  -- 
Jul 12  11:30 PM  555-456-3333  Miami,  FL  Orlando,  FL  1  --  --  -- 
"#;
        let matcher = VerizonCallDetailsMatcher::new();
        let details = matcher.extract_all_call_details(text);

        // Should extract times, originations, and destinations
        assert!(!details.is_empty());

        // Check that times are extracted
        assert!(details.iter().any(|d| d.contains("3:45 PM")));
        assert!(details.iter().any(|d| d.contains("9:15 AM")));

        // Check that locations are extracted
        assert!(details.iter().any(|d| d.contains("Miami")));
    }

    #[test]
    fn test_pattern_matcher_interface() {
        let matcher = VerizonCallDetailsMatcher::new();
        let text = "Call at 3:45 PM from Miami, FL";

        let matches = matcher.extract_all(text);
        assert!(matches.len() >= 2); // Should match time and location

        // Test normalization
        let normalized = matcher.normalize("3:45 PM");
        assert_eq!(normalized, Some("3:45 PM".to_string()));
    }
}
