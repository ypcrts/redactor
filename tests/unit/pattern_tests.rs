//! Unit tests for pattern matching and regex logic.

use anyhow::Result;
use redactor::get_phone_number_pattern;
use regex::Regex;

#[test]
fn test_phone_pattern_basic_formats() {
    let pattern = get_phone_number_pattern();
    
    // Valid formats
    assert!(pattern.is_match("(555) 234-5678"));
    assert!(pattern.is_match("555-234-5678"));
    assert!(pattern.is_match("555.234.5678"));
    assert!(pattern.is_match("+1 555-234-5678"));
}

#[test]
fn test_phone_pattern_invalid_formats() {
    let pattern = get_phone_number_pattern();
    
    // Invalid: too short
    assert!(!pattern.is_match("555-1234"));
    
    // Invalid: not a phone number
    assert!(!pattern.is_match("12345"));
    assert!(!pattern.is_match("abc-def-ghij"));
}

#[test]
fn test_phone_pattern_multiple_in_text() {
    let pattern = get_phone_number_pattern();
    let text = "Call (555) 234-5678 or 555-987-6543 for help";
    
    let matches: Vec<_> = pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_phone_pattern_preserves_other_text() {
    let pattern = get_phone_number_pattern();
    let text = "Employee ID: 12345, Phone: (555) 234-5678";
    
    // Should only match the phone, not the ID
    let matches: Vec<_> = pattern.find_iter(text).collect();
    assert_eq!(matches.len(), 1);
    assert!(matches[0].as_str().contains("555"));
}

#[test]
fn test_account_pattern_formats() {
    // Test various account number formats
    let pattern_9_5 = Regex::new(r"(\d{9})-(\d{5})").unwrap();
    
    assert!(pattern_9_5.is_match("123456789-00001"));
    assert!(!pattern_9_5.is_match("12345678900001")); // No dash
    assert!(!pattern_9_5.is_match("123456789-0001")); // Wrong suffix length
}

#[test]
fn test_account_pattern_14_digits() {
    let pattern_14 = Regex::new(r"\b(\d{14})\b").unwrap();
    
    assert!(pattern_14.is_match("12345678900001"));
    assert!(!pattern_14.is_match("1234567890001")); // 13 digits
    assert!(!pattern_14.is_match("123456789000012")); // 15 digits
}
