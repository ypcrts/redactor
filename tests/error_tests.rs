//! Comprehensive error handling tests.
//!
//! These tests verify all error variants, conversions, and error propagation
//! to ensure robust error handling throughout the application.

use redactor::error::{RedactorError, RedactorResult};
use std::error::Error as StdError;
use std::io;
use std::path::PathBuf;

/// Tests error display formatting for all variants to ensure
/// user-facing error messages are clear and actionable.
#[test]
fn test_io_error_display() {
    let path = PathBuf::from("/test/path.pdf");
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = RedactorError::Io {
        path: path.clone(),
        source: io_err,
    };

    let display = err.to_string();
    assert!(display.contains("/test/path.pdf"));
    assert!(display.contains("IO error"));
    assert!(display.contains("file not found"));
}

#[test]
fn test_pdf_processing_error_display_with_page() {
    let err = RedactorError::PdfProcessing {
        message: "Invalid annotation".to_string(),
        page: Some(5),
        source: None,
    };

    let display = err.to_string();
    assert!(display.contains("page 5"));
    assert!(display.contains("Invalid annotation"));
}

#[test]
fn test_pdf_processing_error_display_without_page() {
    let err = RedactorError::PdfProcessing {
        message: "Document corrupted".to_string(),
        page: None,
        source: None,
    };

    let display = err.to_string();
    assert!(!display.contains("page"));
    assert!(display.contains("Document corrupted"));
}

#[test]
fn test_pattern_error_display() {
    let err = RedactorError::PatternError {
        pattern: "[invalid(".to_string(),
        reason: "unclosed bracket".to_string(),
    };

    let display = err.to_string();
    assert!(display.contains("[invalid("));
    assert!(display.contains("unclosed bracket"));
    assert!(display.contains("Pattern error"));
}

#[test]
fn test_text_extraction_error_display() {
    let path = PathBuf::from("/docs/encrypted.pdf");
    let err = RedactorError::TextExtraction {
        path: path.clone(),
        reason: "Password protected".to_string(),
    };

    let display = err.to_string();
    assert!(display.contains("encrypted.pdf"));
    assert!(display.contains("Password protected"));
    assert!(display.contains("Text extraction failed"));
}

#[test]
fn test_pattern_not_found_error_display() {
    let err = RedactorError::PatternNotFound {
        pattern: "SSN".to_string(),
        context: "document text".to_string(),
    };

    let display = err.to_string();
    assert_eq!(display, "Pattern 'SSN' not found: document text");
}

#[test]
fn test_invalid_input_error_display() {
    let err = RedactorError::InvalidInput {
        parameter: "output".to_string(),
        reason: "Path contains invalid UTF-8".to_string(),
    };

    let display = err.to_string();
    assert!(display.contains("output"));
    assert!(display.contains("invalid UTF-8"));
    assert!(display.contains("Invalid input"));
}

#[test]
fn test_backend_error_display() {
    let err = RedactorError::BackendError {
        backend: "MuPDF".to_string(),
        message: "Failed to initialize".to_string(),
        source: None,
    };

    let display = err.to_string();
    assert!(display.contains("MuPDF"));
    assert!(display.contains("Failed to initialize"));
}

/// Tests error source chaining to ensure proper error context propagation.
#[test]
fn test_io_error_source_chain() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let err = RedactorError::Io {
        path: PathBuf::from("/test"),
        source: io_err,
    };

    let source = StdError::source(&err);
    assert!(source.is_some());
    assert_eq!(source.unwrap().to_string(), "access denied");
}

#[test]
fn test_pdf_processing_error_source_chain() {
    let inner_err = io::Error::new(io::ErrorKind::InvalidData, "corrupted stream");
    let err = RedactorError::PdfProcessing {
        message: "Parse error".to_string(),
        page: Some(3),
        source: Some(Box::new(inner_err)),
    };

    let source = StdError::source(&err);
    assert!(source.is_some());
    assert!(source.unwrap().to_string().contains("corrupted stream"));
}

#[test]
fn test_backend_error_source_chain() {
    let inner_err = io::Error::new(io::ErrorKind::Other, "system error");
    let err = RedactorError::BackendError {
        backend: "pdf-extract".to_string(),
        message: "Extraction failed".to_string(),
        source: Some(Box::new(inner_err)),
    };

    let source = StdError::source(&err);
    assert!(source.is_some());
}

#[test]
fn test_errors_without_source_return_none() {
    let err = RedactorError::PatternNotFound {
        pattern: "test".to_string(),
        context: "context".to_string(),
    };

    assert!(StdError::source(&err).is_none());
}

