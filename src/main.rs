//! PDF Redaction CLI Application.
//!
//! This binary provides a command-line interface for the redactor library,
//! supporting various redaction modes with proper error handling and user feedback.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use redactor::{RedactionService, RedactionTarget, SecureRedactionStrategy};

/// PDF Redaction Tool
///
/// Securely redact sensitive information from PDF documents.
/// By default, performs redaction. Use 'extract' subcommand for text extraction.
#[derive(Parser)]
#[command(name = "redactor")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input PDF file path
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output PDF file path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Text patterns to redact (can be specified multiple times)
    #[arg(short, long, value_name = "PATTERN")]
    pattern: Vec<String>,

    /// Redact American phone numbers
    #[arg(long, conflicts_with = "all")]
    phones: bool,

    /// Redact Verizon account number (automatically includes phone numbers and call details)
    #[arg(long, conflicts_with = "all")]
    verizon: bool,

    /// Redact all text (complete document redaction)
    #[arg(long, conflicts_with_all = ["phones", "verizon"])]
    all: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract text from a PDF (for debugging and verification)
    Extract {
        /// Input PDF file path
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output text file (optional, defaults to stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

/// Redaction command handler with dependency injection.
struct RedactionHandler {
    service: RedactionService,
    verbose: bool,
}

impl RedactionHandler {
    /// Creates a new handler with the secure redaction strategy.
    fn new(verbose: bool) -> Self {
        let strategy = SecureRedactionStrategy::new();
        Self {
            service: RedactionService::new(Box::new(strategy)),
            verbose,
        }
    }

    /// Executes a redaction operation.
    fn redact(&self, input: &Path, output: &Path, targets: Vec<RedactionTarget>) -> Result<()> {
        // Validate inputs
        if !input.exists() {
            anyhow::bail!("Input file does not exist: {}", input.display());
        }

        if targets.is_empty() {
            anyhow::bail!(
                "No redaction targets specified. Use --pattern, --phones, --verizon, or --all."
            );
        }

        if self.verbose {
            println!("Input:  {}", input.display());
            println!("Output: {}", output.display());
            println!("Targets: {} redaction target(s)", targets.len());
        }

        // Perform redaction
        let result = self
            .service
            .redact(input, output, &targets)
            .with_context(|| "Redaction failed")?;

        // Report results
        if self.verbose {
            println!("\nRedaction Summary:");
            println!("  Pages processed: {}", result.pages_processed);
            println!("  Pages modified:  {}", result.pages_modified);
            println!("  Instances redacted: {}", result.instances_redacted);
            println!(
                "  Secure: {}",
                if result.secure {
                    "Yes"
                } else {
                    "No (visual only)"
                }
            );
        }

        if result.instances_redacted > 0 {
            println!(
                "✓ Successfully redacted {} instance(s) → {}",
                result.instances_redacted,
                output.display()
            );
        } else {
            println!("⚠ No instances found to redact");
        }

        Ok(())
    }

    /// Extracts text from a PDF.
    fn extract(&self, input: &Path, output: Option<&Path>) -> Result<()> {
        if !input.exists() {
            anyhow::bail!("Input file does not exist: {}", input.display());
        }

        let text = self
            .service
            .extract_text(input)
            .with_context(|| "Text extraction failed")?;

        if let Some(output_path) = output {
            std::fs::write(output_path, &text)
                .with_context(|| format!("Failed to write to {}", output_path.display()))?;
            println!(
                "✓ Extracted {} characters → {}",
                text.len(),
                output_path.display()
            );
        } else {
            println!("{}", text);
        }

        Ok(())
    }
}

/// Parses command-line arguments and builds redaction targets.
fn build_targets(
    patterns: &[String],
    phones: bool,
    verizon: bool,
    all: bool,
) -> Vec<RedactionTarget> {
    let mut targets = Vec::new();

    if all {
        targets.push(RedactionTarget::AllText);
    } else {
        // Add Verizon account if requested
        if verizon {
            targets.push(RedactionTarget::VerizonAccount);
            // Verizon bills contain phone numbers, so automatically redact them too
            targets.push(RedactionTarget::PhoneNumbers);
            // Also redact call detail columns (time, origination, destination)
            targets.push(RedactionTarget::VerizonCallDetails);
        }

        // Add phone numbers if requested (and not already added by verizon flag)
        if phones && !verizon {
            targets.push(RedactionTarget::PhoneNumbers);
        }

        // Add literal patterns if specified
        targets.extend(patterns.iter().map(|p| RedactionTarget::Literal(p.clone())));
    }

    targets
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let handler = RedactionHandler::new(cli.verbose);

    match &cli.command {
        Some(Commands::Extract { input, output }) => {
            // Extract subcommand
            handler.extract(input, output.as_deref())?;
        }
        None => {
            // Default: redaction mode
            let input = cli
                .input
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("--input is required"))?;
            let output = cli
                .output
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("--output is required"))?;

            let targets = build_targets(&cli.pattern, cli.phones, cli.verizon, cli.all);
            handler.redact(input, output, targets)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_building() {
        // Test verizon flag (should include phones and call details automatically)
        let targets = build_targets(&[], false, true, false);
        assert_eq!(targets.len(), 3); // VerizonAccount + PhoneNumbers + VerizonCallDetails

        // Test literal pattern
        let targets = build_targets(&[String::from("test")], false, false, false);
        assert_eq!(targets.len(), 1);
        assert!(matches!(targets[0], RedactionTarget::Literal(_)));

        // Test phones flag
        let targets = build_targets(&[], true, false, false);
        assert_eq!(targets.len(), 1);
        assert!(matches!(targets[0], RedactionTarget::PhoneNumbers));
    }
}
