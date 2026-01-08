//! Integration tests for regex pattern redaction feature.
//!
//! These tests verify end-to-end functionality of the regex pattern
//! redaction feature, including various pattern types, edge cases,
//! and error handling.

use anyhow::Result;
use redactor::{RedactionService, RedactionTarget};
use std::sync::Mutex;
use tempfile::TempDir;

mod common;
use common::*;

// Global mutex to serialize MuPDF operations across tests
// MuPDF has thread-safety issues with font loading, so we need to ensure
// only one test uses MuPDF at a time
static MUPDF_LOCK: Mutex<()> = Mutex::new(());

/// Helper macro to wrap MuPDF operations with the global lock
macro_rules! with_mupdf_lock {
    ($body:expr) => {{
        let _guard = MUPDF_LOCK.lock().expect("MuPDF lock poisoned");
        $body
    }};
}

#[test]
fn test_regex_ssn_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with SSN patterns
    TestPdfBuilder::new()
        .with_title("Employee Records")
        .with_content("Employee Information:")
        .with_content("John Doe - SSN: 123-45-6789")
        .with_content("Jane Smith - SSN: 987-65-4321")
        .with_content("Bob Johnson - SSN: 555-12-3456")
        .build(&input_pdf)?;

    // Verify SSNs are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("123-45-6789"));
    assert!(input_text.contains("987-65-4321"));
    assert!(input_text.contains("555-12-3456"));

    // Redact SSNs using regex pattern (without word boundaries due to PDF text extraction)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 3);
    assert!(result.secure);

    // Verify SSNs are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("123-45-6789"));
    assert!(!output_text.contains("987-65-4321"));
    assert!(!output_text.contains("555-12-3456"));
    assert!(output_text.contains("Employee Information"));

    Ok(())
}

#[test]
fn test_regex_email_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with email addresses
    TestPdfBuilder::new()
        .with_title("Contact Directory")
        .with_content("Primary: john.doe@example.com")
        .with_content("Secondary: jane.smith@company.org")
        .with_content("Support: help@support.net")
        .with_content("Not an email: test@")
        .build(&input_pdf)?;

    // Verify emails are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("john.doe@example.com"));
    assert!(input_text.contains("jane.smith@company.org"));
    assert!(input_text.contains("help@support.net"));

    // Redact emails using regex pattern (without word boundaries due to PDF text extraction)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(
            r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}".to_string()
        )]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 3);

    // Verify emails are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("john.doe@example.com"));
    assert!(!output_text.contains("jane.smith@company.org"));
    assert!(!output_text.contains("help@support.net"));
    // Labels may or may not be preserved depending on PDF text extraction

    Ok(())
}

#[test]
fn test_regex_currency_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with currency amounts
    TestPdfBuilder::new()
        .with_title("Financial Report")
        .with_content("Revenue: $1,234.56")
        .with_content("Expenses: $987.65")
        .with_content("Balance: $5,432.10")
        .with_content("Total: $7,654.31")
        .build(&input_pdf)?;

    // Verify amounts are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("$1,234.56"));
    assert!(input_text.contains("$987.65"));

    // Redact currency amounts using regex
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"\$[\d,]+\.\d{2}".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 4);

    // Verify amounts are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("$1,234.56"));
    assert!(!output_text.contains("$987.65"));
    assert!(!output_text.contains("$5,432.10"));
    assert!(output_text.contains("Revenue:"));
    assert!(output_text.contains("Total:"));

    Ok(())
}

#[test]
fn test_regex_date_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with dates
    TestPdfBuilder::new()
        .with_title("Event Schedule")
        .with_content("Meeting: 2026-01-15")
        .with_content("Deadline: 2026-02-28")
        .with_content("Launch: 2026-12-31")
        .with_content("Invalid: 2026-13-40")
        .build(&input_pdf)?;

    // Verify dates are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("2026-01-15"));
    assert!(input_text.contains("2026-02-28"));

    // Redact dates using regex (YYYY-MM-DD format)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"\d{4}-\d{2}-\d{2}".to_string())]
    ))?;

    // Verify redaction occurred (note: catches invalid date too)
    assert!(result.has_redactions());
    assert!(result.instances_redacted >= 3); // At least the 3 valid dates

    // Verify dates are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("2026-01-15"));
    assert!(!output_text.contains("2026-02-28"));
    assert!(output_text.contains("Meeting:"));
    assert!(output_text.contains("Deadline:"));

    Ok(())
}