/// Tests automatic error conversions from common error types.
#[test]
fn test_from_io_error_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "missing file");
    let redactor_err: RedactorError = io_err.into();

    match redactor_err {
        RedactorError::BackendError {
            backend, message, ..
        } => {
            assert_eq!(backend, "std::io");
            assert!(message.contains("missing file"));
        }
        _ => panic!("Expected BackendError variant"),
    }
}

#[test]
#[allow(clippy::invalid_regex)]
fn test_from_regex_error_conversion() {
    // Using an intentionally invalid regex pattern to test error conversion
    let invalid_pattern = "[invalid(";
    let regex_result = regex::Regex::new(invalid_pattern);
    assert!(regex_result.is_err());

    let regex_err = regex_result.unwrap_err();
    let redactor_err: RedactorError = regex_err.into();

    match redactor_err {
        RedactorError::PatternError { pattern, reason } => {
            assert_eq!(pattern, "<unknown>");
            assert!(!reason.is_empty());
        }
        _ => panic!("Expected PatternError variant"),
    }
}

#[test]
fn test_from_anyhow_error_conversion() {
    let anyhow_err = anyhow::anyhow!("generic error message");
    let redactor_err: RedactorError = anyhow_err.into();

    match redactor_err {
        RedactorError::BackendError {
            backend,
            message,
            source,
        } => {
            assert_eq!(backend, "anyhow");
            assert!(message.contains("generic error message"));
            assert!(source.is_none());
        }
        _ => panic!("Expected BackendError variant"),
    }
}

/// Tests error result type aliases and composition.
#[test]
fn test_redactor_result_ok() {
    let result: RedactorResult<i32> = Ok(42);
    assert_eq!(result.ok(), Some(42));
}

#[test]
fn test_redactor_result_err() {
    let result: RedactorResult<i32> = Err(RedactorError::InvalidInput {
        parameter: "test".to_string(),
        reason: "invalid".to_string(),
    });

    assert!(result.is_err());
}

/// Tests error matching and pattern handling in realistic scenarios.
#[test]
fn test_error_matching_in_handler() {
    fn handle_error(err: RedactorError) -> String {
        match err {
            RedactorError::Io { path, .. } => format!("File error: {}", path.display()),
            RedactorError::PatternNotFound { pattern, .. } => {
                format!("Pattern '{}' not found", pattern)
            }
            _ => "Unknown error".to_string(),
        }
    }

    let io_err = RedactorError::Io {
        path: PathBuf::from("/test.pdf"),
        source: io::Error::new(io::ErrorKind::NotFound, "not found"),
    };
    assert_eq!(handle_error(io_err), "File error: /test.pdf");

    let pattern_err = RedactorError::PatternNotFound {
        pattern: "account".to_string(),
        context: "bill".to_string(),
    };
    assert_eq!(handle_error(pattern_err), "Pattern 'account' not found");
}

/// Tests error debug representation for logging and debugging.
#[test]
fn test_error_debug_format() {
    let err = RedactorError::InvalidInput {
        parameter: "pattern".to_string(),
        reason: "empty string".to_string(),
    };

    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("InvalidInput"));
    assert!(debug_str.contains("pattern"));
    assert!(debug_str.contains("empty string"));
}

/// Tests that errors are Send + Sync for use in concurrent contexts.
#[test]
fn test_error_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<RedactorError>();
    assert_sync::<RedactorError>();
}

/// Tests error construction with edge cases.
#[test]
fn test_error_with_empty_strings() {
    let err = RedactorError::PatternError {
        pattern: String::new(),
        reason: String::new(),
    };

    let display = err.to_string();
    assert!(display.contains("Pattern error"));
}

#[test]
fn test_error_with_special_characters() {
    let err = RedactorError::PatternNotFound {
        pattern: "test\n\t\r".to_string(),
        context: "file with \"quotes\" and 'apostrophes'".to_string(),
    };

    let display = err.to_string();
    assert!(display.contains("test"));
    assert!(display.contains("quotes"));
}

#[test]
fn test_error_with_unicode() {
    let err = RedactorError::TextExtraction {
        path: PathBuf::from("/files/文档.pdf"),
        reason: "Encoding error: 日本語".to_string(),
    };

    let display = err.to_string();
    assert!(display.contains("文档.pdf"));
    assert!(display.contains("日本語"));
}

/// Tests error propagation through Result chains.
#[test]
fn test_error_propagation_with_question_mark() {
    fn inner() -> RedactorResult<i32> {
        Err(RedactorError::InvalidInput {
            parameter: "inner".to_string(),
            reason: "test".to_string(),
        })
    }

    fn outer() -> RedactorResult<i32> {
        let _value = inner()?;
        Ok(42)
    }

    let result = outer();
    assert!(result.is_err());

    match result.unwrap_err() {
        RedactorError::InvalidInput { parameter, .. } => {
            assert_eq!(parameter, "inner");
        }
        _ => panic!("Expected InvalidInput"),
    }
}

