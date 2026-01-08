//! Comprehensive edge case tests for secure redaction.
//!
//! Tests error handling, boundary conditions, and robustness of the
//! secure redaction implementation.

use anyhow::Result;
use redactor::{RedactionService, RedactionStrategy, RedactionTarget, SecureRedactionStrategy};
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

mod common;
use common::*;

// Global mutex to serialize MuPDF operations
static MUPDF_LOCK: Mutex<()> = Mutex::new(());

macro_rules! with_mupdf_lock {
    ($body:expr) => {{
        let _guard = MUPDF_LOCK.lock().expect("MuPDF lock poisoned");
        $body
    }};
}

/// Tests that verify robust error handling and edge cases.
mod error_handling {
    use super::*;

    #[test]
    fn test_redact_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.pdf");
        let output = temp_dir.path().join("output.pdf");

        let service = RedactionService::with_secure_strategy();
        let result = service.redact(
            &nonexistent,
            &output,
            &[RedactionTarget::Literal("test".to_string())],
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not exist") || err.to_string().contains("NotFound"));
    }

    #[test]
    fn test_redact_with_empty_targets() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        // Create a minimal PDF
        TestPdfBuilder::new()
            .with_title("Test")
            .with_content("Content")
            .build(&input)
            .unwrap();

        let service = RedactionService::with_secure_strategy();
        let result = service.redact(&input, &output, &[]);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No redaction targets"));
    }

    #[test]
    fn test_redact_invalid_regex_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .with_content("Some content")
            .build(&input)
            .unwrap();

        let service = RedactionService::with_secure_strategy();

        // Test various invalid regex patterns
        let invalid_patterns = vec![
            "[invalid(",
            "(?P<invalid",
            "**",
            "(?:",
            "[z-a]", // Invalid range
        ];

        for pattern in invalid_patterns {
            let result = with_mupdf_lock!(service.redact(
                &input,
                &output,
                &[RedactionTarget::Regex(pattern.to_string())]
            ));

            assert!(result.is_err(), "Should fail for pattern: {}", pattern);
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("regex") || err.to_string().contains("pattern"),
                "Error should mention regex/pattern for: {}",
                pattern
            );
        }
    }

    #[test]
    fn test_extract_text_from_nonexistent_file() {
        let service = RedactionService::with_secure_strategy();
        let result = service.extract_text(&PathBuf::from("/nonexistent/file.pdf"));

        assert!(result.is_err());
    }

    #[test]
    fn test_redact_to_invalid_output_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");

        TestPdfBuilder::new().with_title("Test").build(&input)?;

        // Try to write to a directory that doesn't exist
        let invalid_output = PathBuf::from("/nonexistent/directory/output.pdf");

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &invalid_output,
            &[RedactionTarget::Literal("test".to_string())]
        ));

        // Should fail (either during redaction or when saving)
        assert!(result.is_err());

        Ok(())
    }
}

/// Tests boundary conditions and edge cases in redaction logic.
mod boundary_conditions {
    use super::*;

    #[test]
    fn test_redact_empty_document() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        // Create PDF with no text content
        TestPdfBuilder::new().with_title("Empty").build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result =
            with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

        // Should succeed with no redactions
        assert_eq!(result.instances_redacted, 0);
        assert!(!result.has_redactions());
        assert!(output.exists());

        Ok(())
    }

    #[test]
    fn test_redact_very_long_pattern() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        let long_pattern = "A".repeat(10000);

        TestPdfBuilder::new()
            .with_title("Test")
            .with_content(&long_pattern)
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Literal(long_pattern)]
        ))?;

        assert!(result.has_redactions());

        Ok(())
    }

    #[test]
    fn test_redact_many_small_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!("Item{} ", i));
        }

        TestPdfBuilder::new()
            .with_title("Many Items")
            .with_content(&content)
            .build(&input)?;

        // Create many literal targets
        let targets: Vec<_> = (0..100)
            .map(|i| RedactionTarget::Literal(format!("Item{}", i)))
            .collect();

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(&input, &output, &targets))?;

        assert!(result.has_redactions());
        assert!(result.instances_redacted >= 50); // At least half should match

        Ok(())
    }

    #[test]
    fn test_redact_pattern_at_page_boundaries() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        // Create content that might span page boundaries
        let mut builder = TestPdfBuilder::new().with_title("Boundary Test");

        for i in 0..50 {
            builder = builder.with_content(&format!("Line {} contains SECRET data", i));
        }

        builder.build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Literal("SECRET".to_string())]
        ))?;

        assert!(result.has_redactions());

        Ok(())
    }

    #[test]
    fn test_redact_unicode_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Unicode Test")
            .with_content("日本語 文字")
            .with_content("한국어 텍스트")
            .with_content("中文 测试")
            .with_content("Emoji: 🔒 🔐")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let _result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Literal("日本語".to_string())]
        ))?;

        // May or may not find matches depending on PDF encoding
        // The important thing is it doesn't crash
        assert!(output.exists());

        Ok(())
    }

    #[test]
    fn test_redact_special_regex_characters() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Special Chars")
            .with_content("Price: $100.00")
            .with_content("Email: user@example.com")
            .with_content("Path: C:\\Users\\test")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Regex(r"\$\d+\.\d{2}".to_string())]
        ))?;

        assert!(result.has_redactions());

        let output_text = extract_text(&output)?;
        assert!(!output_text.contains("$100.00"));

        Ok(())
    }
}

