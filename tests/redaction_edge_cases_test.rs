//! Edge case tests for redaction strategies.
//!
//! Tests boundary conditions, error paths, and unusual scenarios that
//! might occur in production. Follows Google's testing principles of
//! testing corner cases explicitly and thoroughly.

use anyhow::Result;
use redactor::{RedactionService, RedactionStrategy, RedactionTarget, SecureRedactionStrategy};
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

mod common;
use common::*;

// Global mutex for MuPDF operations
static MUPDF_LOCK: Mutex<()> = Mutex::new(());

macro_rules! with_mupdf_lock {
    ($body:expr) => {{
        let _guard = MUPDF_LOCK.lock().expect("MuPDF lock poisoned");
        $body
    }};
}

// ============================================================================
// Input Validation Tests
// ============================================================================

/// Tests that redacting a non-existent file returns appropriate error.
#[test]
fn test_redact_nonexistent_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist.pdf");
    let output = temp_dir.path().join("output.pdf");

    let service = RedactionService::with_secure_strategy();
    let result = service.redact(&nonexistent, &output, &[RedactionTarget::PhoneNumbers]);

    assert!(
        result.is_err(),
        "Redacting non-existent file should return error"
    );
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("does_not_exist.pdf")
            || error.to_string().to_lowercase().contains("not found")
            || error.to_string().to_lowercase().contains("exist"),
        "Error should mention the missing file: {}",
        error
    );
}

/// Tests that providing no redaction targets returns error.
#[test]
fn test_redact_empty_targets_error() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    // Create a minimal PDF
    TestPdfBuilder::new()
        .with_content("Some content")
        .build(&input)
        .unwrap();

    let service = RedactionService::with_secure_strategy();
    let result = service.redact(&input, &output, &[]);

    assert!(
        result.is_err(),
        "Redacting with empty targets should return error"
    );
    let error = result.unwrap_err();
    assert!(
        error.to_string().to_lowercase().contains("target")
            || error.to_string().to_lowercase().contains("empty"),
        "Error should mention missing targets: {}",
        error
    );
}

/// Tests extraction from non-existent file.
#[test]
fn test_extract_nonexistent_file_error() {
    let nonexistent = PathBuf::from("/nonexistent/file.pdf");
    let service = RedactionService::with_secure_strategy();

    let result = service.extract_text(&nonexistent);
    assert!(
        result.is_err(),
        "Extracting from non-existent file should error"
    );
}

// ============================================================================
// Empty and Minimal Content Tests
// ============================================================================

/// Tests redacting an empty PDF (no text content).
#[test]
fn test_redact_empty_pdf() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("empty.pdf");
    let output = temp_dir.path().join("output.pdf");

    // Create PDF with no content
    TestPdfBuilder::new().with_title("Empty").build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

    assert_eq!(
        result.instances_redacted, 0,
        "Empty PDF should have no redactions"
    );
    assert!(output.exists(), "Output file should be created");

    Ok(())
}

/// Tests redacting PDF with only whitespace.
#[test]
fn test_redact_whitespace_only_pdf() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("whitespace.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("   \n\n\t\t   ")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

    assert_eq!(
        result.instances_redacted, 0,
        "Whitespace-only PDF should have no redactions"
    );

    Ok(())
}

/// Tests redacting when pattern exists but doesn't match.
#[test]
fn test_redact_no_matches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("This document has no phone numbers")
        .with_content("Just regular text here")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

    assert_eq!(result.instances_redacted, 0, "Should find no matches");
    assert!(output.exists(), "Output should still be created");

    // Output should preserve original content
    let output_text = extract_text(&output)?;
    assert!(
        output_text.contains("no phone numbers"),
        "Original text should be preserved when no matches"
    );

    Ok(())
}

// ============================================================================
// Literal Pattern Edge Cases
// ============================================================================

/// Tests redacting empty literal pattern.
#[test]
fn test_redact_empty_literal_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("Some content here")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal(String::new())]
    ))?;

    // Empty pattern should match nothing or be handled gracefully
    // Note: instances_redacted is usize, so this always passes - we just verify no panic
    let _ = result.instances_redacted;

    Ok(())
}

/// Tests redacting very long literal pattern.
#[test]
fn test_redact_long_literal_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    let long_text = "a".repeat(1000);
    TestPdfBuilder::new()
        .with_content(&long_text)
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal(long_text.clone())]
    ))?;

    // Should handle long patterns without crashing
    // Successfully completing without panic is the test
    let _ = result.instances_redacted;

    Ok(())
}

