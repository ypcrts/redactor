//! Tests for PDF utility functions.
//!
//! Comprehensive tests for PDF escape sequence handling and pattern matching
//! to ensure correct parsing of PDF text content.

use redactor::domain::{PdfEscapes, PdfPatterns};

/// Tests PDF escape sequence unescaping.
///
/// PDF text can contain various escape sequences that need to be properly
/// decoded for accurate pattern matching.
mod pdf_escapes_tests {
    use super::*;

    #[test]
    fn test_unescape_space() {
        assert_eq!(PdfEscapes::unescape("Hello\\040World"), "Hello World");
    }

    #[test]
    fn test_unescape_left_parenthesis() {
        assert_eq!(PdfEscapes::unescape("\\050555\\051"), "(555)");
    }

    #[test]
    fn test_unescape_right_parenthesis() {
        assert_eq!(PdfEscapes::unescape("Call\\051Now"), "Call)Now");
    }

    #[test]
    fn test_unescape_newline() {
        assert_eq!(PdfEscapes::unescape("Line1\\nLine2"), "Line1\nLine2");
    }

    #[test]
    fn test_unescape_carriage_return() {
        assert_eq!(PdfEscapes::unescape("Text\\rMore"), "Text\rMore");
    }

    #[test]
    fn test_unescape_tab() {
        assert_eq!(PdfEscapes::unescape("Col1\\tCol2"), "Col1\tCol2");
    }

    #[test]
    fn test_unescape_backslash() {
        assert_eq!(PdfEscapes::unescape("Path\\\\File"), "Path\\File");
    }

    #[test]
    fn test_unescape_multiple_sequences() {
        let input = "\\050555\\051\\040123-4567\\n";
        let expected = "(555) 123-4567\n";
        assert_eq!(PdfEscapes::unescape(input), expected);
    }

    #[test]
    fn test_unescape_empty_string() {
        assert_eq!(PdfEscapes::unescape(""), "");
    }

    #[test]
    fn test_unescape_no_escapes() {
        let text = "Regular text with no escapes";
        assert_eq!(PdfEscapes::unescape(text), text);
    }

    #[test]
    fn test_unescape_consecutive_escapes() {
        assert_eq!(PdfEscapes::unescape("\\n\\n\\n"), "\n\n\n");
    }

    #[test]
    fn test_unescape_mixed_with_regular_text() {
        let input = "Phone:\\040\\050555\\051\\040123-4567";
        let expected = "Phone: (555) 123-4567";
        assert_eq!(PdfEscapes::unescape(input), expected);
    }

    #[test]
    fn test_unescape_preserves_non_escape_backslashes() {
        // Backslash not followed by a known escape sequence
        let input = "Test\\xYZ";
        // Should only replace known escape sequences
        assert_eq!(PdfEscapes::unescape(input), "Test\\xYZ");
    }

    #[test]
    fn test_unescape_unicode_preservation() {
        let input = "Hello\\040世界\\n日本語";
        let expected = "Hello 世界\n日本語";
        assert_eq!(PdfEscapes::unescape(input), expected);
    }

    #[test]
    fn test_unescape_very_long_string() {
        let mut input = String::new();
        let mut expected = String::new();

        for _ in 0..1000 {
            input.push_str("Text\\040");
            expected.push_str("Text ");
        }

        assert_eq!(PdfEscapes::unescape(&input), expected);
    }

    #[test]
    fn test_unescape_all_escape_types_together() {
        let input = "\\050\\051\\040\\n\\r\\t\\\\";
        let expected = "() \n\r\t\\";
        assert_eq!(PdfEscapes::unescape(input), expected);
    }

    /// Property-based test: unescaping should never panic
    #[test]
    fn test_unescape_never_panics() {
        let large_a = "a".repeat(10000);
        let test_strings: Vec<&str> = vec![
            "",
            "\\",
            "\\\\",
            "\\040",
            "\\x",
            "\\0",
            "\\999",
            "\\040\\050\\051",
            &large_a,
        ];

        for s in test_strings {
            let _ = PdfEscapes::unescape(s);
            // Success if no panic
        }
    }
}