#[test]
fn test_regex_custom_id_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with custom ID format (AA123456)
    TestPdfBuilder::new()
        .with_title("ID Registry")
        .with_content("Employee ID: AB123456")
        .with_content("Badge: XY789012")
        .with_content("Access Code: QW345678")
        .with_content("Invalid: 123456AB")
        .build(&input_pdf)?;

    // Verify IDs are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("AB123456"));
    assert!(input_text.contains("XY789012"));

    // Redact custom ID format (2 uppercase letters + 6 digits, without word boundaries)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"[A-Z]{2}\d{6}".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 3);

    // Verify IDs are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("AB123456"));
    assert!(!output_text.contains("XY789012"));
    assert!(!output_text.contains("QW345678"));
    assert!(output_text.contains("Invalid: 123456AB")); // Not matching pattern

    Ok(())
}

#[test]
fn test_regex_multiple_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with mixed sensitive data
    TestPdfBuilder::new()
        .with_title("Mixed Data")
        .with_content("SSN: 123-45-6789")
        .with_content("Email: user@example.com")
        .with_content("Phone: (555) 234-5678")
        .with_content("Amount: $1,000.00")
        .build(&input_pdf)?;

    // Verify all data is in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("123-45-6789"));
    assert!(input_text.contains("user@example.com"));
    assert!(input_text.contains("(555) 234-5678"));
    assert!(input_text.contains("$1,000.00"));

    // Redact using multiple regex patterns (without word boundaries for PDF text)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[
            RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string()), // SSN
            RedactionTarget::Regex(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}".to_string()), // Email
            RedactionTarget::Regex(r"\$[\d,]+\.\d{2}".to_string()), // Currency
        ]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert!(result.instances_redacted >= 3);

    // Verify sensitive data is removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("123-45-6789"));
    assert!(!output_text.contains("user@example.com"));
    assert!(!output_text.contains("$1,000.00"));
    // Phone might still be there since we didn't include phone pattern

    Ok(())
}

#[test]
fn test_regex_case_insensitive_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with mixed case keywords
    TestPdfBuilder::new()
        .with_title("Confidential Data")
        .with_content("Status: CONFIDENTIAL document")
        .with_content("Note: This is confidential")
        .with_content("Level: Confidential info")
        .build(&input_pdf)?;

    // Verify keywords are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.to_lowercase().contains("confidential"));

    // Redact "CONFIDENTIAL" in any case using case-insensitive regex (without word boundaries)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"(?i)CONFIDENTIAL".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert!(result.instances_redacted >= 3); // At least 3 instances

    // Verify keywords are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.to_lowercase().contains("confidential"));

    Ok(())
}

#[test]
fn test_regex_ip_address_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with IP addresses
    TestPdfBuilder::new()
        .with_title("Network Configuration")
        .with_content("Server: 192.168.1.100")
        .with_content("Gateway: 10.0.0.1")
        .with_content("DNS: 8.8.8.8")
        .with_content("Invalid: 999.999.999.999")
        .build(&input_pdf)?;

    // Verify IPs are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("192.168.1.100"));
    assert!(input_text.contains("10.0.0.1"));

    // Redact IP addresses (simple pattern, without word boundaries)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}".to_string()
        )]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 4); // Includes invalid IP

    // Verify IPs are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("192.168.1.100"));
    assert!(!output_text.contains("10.0.0.1"));
    assert!(!output_text.contains("8.8.8.8"));

    Ok(())
}

#[test]
fn test_regex_credit_card_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with credit card numbers
    TestPdfBuilder::new()
        .with_title("Payment Information")
        .with_content("Card 1: 4532-1234-5678-9010")
        .with_content("Card 2: 5425-2334-3010-9876")
        .with_content("Card 3: 3782 822463 10005")
        .build(&input_pdf)?;

    // Verify card numbers are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("4532-1234-5678-9010"));

    // Redact credit card numbers (groups of 4 digits, without word boundaries)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(
            r"\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{3,4}".to_string()
        )]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert!(result.instances_redacted >= 1); // At least one card matched

    // Verify card numbers are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("4532-1234-5678-9010"));
    assert!(!output_text.contains("5425-2334-3010-9876"));

    Ok(())
}

#[test]
fn test_regex_url_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with URLs
    TestPdfBuilder::new()
        .with_title("Web Resources")
        .with_content("Website: https://example.com/page")
        .with_content("API: http://api.service.com/v1/endpoint")
        .with_content("Docs: https://docs.internal.org")
        .build(&input_pdf)?;

    // Verify URLs are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("https://example.com/page"));
    assert!(input_text.contains("http://api.service.com"));

    // Redact URLs
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"https?://[^\s]+".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 3);

    // Verify URLs are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("https://example.com"));
    assert!(!output_text.contains("http://api.service.com"));
    // Note: "Website:" and "API:" may be redacted if they're part of the URL in extracted text

    Ok(())
}