/// Tests error combinations that might occur in real usage.
#[test]
fn test_multiple_error_variants_in_vec() {
    let errors: Vec<RedactorError> = vec![
        RedactorError::PatternNotFound {
            pattern: "test1".to_string(),
            context: "doc1".to_string(),
        },
        RedactorError::InvalidInput {
            parameter: "param1".to_string(),
            reason: "reason1".to_string(),
        },
        RedactorError::PatternError {
            pattern: "test2".to_string(),
            reason: "reason2".to_string(),
        },
    ];

    assert_eq!(errors.len(), 3);

    for err in errors {
        assert!(!err.to_string().is_empty());
    }
}

/// Tests multiple io::ErrorKind variants.
#[test]
fn test_io_error_variants() {
    let path = PathBuf::from("/test/path.pdf");
    let test_cases = vec![
        (io::ErrorKind::NotFound, "not found"),
        (io::ErrorKind::PermissionDenied, "permission denied"),
        (io::ErrorKind::AlreadyExists, "already exists"),
    ];

    for (kind, msg) in test_cases {
        let io_err = io::Error::new(kind, msg);
        let error = RedactorError::Io {
            path: path.clone(),
            source: io_err,
        };
        assert!(error.to_string().contains(msg));
    }
}

/// Realistic scenario: file not found error.
#[test]
fn test_file_not_found_scenario() {
    let error = RedactorError::Io {
        path: PathBuf::from("/nonexistent/file.pdf"),
        source: io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
    };
    assert!(error.to_string().contains("nonexistent"));
    assert!(error.to_string().contains("No such file"));
}

/// Realistic scenario: invalid regex pattern.
#[test]
fn test_invalid_regex_pattern_scenario() {
    let error = RedactorError::InvalidInput {
        parameter: "regex_pattern".to_string(),
        reason: "Invalid regex pattern: regex parse error".to_string(),
    };
    assert!(error.to_string().contains("Invalid regex pattern"));
}

/// Realistic scenario: account number not found.
#[test]
fn test_account_not_found_scenario() {
    let error = RedactorError::PatternNotFound {
        pattern: "Verizon account number".to_string(),
        context: "document text".to_string(),
    };
    assert!(error.to_string().contains("not found"));
}

/// Realistic scenario: corrupted PDF file.
#[test]
fn test_pdf_corruption_scenario() {
    let error = RedactorError::PdfProcessing {
        message: "Invalid PDF header".to_string(),
        page: None,
        source: Some(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected %PDF magic bytes",
        ))),
    };
    assert!(error.to_string().contains("Invalid PDF header"));
    assert!(error.source().is_some());
}

/// Realistic scenario: permission denied when writing.
#[test]
fn test_permission_denied_scenario() {
    let error = RedactorError::Io {
        path: PathBuf::from("/protected/output.pdf"),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied"),
    };
    assert!(error.to_string().contains("Permission denied"));
    assert!(error.to_string().contains("/protected/output.pdf"));
}

/// Property test: all errors have non-empty display.
#[test]
fn test_all_errors_have_nonempty_display() {
    let errors = vec![
        RedactorError::Io {
            path: PathBuf::from("/test"),
            source: io::Error::new(io::ErrorKind::Other, "test"),
        },
        RedactorError::PdfProcessing {
            message: "test".to_string(),
            page: Some(1),
            source: None,
        },
        RedactorError::PatternError {
            pattern: "test".to_string(),
            reason: "test".to_string(),
        },
        RedactorError::TextExtraction {
            path: PathBuf::from("/test"),
            reason: "test".to_string(),
        },
        RedactorError::PatternNotFound {
            pattern: "test".to_string(),
            context: "test".to_string(),
        },
        RedactorError::InvalidInput {
            parameter: "test".to_string(),
            reason: "test".to_string(),
        },
        RedactorError::BackendError {
            backend: "test".to_string(),
            message: "test".to_string(),
            source: None,
        },
    ];

    for error in errors {
        let display = error.to_string();
        assert!(!display.is_empty());
        assert!(display.len() > 10);
    }
}

/// Property test: errors implement std::error::Error trait.
#[test]
fn test_errors_implement_std_error() {
    fn assert_is_std_error<E: std::error::Error>(_: &E) {}
    let error = RedactorError::InvalidInput {
        parameter: "test".to_string(),
        reason: "test".to_string(),
    };
    assert_is_std_error(&error);
}