/// Tests PDF pattern matching for text extraction.
mod pdf_patterns_tests {
    use super::*;

    #[test]
    fn test_text_string_pattern_basic() {
        let pattern = PdfPatterns::text_string();
        let text = "(Hello World)";

        assert!(pattern.is_match(text));

        let caps = pattern.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "Hello World");
    }

    #[test]
    fn test_text_string_pattern_with_phone() {
        let pattern = PdfPatterns::text_string();
        let text = "((555) 123-4567)";

        assert!(pattern.is_match(text));
    }

    #[test]
    fn test_text_string_pattern_empty_parens() {
        let pattern = PdfPatterns::text_string();
        let text = "()";

        assert!(pattern.is_match(text));

        let caps = pattern.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "");
    }

    #[test]
    fn test_text_string_pattern_multiple_matches() {
        let pattern = PdfPatterns::text_string();
        let text = "(First) and (Second) and (Third)";

        let matches: Vec<_> = pattern.find_iter(text).collect();
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_text_string_pattern_no_match() {
        let pattern = PdfPatterns::text_string();
        let text = "No parentheses here";

        assert!(!pattern.is_match(text));
    }

    #[test]
    fn test_text_string_pattern_nested_parens_limitation() {
        let pattern = PdfPatterns::text_string();
        // The pattern doesn't handle nested parens (PDF limitation)
        let text = "(outer (inner))";

        // Will match up to first closing paren
        assert!(pattern.is_match(text));
    }

    #[test]
    fn test_tj_array_pattern_basic() {
        let pattern = PdfPatterns::tj_array();
        let text = "[(Hello)] TJ";

        assert!(pattern.is_match(text));

        let caps = pattern.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "(Hello)");
    }

    #[test]
    fn test_tj_array_pattern_multiple_strings() {
        let pattern = PdfPatterns::tj_array();
        let text = "[(Hello)(World)(123)] TJ";

        assert!(pattern.is_match(text));

        let caps = pattern.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "(Hello)(World)(123)");
    }

    #[test]
    fn test_tj_array_pattern_with_numbers() {
        let pattern = PdfPatterns::tj_array();
        let text = "[(Text)-250(More)-100(Text)] TJ";

        assert!(pattern.is_match(text));
    }

    #[test]
    fn test_tj_array_pattern_with_whitespace() {
        let pattern = PdfPatterns::tj_array();
        let text = "[  (Hello)  (World)  ]  TJ";

        assert!(pattern.is_match(text));
    }

    #[test]
    fn test_tj_array_pattern_empty_array() {
        let pattern = PdfPatterns::tj_array();
        let text = "[] TJ";

        assert!(pattern.is_match(text));

        let caps = pattern.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "");
    }

    #[test]
    fn test_tj_array_pattern_no_match_without_tj() {
        let pattern = PdfPatterns::tj_array();
        let text = "[(Hello)(World)]";

        assert!(!pattern.is_match(text));
    }

    #[test]
    fn test_tj_array_pattern_case_sensitive() {
        let pattern = PdfPatterns::tj_array();
        let text_lower = "[(Hello)] tj";
        let text_upper = "[(Hello)] TJ";

        assert!(!pattern.is_match(text_lower));
        assert!(pattern.is_match(text_upper));
    }

    #[test]
    fn test_tj_array_pattern_in_pdf_stream() {
        let pattern = PdfPatterns::tj_array();
        let pdf_content = r#"
            BT
            /F1 12 Tf
            100 700 Td
            [(Account:)-250(123456789)] TJ
            ET
        "#;

        assert!(pattern.is_match(pdf_content));
    }

    #[test]
    fn test_tj_array_pattern_multiple_in_stream() {
        let pattern = PdfPatterns::tj_array();
        let pdf_content = "[(Line1)] TJ [(Line2)] TJ [(Line3)] TJ";

        let matches: Vec<_> = pattern.find_iter(pdf_content).collect();
        assert_eq!(matches.len(), 3);
    }

    /// Verify patterns are thread-safe (using Lazy static)
    #[test]
    fn test_patterns_are_thread_safe() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let pattern1 = PdfPatterns::text_string();
                    let pattern2 = PdfPatterns::tj_array();

                    assert!(pattern1.is_match("(test)"));
                    assert!(pattern2.is_match("[(test)] TJ"));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Verify patterns are compiled once and reused
    #[test]
    fn test_patterns_are_cached() {
        let ptr1 = PdfPatterns::text_string() as *const _;
        let ptr2 = PdfPatterns::text_string() as *const _;

        assert_eq!(ptr1, ptr2, "Patterns should be cached");
    }

    #[test]
    fn test_tj_array_with_real_pdf_content() {
        let pattern = PdfPatterns::tj_array();

        // Realistic PDF content with phone number
        let content = r#"
            [(Phone:)-200(\050555\051)-100(123-4567)] TJ
        "#;

        assert!(pattern.is_match(content));

        let caps = pattern.captures(content).unwrap();
        let text_array = caps.get(1).unwrap().as_str();

        // Verify we can extract the content
        assert!(text_array.contains("Phone"));
        assert!(text_array.contains("555"));
    }

    /// Integration test: patterns work together
    #[test]
    fn test_patterns_work_together() {
        let text_pattern = PdfPatterns::text_string();
        let tj_pattern = PdfPatterns::tj_array();

        let pdf_stream = "Before (Simple Text) After [(Array Text)] TJ End";

        assert!(text_pattern.is_match(pdf_stream));
        assert!(tj_pattern.is_match(pdf_stream));

        let text_matches: Vec<_> = text_pattern.find_iter(pdf_stream).collect();
        let tj_matches: Vec<_> = tj_pattern.find_iter(pdf_stream).collect();

        // text_pattern matches: (Simple Text) and (Array Text) - 2 total
        assert_eq!(text_matches.len(), 2);
        assert_eq!(tj_matches.len(), 1);
    }
}