#[test]
fn test_regex_invalid_pattern_error() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create a simple PDF
    TestPdfBuilder::new()
        .with_title("Test Document")
        .with_content("Some content here")
        .build(&input_pdf)?;

    // Try to redact with invalid regex pattern
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"[invalid(regex".to_string())] // Invalid regex
    ));

    // Verify error is returned
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid regex pattern") || err.to_string().contains("regex"));

    Ok(())
}

#[test]
fn test_regex_no_matches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF without matching patterns
    TestPdfBuilder::new()
        .with_title("No Sensitive Data")
        .with_content("This document contains no SSNs")
        .with_content("or email addresses")
        .build(&input_pdf)?;

    // Try to redact SSNs (none exist)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())]
    ))?;

    // Verify no redaction occurred
    assert!(!result.has_redactions());
    assert_eq!(result.instances_redacted, 0);

    // Verify output is created and content preserved
    let output_text = extract_text(&output_pdf)?;
    assert!(output_text.contains("No Sensitive Data"));
    assert!(output_text.contains("no SSNs"));

    Ok(())
}

#[test]
fn test_regex_empty_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create a simple PDF
    TestPdfBuilder::new()
        .with_title("Test Document")
        .with_content("Some content")
        .build(&input_pdf)?;

    // Try to redact with empty regex pattern
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex("".to_string())]
    )?);

    // Empty regex is technically valid in Rust regex crate (matches empty strings everywhere)
    // Should complete without error
    // instances_redacted count depends on how many empty strings are found

    Ok(())
}

#[test]
fn test_regex_combined_with_other_targets() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with phone and SSN
    TestPdfBuilder::new()
        .with_title("Personal Information")
        .with_phone("(555) 234-5678")
        .with_content("SSN: 123-45-6789")
        .with_content("Email: test@example.com")
        .build(&input_pdf)?;

    // Verify all data is in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("(555) 234-5678"));
    assert!(input_text.contains("123-45-6789"));
    assert!(input_text.contains("test@example.com"));

    // Redact using phone target AND regex for SSN (without word boundaries)
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[
            RedactionTarget::PhoneNumbers,
            RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string()),
        ]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert!(result.instances_redacted >= 1); // At least something was redacted

    // Verify SSN is removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("123-45-6789"));

    Ok(())
}

#[test]
fn test_regex_word_boundary_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with words that might partially match
    TestPdfBuilder::new()
        .with_title("Word Boundaries")
        .with_content("The secret code is: SECRET")
        .with_content("This is not a SECRET_CODE")
        .with_content("But this is SECRET again")
        .build(&input_pdf)?;

    // Verify content is in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("SECRET"));

    // Redact "SECRET" - note: word boundaries may not work as expected in PDF text
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"SECRET".to_string())]
    ))?;

    // Verify redaction occurred - will match all occurrences including SECRET_CODE
    assert!(result.has_redactions());
    // Note: May match all instances of SECRET, including within SECRET_CODE

    Ok(())
}

#[test]
fn test_regex_multiline_content() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with content across multiple lines
    TestPdfBuilder::new()
        .with_title("Multiline Document")
        .with_content("Line 1: ID-12345")
        .with_content("Line 2: ID-67890")
        .with_content("Line 3: ID-11111")
        .with_content("Line 4: No ID here")
        .with_content("Line 5: ID-99999")
        .build(&input_pdf)?;

    // Verify IDs are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("ID-12345"));
    assert!(input_text.contains("ID-67890"));

    // Redact all ID patterns
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"ID-\d{5}".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 4);

    // Verify IDs are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("ID-12345"));
    assert!(!output_text.contains("ID-67890"));
    assert!(output_text.contains("No ID here"));

    Ok(())
}

#[test]
fn test_regex_special_characters_in_pattern() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_pdf = temp_dir.path().join("input.pdf");
    let output_pdf = temp_dir.path().join("output.pdf");

    // Create PDF with special characters
    TestPdfBuilder::new()
        .with_title("Special Chars")
        .with_content("Reference: [REF-001]")
        .with_content("Code: [REF-002]")
        .with_content("ID: [REF-003]")
        .build(&input_pdf)?;

    // Verify references are in input
    let input_text = extract_text(&input_pdf)?;
    assert!(input_text.contains("[REF-001]"));

    // Redact reference patterns with brackets
    let service = RedactionService::with_secure_strategy();
    let result = with_mupdf_lock!(service.redact(
        &input_pdf,
        &output_pdf,
        &[RedactionTarget::Regex(r"\[REF-\d{3}\]".to_string())]
    ))?;

    // Verify redaction occurred
    assert!(result.has_redactions());
    assert_eq!(result.instances_redacted, 3);

    // Verify references are removed from output
    let output_text = extract_text(&output_pdf)?;
    assert!(!output_text.contains("[REF-001]"));
    assert!(!output_text.contains("[REF-002]"));
    assert!(output_text.contains("Reference:"));

    Ok(())
}
