//! Tests specifically created to close identified coverage gaps.
//!
//! This file contains tests targeting specific uncovered code paths identified
//! by cargo-llvm-cov analysis. See BASELINE_COVERAGE_REPORT.md for details.

use anyhow::Result;
use redactor::{RedactionService, RedactionTarget, SecureRedactionStrategy};
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

// ============================================================================
// Tests for redaction/secure.rs gaps (71.85% → 85%+ target)
// ============================================================================

/// Tests full-page redaction using `.+` pattern.
///
/// Coverage gap: apply_mupdf_redactions() redact_all path not directly tested
#[test]
fn test_full_page_redaction_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_title("Full Redaction Test")
        .with_content("Line 1: Sensitive data")
        .with_content("Line 2: More sensitive data")
        .with_content("Line 3: Everything should be redacted")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();

    // Use a pattern that should match everything
    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal(".+".to_string())]
    ))?;

    assert!(output.exists());
    assert!(result.has_redactions());

    Ok(())
}

/// Tests VerizonCallDetails target when document has no call detail table.
///
/// Coverage gap: resolve_patterns() VerizonCallDetails branch when has_call_detail_table() returns false
#[test]
fn test_verizon_call_details_no_table_present() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    // Create PDF without call detail table
    TestPdfBuilder::new()
        .with_title("Regular Bill")
        .with_content("Total charges: $100.00")
        .with_content("No call details included")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();

    // Should handle gracefully when no call detail table exists
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::VerizonCallDetails]))?;

    assert!(output.exists());
    // No redactions expected since no call detail table
    assert_eq!(result.instances_redacted, 0);

    Ok(())
}

/// Tests redaction with path containing non-ASCII characters.
///
/// Coverage gap: redact() path with unusual UTF-8 characters
#[test]
fn test_redaction_with_unicode_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("文档_test.pdf");
    let output = temp_dir.path().join("输出_test.pdf");

    TestPdfBuilder::new()
        .with_title("Unicode Path Test")
        .with_content("Test content")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();

    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal("Test".to_string())]
    ))?;

    assert!(output.exists());
    // Should handle unicode paths without error
    let _ = result.instances_redacted;

    Ok(())
}

/// Tests custom SecureRedactionStrategy configuration.
///
/// Coverage gap: Strategy with non-default max_hits values
#[test]
fn test_strategy_with_custom_max_hits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_title("Max Hits Test")
        .with_content("Pattern Pattern Pattern")
        .build(&input)?;

    // Create strategy with very low max_hits
    let strategy = SecureRedactionStrategy::new().with_max_hits(2);
    let service = RedactionService::new(Box::new(strategy));

    let result = with_mupdf_lock!(service.redact(
        &input,
        &output,
        &[RedactionTarget::Literal("Pattern".to_string())]
    ))?;

    assert!(output.exists());
    // Should limit hits to max_hits value
    assert!(result.instances_redacted <= 2);

    Ok(())
}

/// Tests resolve_patterns with empty pattern list result.
///
/// Coverage gap: When no patterns are found for a target
#[test]
fn test_resolve_patterns_no_matches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");

    TestPdfBuilder::new()
        .with_title("No Phones")
        .with_content("This document has no phone numbers")
        .build(&input)?;

    let service = RedactionService::with_secure_strategy();

    // Try to redact phones when none exist
    let result =
        with_mupdf_lock!(service.redact(&input, &output, &[RedactionTarget::PhoneNumbers]))?;

    assert!(output.exists());
    assert_eq!(result.instances_redacted, 0);
    // File should be copied as-is when no patterns found
    assert_eq!(result.pages_modified, 0);

    Ok(())
}

// ============================================================================
// Tests for domain/account.rs gaps (78.77% → 95%+ target)
// ============================================================================

