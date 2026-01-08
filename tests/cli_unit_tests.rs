//! Unit tests for CLI components.
//!
//! These tests verify CLI logic without requiring a compiled binary,
//! focusing on the core CLI functions and argument handling logic.

use anyhow::Result;
use redactor::{RedactionService, RedactionTarget};
use std::path::PathBuf;
use tempfile::TempDir;

mod common;
use common::*;

/// Tests for build_targets function behavior (mirrors main.rs logic)
mod build_targets_tests {
    use super::*;
    fn build_targets_mock(
        patterns: &[String],
        phones: bool,
        verizon: bool,
    ) -> Vec<RedactionTarget> {
        let mut targets = Vec::new();

        if verizon {
            targets.push(RedactionTarget::VerizonAccount);
            targets.push(RedactionTarget::PhoneNumbers);
            targets.push(RedactionTarget::VerizonCallDetails);
        }

        if phones && !verizon {
            targets.push(RedactionTarget::PhoneNumbers);
        }

        targets.extend(patterns.iter().map(|p| RedactionTarget::Literal(p.clone())));

        targets
    }

    #[test]
    fn test_verizon_flag_includes_all_three_targets() {
        let targets = build_targets_mock(&[], false, true);

        assert_eq!(targets.len(), 3);
        assert!(matches!(targets[0], RedactionTarget::VerizonAccount));
        assert!(matches!(targets[1], RedactionTarget::PhoneNumbers));
        assert!(matches!(targets[2], RedactionTarget::VerizonCallDetails));
    }

    #[test]
    fn test_phones_flag_adds_single_target() {
        let targets = build_targets_mock(&[], true, false);

        assert_eq!(targets.len(), 1);
        assert!(matches!(targets[0], RedactionTarget::PhoneNumbers));
    }

    #[test]
    fn test_phones_not_duplicated_when_verizon_set() {
        let targets = build_targets_mock(&[], true, true);

        // Should be 3 targets (verizon adds phones, so phones flag is ignored)
        assert_eq!(targets.len(), 3);

        // Count PhoneNumbers targets - should be exactly 1
        let phone_count = targets
            .iter()
            .filter(|t| matches!(t, RedactionTarget::PhoneNumbers))
            .count();
        assert_eq!(phone_count, 1, "PhoneNumbers should not be duplicated");
    }

    #[test]
    fn test_literal_patterns_added() {
        let patterns = vec!["test1".to_string(), "test2".to_string()];
        let targets = build_targets_mock(&patterns, false, false);

        assert_eq!(targets.len(), 2);
        assert!(matches!(targets[0], RedactionTarget::Literal(_)));
        assert!(matches!(targets[1], RedactionTarget::Literal(_)));
    }

    #[test]
    fn test_verizon_plus_literals() {
        let patterns = vec!["custom".to_string()];
        let targets = build_targets_mock(&patterns, false, true);

        assert_eq!(targets.len(), 4);
        // First 3 are verizon-related
        assert!(matches!(targets[0], RedactionTarget::VerizonAccount));
        // Last is literal
        assert!(matches!(targets[3], RedactionTarget::Literal(_)));
    }

    #[test]
    fn test_empty_patterns_array() {
        let targets = build_targets_mock(&[], false, false);
        assert_eq!(targets.len(), 0);
    }

    #[test]
    fn test_multiple_literal_patterns_preserve_order() {
        let patterns = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let targets = build_targets_mock(&patterns, false, false);

        assert_eq!(targets.len(), 3);
        if let RedactionTarget::Literal(s) = &targets[0] {
            assert_eq!(s, "first");
        } else {
            panic!("Expected Literal variant");
        }
        if let RedactionTarget::Literal(s) = &targets[2] {
            assert_eq!(s, "third");
        } else {
            panic!("Expected Literal variant");
        }
    }
}

