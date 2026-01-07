//! PDF manipulation and inspection helpers.

use anyhow::Result;
use regex::Regex;
use std::path::Path;

/// Extracts text from a PDF safely, returning an error instead of panicking.
pub fn extract_text(pdf_path: &Path) -> Result<String> {
    redactor::extract_text_from_pdf(pdf_path)
        .map_err(|e| anyhow::anyhow!("Failed to extract text: {}", e))
}

/// Counts occurrences of a pattern in a PDF.
pub fn count_pattern_in_pdf(pdf_path: &Path, pattern: &str) -> Result<usize> {
    let text = extract_text(pdf_path)?;
    Ok(text.matches(pattern).count())
}

/// Counts phone numbers in a PDF using the phone pattern matcher.
pub fn count_phones_in_pdf(pdf_path: &Path) -> Result<usize> {
    let text = extract_text(pdf_path)?;
    let pattern = redactor::get_phone_number_pattern();
    Ok(pattern.find_iter(&text).count())
}

/// Checks if a PDF contains any of the given patterns.
pub fn pdf_contains_any(pdf_path: &Path, patterns: &[&str]) -> Result<bool> {
    let text = extract_text(pdf_path)?;
    Ok(patterns.iter().any(|p| text.contains(p)))
}

/// Checks if a PDF contains all of the given patterns.
pub fn pdf_contains_all(pdf_path: &Path, patterns: &[&str]) -> Result<bool> {
    let text = extract_text(pdf_path)?;
    Ok(patterns.iter().all(|p| text.contains(p)))
}

/// Gets the file size of a PDF in bytes.
pub fn pdf_size(pdf_path: &Path) -> Result<u64> {
    Ok(std::fs::metadata(pdf_path)?.len())
}

/// Validates that a PDF is loadable and has basic structure.
pub fn is_valid_pdf(pdf_path: &Path) -> bool {
    ::lopdf::Document::load(pdf_path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_contains_any() {
        // Would need actual PDF for real test
        // This is a placeholder structure
    }
}