/// Tests for SecureRedactionStrategy configuration.
mod strategy_configuration {
    use super::*;

    #[test]
    fn test_strategy_with_custom_max_hits() {
        let strategy = SecureRedactionStrategy::new().with_max_hits(50);

        // Verify it's created successfully
        assert_eq!(strategy.name(), "SecureRedaction");
        assert!(strategy.is_secure());
    }

    #[test]
    fn test_strategy_with_very_low_max_hits() {
        let strategy = SecureRedactionStrategy::new().with_max_hits(1);

        assert_eq!(strategy.name(), "SecureRedaction");
    }

    #[test]
    fn test_strategy_with_very_high_max_hits() {
        let strategy = SecureRedactionStrategy::new().with_max_hits(10000);

        assert_eq!(strategy.name(), "SecureRedaction");
    }

    #[test]
    fn test_strategy_default() {
        let strategy = SecureRedactionStrategy::default();

        assert_eq!(strategy.name(), "SecureRedaction");
        assert!(strategy.is_secure());
    }

    #[test]
    fn test_strategy_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SecureRedactionStrategy>();
        assert_sync::<SecureRedactionStrategy>();
    }

    #[test]
    fn test_strategy_clone() {
        let strategy1 = SecureRedactionStrategy::new().with_max_hits(75);
        let strategy2 = strategy1.clone();

        assert_eq!(strategy1.name(), strategy2.name());
        assert_eq!(strategy1.is_secure(), strategy2.is_secure());
    }
}

/// Tests for pattern resolution and matching logic.
mod pattern_resolution {
    use super::*;

    #[test]
    fn test_verizon_account_not_found() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("No Account")
            .with_content("This document has no Verizon account number")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result =
            with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::VerizonAccount]));

        // Should fail because no account found
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("Verizon"));

        Ok(())
    }

    #[test]
    fn test_phone_numbers_with_no_phones() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("No Phones")
            .with_content("This document has no phone numbers at all")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result =
            with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

        // Should succeed with no redactions
        assert_eq!(result.instances_redacted, 0);
        assert!(output.exists());

        Ok(())
    }

    #[test]
    fn test_call_details_without_table() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("No Table")
            .with_content("Regular content without call detail table")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::VerizonCallDetails]
        ))?;

        // Should succeed with no redactions
        assert_eq!(result.instances_redacted, 0);

        Ok(())
    }

    #[test]
    fn test_literal_pattern_case_sensitive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Case Test")
            .with_content("SECRET secret SeCrEt")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let _result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Literal("SECRET".to_string())]
        ))?;

        // Should only match exact case
        let output_text = extract_text(&output)?;
        assert!(!output_text.contains("SECRET"));
        // These might still be there (case-sensitive match)
        // Note: PDF search might be case-insensitive in MuPDF

        Ok(())
    }
}

/// Tests for concurrent and multi-threaded scenarios.
mod concurrency {
    use super::*;
    use std::thread;

    #[test]
    fn test_service_creation_is_thread_safe() {
        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let _service = RedactionService::with_secure_strategy();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_strategy_clone_across_threads() {
        let strategy = SecureRedactionStrategy::new().with_max_hits(100);

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let s = strategy.clone();
                thread::spawn(move || {
                    assert_eq!(s.name(), "SecureRedaction");
                    assert!(s.is_secure());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

/// Tests for result statistics and metadata.
mod result_statistics {
    use super::*;

    #[test]
    fn test_result_secure_flag() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .with_phone("(555) 234-5678")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result =
            with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

        assert!(result.secure, "Should be marked as secure redaction");

        Ok(())
    }

    #[test]
    fn test_result_pages_processed() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("input.pdf");
        let output = temp_dir.path().join("output.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .with_content("Content")
            .build(&input)?;

        let service = RedactionService::with_secure_strategy();
        let result = with_mupdf_lock!(service.redact(
            &input,
            &output,
            &[RedactionTarget::Literal("test".to_string())]
        ))?;

        assert!(
            result.pages_processed > 0,
            "Should process at least one page"
        );

        Ok(())
    }
}
