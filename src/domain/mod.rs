//! Domain models and business logic for pattern matching.
//!
//! This module contains the core business logic for detecting and matching
//! sensitive patterns in PDF text, including phone numbers and account numbers.

pub mod account;
pub mod call_details;
pub mod phone;

pub use account::VerizonAccountMatcher;
pub use call_details::VerizonCallDetailsMatcher;
pub use phone::PhoneNumberMatcher;

use once_cell::sync::Lazy;
use regex::Regex;

/// Trait for pattern matching strategies.
pub trait PatternMatcher: Send + Sync {
    fn pattern(&self) -> &Regex;
    fn extract_all<'a>(&self, text: &'a str) -> Vec<&'a str>;
    fn normalize(&self, text: &str) -> Option<String>;
    fn generate_variants(&self, normalized: &str) -> Vec<String>;
}

/// PDF escape sequences and patterns.
pub struct PdfEscapes;

impl PdfEscapes {
    pub fn unescape(text: &str) -> String {
        text.replace("\\040", " ")
            .replace("\\050", "(")
            .replace("\\051", ")")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\\", "\\")
    }
}

pub struct PdfPatterns;

impl PdfPatterns {
    pub fn text_string() -> &'static Regex {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\(([^)]*)\)").expect("Valid regex pattern"));
        &PATTERN
    }

    pub fn tj_array() -> &'static Regex {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\[([^\]]*)\]\s*TJ").expect("Valid regex pattern"));
        &PATTERN
    }
}
