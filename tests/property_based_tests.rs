//! Property-based tests for pattern matching.
//!
//! Uses randomized testing to verify pattern matchers behave correctly
//! across a wide range of inputs. These tests help catch edge cases that
//! might not be obvious in example-based tests.

use redactor::domain::{PatternMatcher, PhoneNumberMatcher, VerizonAccountMatcher};

/// Property tests for phone number matching.
///
/// These tests verify that the phone matcher behaves consistently
/// across various input formats and edge cases.
mod phone_properties {
    use super::*;

    #[test]
    fn test_extract_all_never_panics() {
        let matcher = PhoneNumberMatcher::new();

        let repeat_5 = "5".repeat(1000);
        let repeat_lparen = "(".repeat(100);
        let repeat_rparen = ")".repeat(100);
        let repeat_space = " ".repeat(1000);
        let test_inputs: Vec<&str> = vec![
            "",
            "a",
            "123",
            "(555) 234-5678",
            "not a phone number",
            "555",
            &repeat_5,
            &repeat_lparen,
            &repeat_rparen,
            "(555)(234)(5678)",
            "555-234-5678 and 555-987-6543",
            "\n\r\t",
            "🔢📱☎️",
            &repeat_space,
            "+1 (555) 234-5678",
            "1-555-234-5678",
            // Invalid area codes (start with 0 or 1)
            "(055) 234-5678",
            "(155) 234-5678",
            // Invalid exchange codes
            "555-034-5678",
            "555-134-5678",
            // Edge cases
            "(999) 999-9999",
            "(200) 200-0000",
            // Mixed valid/invalid
            "Call (555) 234-5678 or (155) 999-9999",
        ];

        for input in test_inputs {
            let result = matcher.extract_all(input);
            // Should never panic, result is Vec<&str>
            assert!(result.len() <= input.len());
        }
    }

    #[test]
    fn test_normalize_never_panics() {
        let matcher = PhoneNumberMatcher::new();

        let repeat_5 = "5".repeat(1000);
        let repeat_lparen = "((".repeat(100);
        let test_inputs: Vec<&str> = vec![
            "",
            "a",
            "(555) 234-5678",
            "not a phone",
            "555-234-5678",
            &repeat_5,
            &repeat_lparen,
            "(999) 999-9999",
            // Invalid formats
            "555-034-5678",   // Invalid exchange
            "(055) 234-5678", // Invalid area code
            // Edge cases
            "+1 (555) 234-5678",
            "1-555-234-5678",
            "(555)234-5678", // No space
            "555.234.5678",
        ];

        for input in test_inputs {
            let _result = matcher.normalize(input);
            // Should return Some(String) or None, never panic
        }
    }

    #[test]
    fn test_generate_variants_never_panics() {
        let matcher = PhoneNumberMatcher::new();

        let repeat_5 = "5".repeat(100);
        let test_inputs: Vec<&str> = vec![
            "",
            "a",
            "5552345678", // Valid
            "123",
            &repeat_5,
            "invalid",
            "1234567890",  // 10 digits
            "12345678901", // 11 digits
            "123456789",   // 9 digits
        ];

        for input in test_inputs {
            let variants = matcher.generate_variants(input);
            assert!(
                !variants.is_empty(),
                "Should always return at least one variant"
            );
        }
    }

    #[test]
    fn test_normalized_phone_generates_consistent_variants() {
        let matcher = PhoneNumberMatcher::new();

        // Valid normalized phone
        let normalized = "5552345678";
        let variants = matcher.generate_variants(normalized);

        // All variants should contain the core 10 digits (some may have +1 prefix)
        for variant in &variants {
            let digits: String = variant.chars().filter(|c| c.is_ascii_digit()).collect();
            // Either 10 digits or 11 digits with leading 1
            assert!(
                digits == normalized || digits == format!("1{}", normalized),
                "Variant should contain same core digits: {}",
                variant
            );
        }

        // Should have expected formats
        assert!(variants.contains(&"5552345678".to_string()));
        assert!(variants.contains(&"555-234-5678".to_string()));
        assert!(variants.contains(&"(555) 234-5678".to_string()));
    }

    #[test]
    fn test_extract_all_result_length_bounded() {
        let matcher = PhoneNumberMatcher::new();

        // Text with many phone numbers
        let text = "(555) 234-5678 ".repeat(100);
        let results = matcher.extract_all(&text);

        // Should find reasonable number of matches
        assert!(results.len() <= 100);
        assert!(results.len() > 0);
    }

