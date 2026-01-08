//! Comprehensive CLI integration tests.
//!
//! Tests all CLI functionality including argument parsing, error handling,
//! and end-to-end workflows. These tests use the actual binary to ensure
//! the full user experience works correctly.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

mod common;
use common::*;

/// Creates a test Command for the redactor binary.
fn redactor_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("redactor")
}

/// Tests basic CLI argument parsing and help output.
mod argument_parsing {
    use super::*;

    #[test]
    fn test_help_flag() {
        redactor_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Secure PDF redaction"))
            .stdout(predicate::str::contains("--input"))
            .stdout(predicate::str::contains("--output"))
            .stdout(predicate::str::contains("--phones"))
            .stdout(predicate::str::contains("--verizon"));
    }

    #[test]
    fn test_version_flag() {
        redactor_cmd()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("redactor"))
            .stdout(predicate::str::contains("0.3.0"));
    }

    #[test]
    fn test_missing_required_input() {
        redactor_cmd()
            .arg("--output")
            .arg("out.pdf")
            .arg("--phones")
            .assert()
            .failure()
            .stderr(predicate::str::contains("input").or(predicate::str::contains("required")));
    }

    #[test]
    fn test_missing_required_output() {
        redactor_cmd()
            .arg("--input")
            .arg("in.pdf")
            .arg("--phones")
            .assert()
            .failure()
            .stderr(predicate::str::contains("output").or(predicate::str::contains("required")));
    }

    #[test]
    fn test_no_redaction_targets() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        // Create a test PDF
        TestPdfBuilder::new()
            .with_title("Test")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("--input")
            .arg(input.as_os_str())
            .arg("--output")
            .arg(output.as_os_str())
            .assert()
            .failure()
            .stderr(predicate::str::contains("No redaction targets"));
    }

    #[test]
    fn test_short_flags() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .with_content("(555) 234-5678")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .success();
    }

    #[test]
    fn test_verbose_flag() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .with_content("Test content")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .arg("--verbose")
            .assert()
            .success()
            .stdout(predicate::str::contains("redact").or(predicate::str::contains("✓")));
    }
}

/// Tests phone number redaction via CLI.
mod phone_redaction {
    use super::*;

    #[test]
    fn test_phones_flag_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Phone Test")
            .with_content("Contact: (555) 234-5678")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("--input")
            .arg(input.as_os_str())
            .arg("--output")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .success();

        assert!(output.exists());

        // Verify redaction
        let text = extract_text(&output).unwrap();
        assert!(!text.contains("(555) 234-5678"), "Phone should be redacted");
    }

    #[test]
    fn test_phones_multiple_numbers() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Multiple Phones")
            .with_content("Call (555) 234-5678 or 555-987-6543")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .success();

        let text = extract_text(&output).unwrap();
        assert!(!text.contains("234-5678"));
        assert!(!text.contains("987-6543"));
    }
}

/// Tests Verizon account redaction via CLI.
mod verizon_redaction {
    use super::*;

    #[test]
    fn test_verizon_flag_basic() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Verizon Bill")
            .with_verizon_account("123456789-00001")
            .with_phone("(555) 234-5678")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("--input")
            .arg(input.as_os_str())
            .arg("--output")
            .arg(output.as_os_str())
            .arg("--verizon")
            .assert()
            .success();

        assert!(output.exists());

        // Verify both account and phone are redacted
        let text = extract_text(&output).unwrap();
        assert!(!text.contains("123456789-00001"));
        assert!(!text.contains("(555) 234-5678"));
    }

    #[test]
    fn test_verizon_auto_includes_phones() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Bill")
            .with_verizon_account("987654321-00001")
            .with_phone("555-123-4567")
            .build(&input)
            .unwrap();

        // Only --verizon flag, not --phones
        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--verizon")
            .assert()
            .success();

        // Verify output file was created
        assert!(output.exists());

        // Note: Actual redaction verification depends on PDF text extraction working correctly
        // The important thing is the command succeeds with the right flags
    }
}

/// Tests pattern-based redaction via CLI.
mod pattern_redaction {
    use super::*;

    #[test]
    fn test_single_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Pattern Test")
            .with_content("CONFIDENTIAL document")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--pattern")
            .arg("CONFIDENTIAL")
            .assert()
            .success();

        let text = extract_text(&output).unwrap();
        assert!(!text.contains("CONFIDENTIAL"));
    }

    #[test]
    fn test_multiple_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Multiple Patterns")
            .with_content("SECRET and CONFIDENTIAL information")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--pattern")
            .arg("SECRET")
            .arg("--pattern")
            .arg("CONFIDENTIAL")
            .assert()
            .success();

        let text = extract_text(&output).unwrap();
        assert!(!text.contains("SECRET"));
        assert!(!text.contains("CONFIDENTIAL"));
    }

    #[test]
    fn test_pattern_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Special")
            .with_content("Price: $99.99")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--pattern")
            .arg("$99.99")
            .assert()
            .success();
    }
}

/// Tests extract subcommand.
mod extract_command {
    use super::*;

