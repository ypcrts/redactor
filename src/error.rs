//! Error types for the PDF redaction library.
//!
//! This module provides a comprehensive error handling strategy with proper
//! error categorization and context preservation.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type alias for redaction operations.
pub type RedactorResult<T> = Result<T, RedactorError>;

/// Comprehensive error type for all redaction operations.
///
/// This enum categorizes errors by their source and provides rich context
/// for debugging and error recovery.
#[derive(Debug)]
pub enum RedactorError {
    /// Error occurred while reading or writing files
    Io { path: PathBuf, source: io::Error },

    /// Error occurred during PDF processing
    PdfProcessing {
        message: String,
        page: Option<usize>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Pattern matching or regex compilation error
    PatternError { pattern: String, reason: String },

    /// Text extraction failed
    TextExtraction { path: PathBuf, reason: String },

    /// Pattern not found in document
    PatternNotFound { pattern: String, context: String },

    /// Invalid configuration or parameters
    InvalidInput { parameter: String, reason: String },

    /// Backend-specific error (MuPDF, LoPDF, etc.)
    BackendError {
        backend: String,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl fmt::Display for RedactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "IO error for path '{}': {}", path.display(), source)
            }
            Self::PdfProcessing { message, page, .. } => {
                if let Some(p) = page {
                    write!(f, "PDF processing error on page {}: {}", p, message)
                } else {
                    write!(f, "PDF processing error: {}", message)
                }
            }
            Self::PatternError { pattern, reason } => {
                write!(f, "Pattern error for '{}': {}", pattern, reason)
            }
            Self::TextExtraction { path, reason } => {
                write!(
                    f,
                    "Text extraction failed for '{}': {}",
                    path.display(),
                    reason
                )
            }
            Self::PatternNotFound { pattern, context } => {
                write!(f, "Pattern '{}' not found: {}", pattern, context)
            }
            Self::InvalidInput { parameter, reason } => {
                write!(f, "Invalid input for '{}': {}", parameter, reason)
            }
            Self::BackendError {
                backend, message, ..
            } => {
                write!(f, "{} backend error: {}", backend, message)
            }
        }
    }
}

impl std::error::Error for RedactorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::PdfProcessing { source, .. } | Self::BackendError { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
            _ => None,
        }
    }
}

// Conversion implementations for common error types
impl From<io::Error> for RedactorError {
    fn from(err: io::Error) -> Self {
        Self::BackendError {
            backend: "std::io".to_string(),
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<regex::Error> for RedactorError {
    fn from(err: regex::Error) -> Self {
        Self::PatternError {
            pattern: "<unknown>".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for RedactorError {
    fn from(err: anyhow::Error) -> Self {
        Self::BackendError {
            backend: "anyhow".to_string(),
            message: err.to_string(),
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RedactorError::PatternNotFound {
            pattern: "test".to_string(),
            context: "document".to_string(),
        };
        assert_eq!(err.to_string(), "Pattern 'test' not found: document");
    }
}
