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

// Re-export as a module for test backwards compatibility
pub mod redactor {
    pub use super::domain::*;
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