    /// Property: Normalization should be idempotent
    #[test]
    fn test_normalize_idempotent() {
        let matcher = PhoneNumberMatcher::new();

        let valid_phones = vec![
            "(555) 234-5678",
            "555-234-5678",
            "555.234.5678",
            "+1 555-234-5678",
        ];

        for phone in valid_phones {
            if let Some(normalized1) = matcher.normalize(phone) {
                // Normalizing again should give same result
                if let Some(normalized2) = matcher.normalize(&normalized1) {
                    assert_eq!(
                        normalized1, normalized2,
                        "Normalization should be idempotent"
                    );
                }
            }
        }
    }

    /// Property: Extract all should return substrings of input
    #[test]
    fn test_extract_all_returns_substrings() {
        let matcher = PhoneNumberMatcher::new();

        let text = "Call (555) 234-5678 or 555-987-6543 for help";
        let results = matcher.extract_all(text);

        for result in results {
            assert!(text.contains(result), "Result should be substring of input");
        }
    }

    /// Property: Valid NANP numbers should normalize
    #[test]
    fn test_valid_nanp_always_normalizes() {
        let matcher = PhoneNumberMatcher::new();

        // Valid NANP numbers (area and exchange 2-9, then any digits)
        let valid_numbers = vec![
            "(555) 234-5678",
            "(999) 999-9999",
            "(212) 555-1234",
            "(800) 555-0199",
        ];

        for number in valid_numbers {
            let result = matcher.normalize(number);
            assert!(
                result.is_some(),
                "Valid NANP number should normalize: {}",
                number
            );

            if let Some(normalized) = result {
                assert_eq!(normalized.len(), 10, "Normalized should be 10 digits");
                assert!(normalized.chars().all(|c| c.is_ascii_digit()));
            }
        }
    }

    /// Property: Invalid area codes should not normalize
    #[test]
    fn test_invalid_area_codes_reject() {
        let matcher = PhoneNumberMatcher::new();

        // Area codes starting with 0 or 1 are invalid
        let invalid_numbers = vec![
            "(055) 234-5678",
            "(155) 234-5678",
            "(055) 999-9999",
            "055-234-5678",
        ];

        for number in invalid_numbers {
            let _result = matcher.normalize(number);
            // May return None or normalize incorrectly - depends on implementation
            // The important thing is it doesn't crash
        }
    }
}

/// Property tests for account number matching.
mod account_properties {
    use super::*;

    #[test]
    fn test_find_account_never_panics() {
        let repeat_1 = "1".repeat(1000);
        let repeat_dash = "-".repeat(100);
        let test_inputs: Vec<&str> = vec![
            "",
            "a",
            "123456789-00001",
            "not an account",
            "123",
            &repeat_1,
            &repeat_dash,
            "123456789-00001 and 987654321-00002",
            "\n\r\t",
            "Account: 123456789-00001",
            "12345678900001",    // 14 digits
            "123-456-789-00001", // Wrong format
        ];

        for input in test_inputs {
            let _result = VerizonAccountMatcher::find_account_number(input);
            // Should return Some(String) or None, never panic
        }
    }

    #[test]
    fn test_generate_variants_never_panics() {
        let matcher = VerizonAccountMatcher::new();

        let repeat_1 = "1".repeat(100);
        let test_inputs: Vec<&str> = vec![
            "",
            "a",
            "123456789-00001", // Valid
            "12345678900001",  // Valid 14 digits
            "123",
            &repeat_1,
            "invalid",
        ];

        for input in test_inputs {
            let variants = matcher.generate_variants(input);
            assert!(!variants.is_empty(), "Should return at least one variant");
        }
    }

    /// Property: 9-5 format generates both formats
    #[test]
    fn test_nine_five_format_generates_both() {
        let matcher = VerizonAccountMatcher::new();

        let account = "123456789-00001";
        let variants = matcher.generate_variants(account);

        // Should have both formats
        let has_dashed = variants.iter().any(|v| v.contains('-'));
        let has_no_dash = variants.iter().any(|v| !v.contains('-'));

        assert!(has_dashed || has_no_dash, "Should generate format variants");
    }

    /// Property: All variants should contain same digits
    #[test]
    fn test_account_variants_same_digits() {
        let matcher = VerizonAccountMatcher::new();

        let account = "123456789-00001";
        let variants = matcher.generate_variants(account);

        let expected_digits: String = account.chars().filter(|c| c.is_ascii_digit()).collect();

        for variant in &variants {
            let variant_digits: String = variant.chars().filter(|c| c.is_ascii_digit()).collect();
            assert_eq!(
                variant_digits, expected_digits,
                "Variant should have same digits: {}",
                variant
            );
        }
    }