/// Performance and edge case tests
mod edge_cases {
    use super::*;

    #[test]
    fn test_unescape_with_max_performance() {
        let input = "Text\\040".repeat(10000);
        let start = std::time::Instant::now();
        let result = PdfEscapes::unescape(&input);
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 1000,
            "Should complete in under 1 second"
        );
        assert!(result.contains("Text "));
    }

    #[test]
    fn test_pattern_matching_performance() {
        let pattern = PdfPatterns::text_string();
        let large_text = "(text)".repeat(1000);

        let start = std::time::Instant::now();
        let matches: Vec<_> = pattern.find_iter(&large_text).collect();
        let duration = start.elapsed();

        assert_eq!(matches.len(), 1000);
        assert!(duration.as_millis() < 100, "Should complete in under 100ms");
    }

    #[test]
    fn test_empty_input_edge_cases() {
        assert_eq!(PdfEscapes::unescape(""), "");

        let text_pattern = PdfPatterns::text_string();
        let tj_pattern = PdfPatterns::tj_array();

        assert!(!text_pattern.is_match(""));
        assert!(!tj_pattern.is_match(""));
    }

    #[test]
    fn test_special_characters_in_patterns() {
        let text_pattern = PdfPatterns::text_string();

        // PDF strings can contain various special characters
        let test_cases = vec![
            "(test@example.com)",
            "(Price: $19.99)",
            "(100% complete)",
            "(Version 2.0.1)",
            "(file-name_123.pdf)",
        ];

        for case in test_cases {
            assert!(text_pattern.is_match(case), "Failed for: {}", case);
        }
    }
}
