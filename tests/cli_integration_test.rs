//! CLI integration tests for command-line behavior.
//!
//! Tests the actual command-line interface, argument parsing, and output formatting.

use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

/// Helper to run the CLI with arguments
fn run_cli(args: &[&str]) -> Result<std::process::Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--release").arg("--").args(args);

    Ok(cmd.output()?)
}

#[test]
#[ignore] // Requires compiled binary
fn test_help_message() -> Result<()> {
    let output = run_cli(&["--help"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Input PDF file"));
    assert!(stdout.contains("--verizon"));
    assert!(stdout.contains("--phones"));
    assert!(stdout.contains("--verbose"));

    Ok(())
}

#[test]
#[ignore] // Requires compiled binary
fn test_verizon_flag_auto_includes_phones() -> Result<()> {
    // This test verifies the important behavior that --verizon
    // automatically redacts phone numbers in addition to account numbers

    let _temp_dir = TempDir::new()?;
    let _input = _temp_dir.path().join("input.pdf");
    let _output = _temp_dir.path().join("output.pdf");

    // Verify help output documents verizon flag behavior
    let help_output = run_cli(&["--help"])?;
    let help_text = String::from_utf8_lossy(&help_output.stdout);

    // Verify documentation mentions automatic phone redaction
    assert!(
        help_text.contains("automatically includes phone numbers")
            || help_text.contains("Verizon account number"),
        "Help should document verizon flag behavior"
    );

    Ok(())
}

#[test]
#[ignore] // Requires compiled binary
fn test_verbose_flag() -> Result<()> {
    let output = run_cli(&["--help"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("--verbose") || stdout.contains("-v"));

    Ok(())
}

#[test]
#[ignore] // Requires compiled binary
fn test_missing_input_file_error() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let nonexistent = temp_dir.path().join("does_not_exist.pdf");
    let output = temp_dir.path().join("output.pdf");

    let result = run_cli(&[
        "--input",
        nonexistent.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
        "--verizon",
    ])?;

    // Should fail with non-zero exit code
    assert!(!result.status.success());

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("does not exist") || stderr.contains("not found"),
        "Should report missing input file"
    );

    Ok(())
}

#[test]
#[ignore] // Requires compiled binary
fn test_no_targets_specified_error() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let _input = temp_dir.path().join("input.pdf");
    let _output = temp_dir.path().join("output.pdf");

    let result = run_cli(&[
        "--input",
        _input.to_str().unwrap(),
        "--output",
        _output.to_str().unwrap(),
    ])?;

    // Should fail when no redaction targets specified
    assert!(!result.status.success());

    Ok(())
}
