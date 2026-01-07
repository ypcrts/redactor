//! Unit tests for domain models and business logic.

use anyhow::Result;
use redactor::domain::{PhoneNumberMatcher, VerizonAccountMatcher, PatternMatcher};

#[test]
fn test_phone_matcher_creation() {
    let matcher = PhoneNumberMatcher::new();
    assert!(matcher.pattern().as_str().len() > 0);
}

#[test]
fn test_phone_normalization_valid_nanp() {
    let matcher = PhoneNumberMatcher::new();
    
    // Valid NANP numbers (area and exchange must start with 2-9)
    assert_eq!(
        matcher.normalize("(555) 234-5678"),
        Some("5552345678".to_string())
    );
    
    assert_eq!(
        matcher.normalize("555-234-5678"),
        Some("5552345678".to_string())
    );
}

#[test]
fn test_phone_variants_generation() {
    let matcher = PhoneNumberMatcher::new();
    let variants = matcher.generate_variants("5552345678");
    
    assert!(variants.len() >= 3, "Should generate multiple variants");
    assert!(variants.contains(&"5552345678".to_string()));
    assert!(variants.contains(&"555-234-5678".to_string()));
    assert!(variants.contains(&"(555) 234-5678".to_string()));
}

#[test]
fn test_verizon_account_matcher_creation() {
    let matcher = VerizonAccountMatcher::new();
    // Should not panic
    let _ = matcher.pattern();
}

#[test]
fn test_verizon_account_detection() {
    let text = "Account Number: 123456789-00001";
    let account = VerizonAccountMatcher::find_account_number(text);
    
    assert_eq!(account, Some("12345678900001".to_string()));
}

#[test]
fn test_verizon_account_priority() {
    // Should prefer account with context over standalone
    let text = "Random: 999999999-99999 Account: 123456789-00001";
    let account = VerizonAccountMatcher::find_account_number(text);
    
    assert_eq!(account, Some("12345678900001".to_string()));
}

#[test]
fn test_verizon_account_variants() {
    let matcher = VerizonAccountMatcher::new();
    let variants = matcher.generate_variants("12345678900001");
    
    assert!(variants.contains(&"12345678900001".to_string()));
    assert!(variants.contains(&"123456789-00001".to_string()));
    assert!(variants.contains(&"123456789 00001".to_string()));
    assert_eq!(variants.len(), 3);
}

#[test]
fn test_no_account_found() {
    let text = "This document has no account number";
    let account = VerizonAccountMatcher::find_account_number(text);
    
    assert_eq!(account, None);
}

#[test]
fn test_invalid_area_code() {
    let matcher = PhoneNumberMatcher::new();
    
    // Area code cannot start with 0 or 1
    assert!(!PhoneNumberMatcher::validate("155", "234", "5678"));
    assert!(!PhoneNumberMatcher::validate("055", "234", "5678"));
    
    // Valid area code
    assert!(PhoneNumberMatcher::validate("555", "234", "5678"));
}