mod validation_tests {
    use super::*;
    #[test]
    fn test_input_file_validation_missing() {
        let result = validate_input_path(PathBuf::from("/nonexistent.pdf"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_input_file_validation_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pdf = temp_dir.path().join("test.pdf");

        TestPdfBuilder::new().with_title("Test").build(&pdf)?;

        let result = validate_input_path(pdf);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_empty_targets_validation() {
        let targets: Vec<RedactionTarget> = vec![];
        let result = validate_targets(&targets);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No redaction targets"));
    }

    #[test]
    fn test_non_empty_targets_validation() {
        let targets = vec![RedactionTarget::PhoneNumbers];
        let result = validate_targets(&targets);

        assert!(result.is_ok());
    }

    fn validate_input_path(path: PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Input file does not exist: {}", path.display()));
        }
        Ok(())
    }

    fn validate_targets(targets: &[RedactionTarget]) -> Result<(), String> {
        if targets.is_empty() {
            return Err(
                "No redaction targets specified. Use --pattern, --phones, or --verizon."
                    .to_string(),
            );
        }
        Ok(())
    }
}

mod output_formatting_tests {
    use redactor::RedactionResult;

    #[test]
    fn test_success_message_with_redactions() {
        let result = RedactionResult {
            instances_redacted: 5,
            pages_processed: 3,
            pages_modified: 2,
            secure: true,
        };

        let message = format_success_message(&result, "/output.pdf");
        assert!(message.contains("5"));
        assert!(message.contains("output.pdf"));
    }

    #[test]
    fn test_success_message_no_redactions() {
        let message = format_no_redactions_message();
        assert!(message.contains("No instances"));
    }

    #[test]
    fn test_verbose_output_format() {
        let result = RedactionResult {
            instances_redacted: 10,
            pages_processed: 5,
            pages_modified: 3,
            secure: true,
        };

        let message = format_verbose_message(&result);
        assert!(message.contains("10"));
        assert!(message.contains("5"));
        assert!(message.contains("3"));
    }

    fn format_success_message(result: &RedactionResult, output_path: &str) -> String {
        format!(
            "✓ Successfully redacted {} instance(s) → {}",
            result.instances_redacted, output_path
        )
    }

    fn format_no_redactions_message() -> String {
        "⚠ No instances found to redact".to_string()
    }

    fn format_verbose_message(result: &RedactionResult) -> String {
        format!(
            "Pages processed: {}\nPages modified: {}\nInstances redacted: {}",
            result.pages_processed, result.pages_modified, result.instances_redacted
        )
    }
}

mod error_handling_tests {
    #[test]
    fn test_error_message_formatting() {
        let errors = vec![
            (
                "Input file not found",
                "Input file does not exist: /test.pdf",
            ),
            ("No targets", "No redaction targets specified"),
            ("Invalid pattern", "Invalid regex pattern: unclosed bracket"),
        ];

        for (category, message) in errors {
            let formatted = format_error(category, message);
            assert!(formatted.contains(category) || formatted.contains(message));
        }
    }

    #[test]
    fn test_error_with_context() {
        let error = "Pattern 'account' not found";
        let context = "in document text";

        let formatted = format_error_with_context(error, context);
        assert!(formatted.contains(error));
        assert!(formatted.contains(context));
    }

    fn format_error(category: &str, message: &str) -> String {
        format!("Error: {}: {}", category, message)
    }

    fn format_error_with_context(error: &str, context: &str) -> String {
        format!("{}: {}", error, context)
    }
}

mod extract_command_tests {
    use super::*;
    #[test]
    fn test_extract_command_logic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_pdf = temp_dir.path().join("input.pdf");

        TestPdfBuilder::new()
            .with_title("Extract Test")
            .with_content("This is test content for extraction")
            .build(&input_pdf)?;

        let service = RedactionService::with_secure_strategy();
        let text = service.extract_text(&input_pdf)?;

        assert!(text.contains("Extract Test"));
        assert!(text.contains("test content"));
        Ok(())
    }

    #[test]
    fn test_extract_empty_pdf() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_pdf = temp_dir.path().join("empty.pdf");

        TestPdfBuilder::new().with_title("").build(&input_pdf)?;

        let service = RedactionService::with_secure_strategy();
        let text = service.extract_text(&input_pdf)?;

        // Should return some text (at least empty or minimal)
        assert_eq!(text.trim(), "");
        Ok(())
    }
}

mod integration_workflow_tests {
    use super::*;
    use std::sync::Mutex;

    static MUPDF_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_complete_redaction_workflow() -> Result<()> {
        let _guard = MUPDF_LOCK.lock().unwrap();
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        // Create input PDF
        TestPdfBuilder::new()
            .with_title("Test Document")
            .with_phone("(555) 234-5678")
            .build(&input)?;

        // Validate input exists
        assert!(input.exists());

        // Build targets
        let targets = vec![RedactionTarget::PhoneNumbers];
        assert!(!targets.is_empty());

        // Execute redaction
        let service = RedactionService::with_secure_strategy();
        let result = service.redact(&input, &output, &targets)?;

        // Verify results
        assert!(result.has_redactions());
        assert!(output.exists());

        Ok(())
    }

    #[test]
    fn test_workflow_with_multiple_targets() -> Result<()> {
        let _guard = MUPDF_LOCK.lock().unwrap();
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Multi-Target Test")
            .with_phone("(555) 234-5678")
            .with_content("Secret: CONFIDENTIAL")
            .build(&input)?;

        let targets = vec![
            RedactionTarget::PhoneNumbers,
            RedactionTarget::Literal("CONFIDENTIAL".to_string()),
        ];

        let service = RedactionService::with_secure_strategy();
        let result = service.redact(&input, &output, &targets)?;

        assert!(result.has_redactions());
        // At least one pattern should match (phone and/or CONFIDENTIAL)
        // Due to PDF text encoding, exact match count may vary
        assert!(result.instances_redacted >= 1);

        Ok(())
    }
}
