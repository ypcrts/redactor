//! Redaction strategy trait and supporting types.
//!
//! This module defines the core abstraction for redaction strategies,
//! allowing for different implementations (secure, visual, etc.).

use crate::error::RedactorResult;
use std::path::Path;

/// A pattern or text to be redacted from a document.
#[derive(Debug, Clone)]
pub enum RedactionTarget {
    /// Exact text match
    Literal(String),

    /// Regular expression pattern
    Regex(String),

    /// Phone numbers (using domain-specific logic)
    PhoneNumbers,

    /// Verizon account numbers (using domain-specific logic)
    VerizonAccount,

    /// Verizon call detail columns (time, origination, destination)
    VerizonCallDetails,

    /// All text (complete document redaction)
    AllText,
}

/// Statistics about a redaction operation.
#[derive(Debug, Clone, Default)]
pub struct RedactionResult {
    /// Number of instances redacted
    pub instances_redacted: usize,

    /// Pages processed
    pub pages_processed: usize,

    /// Pages with redactions
    pub pages_modified: usize,

    /// Whether text was physically removed (vs visually obscured)
    pub secure: bool,
}

impl RedactionResult {
    /// Creates a result indicating no redactions were needed.
    pub fn none() -> Self {
        Self::default()
    }

    /// Returns true if any redactions were applied.
    pub fn has_redactions(&self) -> bool {
        self.instances_redacted > 0
    }
}

/// Strategy for redacting sensitive information from PDFs.
///
/// Implementations of this trait define how redaction is performed,
/// allowing for different approaches (secure deletion, visual overlay, etc.).
pub trait RedactionStrategy: Send + Sync {
    /// Redacts specified patterns from a PDF document.
    ///
    /// # Arguments
    /// * `input` - Path to input PDF file
    /// * `output` - Path where redacted PDF should be written
    /// * `targets` - Patterns/text to redact
    ///
    /// # Returns
    /// Statistics about the redaction operation
    fn redact(
        &self,
        input: &Path,
        output: &Path,
        targets: &[RedactionTarget],
    ) -> RedactorResult<RedactionResult>;

    /// Extracts text from a PDF for pattern matching.
    ///
    /// This method should handle complex text encodings (e.g., Type3 fonts).
    fn extract_text(&self, input: &Path) -> RedactorResult<String>;

    /// Returns a human-readable name for this strategy.
    fn name(&self) -> &str;

    /// Returns whether this strategy provides secure (physical) deletion.
    fn is_secure(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_result() {
        let result = RedactionResult::none();
        assert!(!result.has_redactions());

        let result = RedactionResult {
            instances_redacted: 5,
            ..Default::default()
        };
        assert!(result.has_redactions());
    }
}