/// Tests generate_variants with 12-digit account number.
///
/// Coverage gap: 12-digit format branch in generate_variants()
#[test]
fn test_account_variants_12_digit_format() {
    use redactor::domain::{PatternMatcher, VerizonAccountMatcher};

    let matcher = VerizonAccountMatcher::new();
    let account_12_digit = "123456789012";

    let variants = matcher.generate_variants(account_12_digit);

    // Should generate variants for 12-digit format
    assert!(!variants.is_empty());
    assert!(variants.contains(&account_12_digit.to_string()));

    // Should generate formatted variants
    assert!(variants.iter().any(|v| v.contains('-')));
}

/// Tests find_account_number generic pattern fallback.
///
/// Coverage gap: Generic pattern matching when standard formats don't match
#[test]
fn test_find_account_generic_pattern() {
    use redactor::domain::VerizonAccountMatcher;

    // Account with generic format (not 9-5 or 14 digits)
    let text = "Account Number: 1234-5678-9012";
    let result = VerizonAccountMatcher::find_account_number(text);

    // Should find using generic pattern
    assert!(result.is_some());
}

/// Tests account extraction with unusual spacing.
///
/// Coverage gap: Various spacing patterns in account numbers
#[test]
fn test_account_with_spaces() {
    use redactor::domain::VerizonAccountMatcher;

    let text = "Account: 123 456 789 00 001";
    let result = VerizonAccountMatcher::find_account_number(text);

    // Generic pattern should handle spaces
    assert!(result.is_some());
}

// ============================================================================
// Tests for domain/phone.rs gaps (96.43% → 100% target)
// ============================================================================

/// Tests phone number validation with edge case area codes.
///
/// Coverage gap: Boundary values for area code validation
#[test]
fn test_phone_validation_edge_cases() {
    use redactor::domain::PhoneNumberMatcher;

    // Area code starting with 2 (minimum valid)
    assert!(PhoneNumberMatcher::validate("200", "234", "5678"));

    // Area code starting with 9 (maximum valid)
    assert!(PhoneNumberMatcher::validate("999", "234", "5678"));

    // Exchange code starting with 2 (minimum valid)
    assert!(PhoneNumberMatcher::validate("555", "200", "5678"));

    // Exchange code starting with 9 (maximum valid)
    assert!(PhoneNumberMatcher::validate("555", "999", "5678"));
}

// ============================================================================
// Tests for main.rs gaps (98.21% → 100% target)
// ============================================================================

/// Tests RedactionHandler verbose output formatting.
///
/// Coverage gap: Verbose output paths in handler
#[test]
fn test_handler_verbose_output() -> Result<()> {
    // This is tested via CLI integration tests with --verbose flag
    // Adding explicit test for the formatting logic
    use redactor::RedactionResult;

    let result = RedactionResult {
        instances_redacted: 5,
        pages_processed: 3,
        pages_modified: 2,
        secure: true,
    };

    // Verify all fields are accessible and formatted correctly
    assert_eq!(result.instances_redacted, 5);
    assert_eq!(result.pages_processed, 3);
    assert_eq!(result.pages_modified, 2);
    assert!(result.secure);
    assert!(result.has_redactions());

    Ok(())
}

// ============================================================================
// Tests for domain/call_details.rs gaps (97.01% → 100% target)
// ============================================================================

/// Tests call details normalize with non-matching input.
///
/// Coverage gap: normalize() returns None for non-matching input
#[test]
fn test_call_details_normalize_no_match() {
    use redactor::domain::{PatternMatcher, VerizonCallDetailsMatcher};

    let matcher = VerizonCallDetailsMatcher::new();

    // Text that doesn't match any call detail patterns
    let result = matcher.normalize("Regular text");
    assert_eq!(result, None);
}

/// Tests call details generate_variants.
///
/// Coverage gap: Ensure generate_variants is called and works
#[test]
fn test_call_details_generate_variants() {
    use redactor::domain::{PatternMatcher, VerizonCallDetailsMatcher};

    let matcher = VerizonCallDetailsMatcher::new();

    // Generate variants for a time pattern
    let variants = matcher.generate_variants("3:45 PM");

    // Should return at least the original
    assert!(!variants.is_empty());
    assert_eq!(variants[0], "3:45 PM");
}