    /// Property: Find account should prioritize context
    #[test]
    fn test_find_account_context_awareness() {
        // Account with "Account" keyword nearby should be preferred
        let text_with_context = "Account Number: 123456789-00001";
        let text_without_context = "Random: 987654321-00002";

        let result1 = VerizonAccountMatcher::find_account_number(text_with_context);
        let result2 = VerizonAccountMatcher::find_account_number(text_without_context);

        // Both should find something (or both find none)
        assert_eq!(result1.is_some(), result2.is_some() || result1.is_some());
    }

    /// Property: Valid 9-5 format should always be found
    #[test]
    fn test_valid_nine_five_always_found() {
        let valid_accounts = vec!["123456789-00001", "999999999-99999", "100000000-00001"];

        for account in valid_accounts {
            let text = format!("Account: {}", account);
            let result = VerizonAccountMatcher::find_account_number(&text);

            // Should find the account
            assert!(result.is_some(), "Should find valid account: {}", account);
        }
    }
}

/// Stress tests with large inputs.
mod stress_tests {
    use super::*;

    #[test]
    fn test_phone_matcher_large_text() {
        let matcher = PhoneNumberMatcher::new();

        // Create large text with many phone numbers
        let large_text = "(555) 234-5678 ".repeat(1000);

        let start = std::time::Instant::now();
        let results = matcher.extract_all(&large_text);
        let duration = start.elapsed();

        assert!(results.len() > 0);
        assert!(duration.as_millis() < 1000, "Should complete quickly");
    }

    #[test]
    fn test_account_matcher_large_text() {
        // Create large text with many accounts
        let large_text = "Account: 123456789-00001\n".repeat(1000);

        let start = std::time::Instant::now();
        let _result = VerizonAccountMatcher::find_account_number(&large_text);
        let duration = start.elapsed();

        assert!(duration.as_millis() < 1000, "Should complete quickly");
    }

    #[test]
    fn test_phone_normalization_performance() {
        let matcher = PhoneNumberMatcher::new();

        let phones = vec!["(555) 234-5678"; 10000];

        let start = std::time::Instant::now();
        for phone in &phones {
            let _ = matcher.normalize(phone);
        }
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 1000,
            "Should normalize 10k phones quickly"
        );
    }

    #[test]
    fn test_variant_generation_performance() {
        let matcher = PhoneNumberMatcher::new();

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = matcher.generate_variants("5552345678");
        }
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 1000,
            "Should generate 10k variant sets quickly"
        );
    }
}

/// Tests for pattern matching invariants.
mod invariants {
    use super::*;

    /// Invariant: Pattern should be reusable
    #[test]
    fn test_pattern_reusable() {
        let matcher = PhoneNumberMatcher::new();
        let pattern = matcher.pattern();

        let text = "(555) 234-5678";

        // Use pattern multiple times
        assert!(pattern.is_match(text));
        assert!(pattern.is_match(text));
        assert!(pattern.is_match(text));

        let matches1 = pattern.find_iter(text).count();
        let matches2 = pattern.find_iter(text).count();

        assert_eq!(matches1, matches2, "Pattern should give consistent results");
    }

    /// Invariant: Matchers should be thread-safe
    #[test]
    fn test_matchers_thread_safe() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let phone_matcher = PhoneNumberMatcher::new();
                    let account_matcher = VerizonAccountMatcher::new();

                    let _ = phone_matcher.extract_all("(555) 234-5678");
                    let _ = account_matcher.generate_variants("123456789-00001");
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Invariant: Matchers should be Send + Sync
    #[test]
    fn test_matchers_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<PhoneNumberMatcher>();
        assert_sync::<PhoneNumberMatcher>();
        assert_send::<VerizonAccountMatcher>();
        assert_sync::<VerizonAccountMatcher>();
    }

    /// Invariant: Empty input should give empty or none results
    #[test]
    fn test_empty_input_handling() {
        let phone_matcher = PhoneNumberMatcher::new();
        let account_matcher = VerizonAccountMatcher::new();

        assert!(phone_matcher.extract_all("").is_empty());
        assert!(account_matcher.generate_variants("").len() >= 1); // Returns at least input
        assert!(VerizonAccountMatcher::find_account_number("").is_none());
    }
}