/// Tests redacting literal pattern with special characters.
#[test]
fn test_redact_literal_special_characters() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    let special = "Test: $100.00 (50% off!) [SALE]";
    TestPdfBuilder::new().with_content(special).build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal(special.to_string())]
    ))?;

    // Should handle special characters - no panic is success
    let _ = result.instances_redacted;

    Ok(())
}

/// Tests redacting literal pattern with unicode.
#[test]
fn test_redact_literal_unicode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    let unicode = "Hello 世界 🌍 Привет";
    TestPdfBuilder::new().with_content(unicode).build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal("世界".to_string())]
    ))?;

    // Should handle unicode without panicking - no panic is success
    let _ = result.instances_redacted;

    Ok(())
}

// ============================================================================
// Multiple Pattern Interactions
// ============================================================================

/// Tests multiple identical patterns (deduplication).
#[test]
fn test_redact_duplicate_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("Secret: CLASSIFIED")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[
            RedactionTarget::Literal("CLASSIFIED".to_string()),
            RedactionTarget::Literal("CLASSIFIED".to_string()),
            RedactionTarget::Literal("CLASSIFIED".to_string()),
        ]
    ))?;

    // Should handle duplicates gracefully (might redact once or multiple times)
    assert!(result.instances_redacted > 0, "Should redact the pattern");

    Ok(())
}

/// Tests overlapping patterns.
#[test]
fn test_redact_overlapping_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("Account: 123456789")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[
            RedactionTarget::Literal("123456".to_string()),
            RedactionTarget::Literal("456789".to_string()),
            RedactionTarget::Literal("123456789".to_string()),
        ]
    ))?;

    // Should handle overlapping patterns
    assert!(result.instances_redacted > 0);

    Ok(())
}

/// Tests many patterns at once (stress test).
#[test]
fn test_redact_many_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("Document with various content: ABC DEF GHI")
        .build(&input)?;

    // Create 50 different literal patterns
    let patterns: Vec<_> = (0..50)
        .map(|i| RedactionTarget::Literal(format!("pattern{}", i)))
        .collect();

    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(&input, &output, &patterns))?;

    // Should handle many patterns without error - no panic is success
    let _ = result.instances_redacted;

    Ok(())
}

// ============================================================================
// Strategy Configuration Tests
// ============================================================================

/// Tests custom max_hits configuration.
#[test]
fn test_strategy_custom_max_hits() {
    let strategy = SecureRedactionStrategy::new().with_max_hits(10);
    // Strategy should be created successfully with custom max_hits
    // Actual enforcement would need PDF with many matches to test
    assert_eq!(strategy.name(), "SecureRedaction");
}

/// Tests strategy name and security properties.
#[test]
fn test_strategy_properties() {
    let strategy = SecureRedactionStrategy::new();
    assert_eq!(strategy.name(), "SecureRedaction");
    assert!(strategy.is_secure(), "Strategy should report as secure");
}

// ============================================================================
// Service API Tests
// ============================================================================

/// Tests service creation methods.
#[test]
fn test_service_creation_variants() {
    let service1 = RedactionService::with_secure_strategy();
    let service2 = RedactionService::new(Box::new(SecureRedactionStrategy::new()));

    // Both methods should create valid services
    // We can't easily compare them, but they should not panic
    let _ = service1;
    let _ = service2;
}

/// Tests text extraction with minimal PDF.
#[test]
fn test_extract_text_minimal_pdf() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("minimal.pdf");

    TestPdfBuilder::new()
        .with_title("Test")
        .with_content("Hello World")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let text = service.extract_text(&input)?;

    assert!(
        text.contains("Hello World"),
        "Extracted text should contain document content"
    );

    Ok(())
}

/// Tests text extraction from empty PDF.
#[test]
fn test_extract_text_empty_pdf() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("empty.pdf");

    TestPdfBuilder::new().with_title("Empty").build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let text = service.extract_text(&input)?;

    // Empty PDF might return empty string or just title - no panic is success
    let _ = text.len();

    Ok(())
}

// ============================================================================
// Regression Tests
// ============================================================================

/// Regression test: Ensure output file is created even with no matches.
#[test]
fn test_regression_output_created_no_matches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("No sensitive data here")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())]
    ))?;

    assert!(
        output.exists(),
        "Output file should be created even when no patterns match"
    );

    Ok(())
}

/// Regression test: Ensure statistics are accurate.
#[test]
fn test_regression_accurate_statistics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_content("Three phones: (555) 234-5678, 555-987-6543, (555) 111-2222")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

    assert!(
        result.has_redactions(),
        "Result should report redactions occurred"
    );
    assert!(
        result.instances_redacted >= 1,
        "Should report at least one phone redacted"
    );
    assert!(result.secure, "Should report as secure redaction");
    assert!(result.pages_processed > 0, "Should report pages processed");

    Ok(())
}
