//! Production-grade PDF redaction library with secure text removal.
//!
//! This library provides secure PDF redaction with support for complex
//! text encodings including Type3 fonts. It uses MuPDF's redaction API
//! to physically remove sensitive information from PDF documents.
//!
//! # Features
//!
//! - **Secure Redaction**: Physically removes text from PDFs (not just visual overlay)
//! - **Type3 Font Support**: Handles complex PDF encodings via MuPDF
//! - **Phone Number Detection**: Automatic NANP phone number redaction
//! - **Verizon Account Numbers**: Specialized detection for 9-5 format accounts
//! - **Pattern Matching**: Literal strings and regex patterns
//!
//! # Architecture
//!
//! - [`domain`]: Business logic for pattern matching (phone numbers, accounts)
//! - [`redaction`]: Redaction strategies and service layer
//! - [`error`]: Comprehensive error handling
//!
//! # Quick Start
//!
//! ```no_run
//! use redactor::{RedactionService, RedactionTarget};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = RedactionService::with_secure_strategy();
//!
//! service.redact(
//!     Path::new("input.pdf"),
//!     Path::new("output.pdf"),
//!     &[RedactionTarget::PhoneNumbers]
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! # Examples
//!
//! ## Redact Verizon Account Numbers
//!
//! ```no_run
//! use redactor::{RedactionService, RedactionTarget};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = RedactionService::with_secure_strategy();
//!
//! service.redact(
//!     Path::new("bill.pdf"),
//!     Path::new("redacted.pdf"),
//!     &[RedactionTarget::VerizonAccount]
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Pattern Matching
//!
//! ```
//! use redactor::domain::{PhoneNumberMatcher, PatternMatcher};
//!
//! let matcher = PhoneNumberMatcher::new();
//! let text = "Call (555) 234-5678 or 555-987-6543";
//! let phones = matcher.extract_all(text);
//! assert_eq!(phones.len(), 2);
//! ```

// Public API
pub mod domain;
pub mod error;
pub mod redaction;

// Re-exports for convenient access
pub use domain::{
    PatternMatcher, PhoneNumberMatcher, VerizonAccountMatcher, VerizonCallDetailsMatcher,
};
pub use error::{RedactorError, RedactorResult};
pub use redaction::{
    RedactionResult, RedactionService, RedactionStrategy, RedactionTarget, SecureRedactionStrategy,
};

// Legacy compatibility layer
pub mod legacy {
    //! Legacy API for backwards compatibility with existing code.
    //!
    //! This module provides the old function-based API that existing
    //! code depends on. New code should use the service-based API instead.

    use super::*;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use std::path::Path;

    /// Legacy: Get phone number regex pattern.
    pub fn get_phone_number_pattern() -> Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| PhoneNumberMatcher::new().pattern().clone());
        PATTERN.clone()
    }

    /// Legacy: Extract text from PDF.
    pub fn extract_text_from_pdf(path: &Path) -> RedactorResult<String> {
        let service = RedactionService::with_secure_strategy();
        service.extract_text(path)
    }

    /// Legacy: Find Verizon account number.
    pub fn find_verizon_account_number(text: &str) -> Option<String> {
        VerizonAccountMatcher::find_account_number(text)
    }

    /// Legacy: Generate account patterns.
    pub fn generate_account_patterns(account: &str) -> Vec<String> {
        VerizonAccountMatcher::new().generate_variants(account)
    }

    /// Legacy: Redact Verizon account in PDF.
    pub fn redact_verizon_account_in_pdf(input: &Path, output: &Path) -> RedactorResult<()> {
        let service = RedactionService::with_secure_strategy();
        service.redact(input, output, &[RedactionTarget::VerizonAccount])?;
        Ok(())
    }

    /// Legacy: Redact patterns with MuPDF.
    pub fn redact_patterns_with_mupdf(
        input: &Path,
        output: &Path,
        patterns: &[String],
    ) -> RedactorResult<()> {
        let service = RedactionService::with_secure_strategy();
        let targets: Vec<_> = patterns
            .iter()
            .map(|p| RedactionTarget::Literal(p.clone()))
            .collect();
        service.redact(input, output, &targets)?;
        Ok(())
    }

    /// Legacy: Redact phone numbers (content-stream based).
    ///
    /// Note: This is visual-only redaction. For secure redaction, use the service API.
    pub fn redact_phone_numbers(content: &[u8], phone_pattern: &Regex) -> Vec<u8> {
        use domain::PdfPatterns;

        let content_str = match String::from_utf8(content.to_vec()) {
            Ok(s) => s,
            Err(_) => return content.to_vec(),
        };

        let text_pattern = PdfPatterns::text_string();
        let redacted = text_pattern
            .replace_all(&content_str, |caps: &regex::Captures| {
                let text = &caps[1];
                if phone_pattern.is_match(text) {
                    let cleaned =
                        phone_pattern.replace_all(text, |_: &regex::Captures| "█".repeat(12));
                    format!("({})", cleaned)
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        redacted.into_bytes()
    }

    /// Legacy: Redact phone numbers in PDF file.
    pub fn redact_phone_numbers_in_pdf(
        input_path: &Path,
        output_path: &Path,
    ) -> RedactorResult<()> {
        let service = RedactionService::with_secure_strategy();
        service.redact(input_path, output_path, &[RedactionTarget::PhoneNumbers])?;
        Ok(())
    }

    /// Legacy: Extract text from page content (basic parser).
    ///
    /// This is a simple parser that doesn't handle Type3 fonts well.
    /// For robust extraction, use `extract_text_from_pdf` instead.
    pub fn extract_text_from_page_content(content: &[u8]) -> String {
        use domain::{PdfEscapes, PdfPatterns};

        let content_str = String::from_utf8_lossy(content);
        let text_pattern = PdfPatterns::text_string();
        let mut extracted_text = String::new();

        for cap in text_pattern.captures_iter(&content_str) {
            if let Some(text_match) = cap.get(1) {
                let text = text_match.as_str();
                let unescaped = PdfEscapes::unescape(text);
                extracted_text.push_str(&unescaped);
                extracted_text.push(' ');
            }
        }

        let tj_pattern = PdfPatterns::tj_array();
        for cap in tj_pattern.captures_iter(&content_str) {
            if let Some(tj_content) = cap.get(1) {
                for text_cap in text_pattern.captures_iter(tj_content.as_str()) {
                    if let Some(text_match) = text_cap.get(1) {
                        let text = text_match.as_str();
                        let unescaped = PdfEscapes::unescape(text);
                        extracted_text.push_str(&unescaped);
                        extracted_text.push(' ');
                    }
                }
            }
        }

        extracted_text
    }
}

// Re-export legacy API at module root for backwards compatibility
pub use legacy::*;

// Re-export as a module for test backwards compatibility
pub mod redactor {
    pub use super::domain::*;
    pub use super::legacy::*;
    pub use super::redaction::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let _service = RedactionService::with_secure_strategy();
    }

    #[test]
    fn test_pattern_matchers() {
        let phone_matcher = PhoneNumberMatcher::new();
        // Use valid NANP number (exchange must start with 2-9, not 0 or 1)
        assert!(phone_matcher.normalize("(555) 234-5678").is_some());

        let account_matcher = VerizonAccountMatcher::new();
        let variants = account_matcher.generate_variants("12345678900001");
        assert!(!variants.is_empty());
    }
}
