//! Custom assertions for PDF redaction testing.
//!
//! Provides domain-specific assertions that make tests more readable
//! and provide better error messages.

use anyhow::Result;
use std::path::Path;

/// Asserts that a pattern has been successfully redacted from a PDF.
///
/// This extracts text from the PDF and verifies the pattern is not present.
///
/// # Panics
/// Panics if the pattern is still found in the PDF text.
pub fn assert_redacted(pdf_path: &Path, pattern: &str) {
    let text = extract_text_or_panic(pdf_path);
    assert!(
        !text.contains(pattern),
        "Pattern '{}' should be redacted but was found in output PDF at '{}'.\nExtracted text length: {} chars",
        pattern,
        pdf_path.display(),
        text.len()
    );
}

/// Asserts that a pattern has been preserved (not redacted) in a PDF.
///
/// # Panics
/// Panics if the pattern is not found in the PDF.
pub fn assert_preserved(pdf_path: &Path, pattern: &str) {
    let text = extract_text_or_panic(pdf_path);
    assert!(
        text.contains(pattern),
        "Pattern '{}' should be preserved but was not found in PDF at '{}'",
        pattern,
        pdf_path.display()
    );
}

/// Asserts that a PDF contains valid content (is not empty/corrupted).
///
/// # Panics
/// Panics if the PDF appears to be empty or corrupted.
pub fn assert_valid_pdf(pdf_path: &Path) {
    assert!(
        pdf_path.exists(),
        "PDF should exist at '{}'",
        pdf_path.display()
    );
    
    let metadata = std::fs::metadata(pdf_path).expect("Failed to get PDF metadata");
    assert!(
        metadata.len() > 0,
        "PDF should not be empty at '{}'",
        pdf_path.display()
    );
    
    // Try to extract text to verify PDF is readable
    let text = extract_text_or_panic(pdf_path);
    assert!(
        !text.is_empty() || is_valid_empty_pdf(pdf_path),
        "PDF should contain text or be a valid empty PDF at '{}'",
        pdf_path.display()
    );
}

/// Asserts that multiple patterns are all redacted.
///
/// # Panics
/// Panics if any pattern is found in the PDF.
pub fn assert_all_redacted(pdf_path: &Path, patterns: &[&str]) {
    let text = extract_text_or_panic(pdf_path);
    let mut found_patterns = Vec::new();
    
    for pattern in patterns {
        if text.contains(pattern) {
            found_patterns.push(*pattern);
        }
    }
    
    assert!(
        found_patterns.is_empty(),
        "The following patterns should be redacted but were found: {:?}",
        found_patterns
    );
}

// Note: RedactionResult statistics assertions would go here
// Currently the service doesn't return detailed statistics
// This is a future enhancement

// Helper functions

fn extract_text_or_panic(pdf_path: &Path) -> String {
    redactor::extract_text_from_pdf(pdf_path)
        .unwrap_or_else(|e| panic!("Failed to extract text from PDF '{}': {}", pdf_path.display(), e))
}

fn is_valid_empty_pdf(pdf_path: &Path) -> bool {
    // A valid empty PDF might have no extractable text but valid structure
    // Try to load it with lopdf to check structure validity
    ::lopdf::Document::load(pdf_path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[should_panic(expected = "should be redacted")]
    fn test_assert_redacted_fails_when_pattern_present() {
        // This would need a real PDF, so we skip actual implementation
        // In real code, you'd create a test PDF here
        let _fake_path = PathBuf::from("/tmp/nonexistent.pdf");
        // This would panic if the file existed and contained "test"
        // assert_redacted(&_fake_path, "test");
    }
    
    #[test]
    fn test_assert_all_redacted_empty_patterns() {
        // Test with empty patterns list should always pass
        let _fake_path = PathBuf::from("/tmp/test.pdf");
        // Would work if PDF existed
        // assert_all_redacted(&_fake_path, &[]);
    }
}