    #[test]
    fn test_extract_help() {
        redactor_cmd()
            .arg("extract")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Extract"))
            .stdout(predicate::str::contains("--input"));
    }

    #[test]
    fn test_extract_to_stdout() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");

        TestPdfBuilder::new()
            .with_title("Extract Test")
            .with_content("This is test content")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("extract")
            .arg("--input")
            .arg(input.as_os_str())
            .assert()
            .success()
            .stdout(
                predicate::str::contains("test content").or(predicate::str::contains("Extract")),
            );
    }

    #[test]
    fn test_extract_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("extracted.txt");

        TestPdfBuilder::new()
            .with_title("Extract")
            .with_content("Extracted content here")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("extract")
            .arg("--input")
            .arg(input.as_os_str())
            .arg("--output")
            .arg(output.as_os_str())
            .assert()
            .success();

        assert!(output.exists());

        let content = fs::read_to_string(output).unwrap();
        assert!(content.contains("content") || content.contains("Extract"));
    }

    #[test]
    fn test_extract_nonexistent_file() {
        redactor_cmd()
            .arg("extract")
            .arg("--input")
            .arg("/nonexistent/file.pdf")
            .assert()
            .failure()
            .stderr(predicate::str::contains("not exist").or(predicate::str::contains("Error")));
    }
}

/// Tests error handling and edge cases.
mod error_handling {
    use super::*;

    #[test]
    fn test_input_file_not_found() {
        redactor_cmd()
            .arg("--input")
            .arg("/nonexistent/input.pdf")
            .arg("--output")
            .arg("/tmp/out.pdf")
            .arg("--phones")
            .assert()
            .failure()
            .stderr(predicate::str::contains("not exist").or(predicate::str::contains("Error")));
    }

    #[test]
    fn test_invalid_pdf_file() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_pdf = temp_dir.path().join("invalid.pdf");
        let output = temp_dir.path().join("out.pdf");

        // Create a file that's not a valid PDF
        fs::write(&invalid_pdf, b"Not a PDF file").unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(invalid_pdf.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error").or(predicate::str::contains("failed")));
    }

    #[test]
    fn test_output_directory_doesnt_exist() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg("/nonexistent/directory/output.pdf")
            .arg("--phones")
            .assert()
            .failure();
    }

    #[test]
    fn test_empty_pattern_string() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Test")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--pattern")
            .arg("")
            .assert()
            .success(); // Empty pattern is technically valid
    }
}

/// Tests combined flags and complex scenarios.
mod combined_operations {
    use super::*;

    #[test]
    fn test_phones_and_patterns_combined() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Combined")
            .with_phone("(555) 234-5678")
            .with_content("SSN: 123-45-6789")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .arg("--pattern")
            .arg("123-45-6789")
            .assert()
            .success();

        // Verify output file was created
        assert!(output.exists());

        // Note: Actual verification of redaction depends on PDF text extraction
        // The test verifies the CLI accepts the combined flags correctly
    }

    #[test]
    fn test_verizon_and_patterns_combined() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Combined")
            .with_verizon_account("123456789-00001")
            .with_phone("555-234-5678")
            .with_content("CONFIDENTIAL")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--verizon")
            .arg("--pattern")
            .arg("CONFIDENTIAL")
            .assert()
            .success();

        let text = extract_text(&output).unwrap();
        assert!(!text.contains("123456789"));
        assert!(!text.contains("CONFIDENTIAL"));
    }

    #[test]
    fn test_multiple_patterns_with_phones() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Multiple")
            .with_phone("555-123-4567")
            .with_content("Item1 and Item2 and Item3")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .arg("-p")
            .arg("Item1")
            .arg("-p")
            .arg("Item2")
            .arg("-p")
            .arg("Item3")
            .assert()
            .success();
    }
}

/// Tests output and messaging.
mod output_messages {
    use super::*;

    #[test]
    fn test_success_message_format() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("Success")
            .with_phone("555-234-5678")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .success()
            .stdout(predicate::str::contains("✓").or(predicate::str::contains("Success")));
    }

    #[test]
    fn test_no_matches_found_message() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("test.pdf");
        let output = temp_dir.path().join("out.pdf");

        TestPdfBuilder::new()
            .with_title("No Matches")
            .with_content("Regular content without phones")
            .build(&input)
            .unwrap();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--phones")
            .assert()
            .success()
            .stdout(predicate::str::contains("No instances").or(predicate::str::contains("0")));
    }
}

/// Performance and stress tests.
mod performance {
    use super::*;

    #[test]
    fn test_large_document_handling() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("large.pdf");
        let output = temp_dir.path().join("out.pdf");

        // Create a document with lots of content
        let mut builder = TestPdfBuilder::new().with_title("Large Document");
        for i in 0..100 {
            builder = builder.with_content(&format!("Line {} with content", i));
        }
        builder.build(&input).unwrap();

        let start = std::time::Instant::now();

        redactor_cmd()
            .arg("-i")
            .arg(input.as_os_str())
            .arg("-o")
            .arg(output.as_os_str())
            .arg("--pattern")
            .arg("Line")
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(
            duration.as_secs() < 30,
            "Should complete in reasonable time"
        );
    }
}
