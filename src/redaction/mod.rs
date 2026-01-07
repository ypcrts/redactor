//! Redaction strategies and implementations.
//!
//! This module provides a strategy pattern for different redaction approaches,
//! allowing for flexible and testable redaction implementations.

pub mod secure;
pub mod strategy;

pub use secure::SecureRedactionStrategy;
pub use strategy::{RedactionResult, RedactionStrategy, RedactionTarget};

use crate::error::{RedactorError, RedactorResult};
use std::path::Path;

/// Redaction service coordinating strategy execution.
///
/// This service provides a high-level API for redacting documents
/// using different strategies while handling common concerns like
/// progress reporting and error handling.
pub struct RedactionService {
    strategy: Box<dyn RedactionStrategy>,
}

impl RedactionService {
    /// Creates a new redaction service with the specified strategy.
    pub fn new(strategy: Box<dyn RedactionStrategy>) -> Self {
        Self { strategy }
    }

    /// Creates a service with secure (physical removal) redaction.
    pub fn with_secure_strategy() -> Self {
        Self::new(Box::new(SecureRedactionStrategy::default()))
    }

    /// Redacts patterns from a PDF document.
    ///
    /// # Arguments
    /// * `input` - Path to input PDF
    /// * `output` - Path for output PDF
    /// * `targets` - Patterns to redact
    ///
    /// # Returns
    /// Result containing redaction statistics
    pub fn redact(
        &self,
        input: &Path,
        output: &Path,
        targets: &[RedactionTarget],
    ) -> RedactorResult<RedactionResult> {
        // Validate inputs
        if !input.exists() {
            return Err(RedactorError::Io {
                path: input.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Input file does not exist",
                ),
            });
        }

        if targets.is_empty() {
            return Err(RedactorError::InvalidInput {
                parameter: "targets".to_string(),
                reason: "No redaction targets specified".to_string(),
            });
        }

        // Execute redaction strategy
        self.strategy.redact(input, output, targets)
    }

    /// Extracts text from a PDF for analysis.
    pub fn extract_text(&self, input: &Path) -> RedactorResult<String> {
        self.strategy.extract_text(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let _service = RedactionService::with_secure_strategy();
    }
}
