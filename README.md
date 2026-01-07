# redactor

[![Crates.io](https://img.shields.io/crates/v/redactor.svg)](https://crates.io/crates/redactor)
[![Documentation](https://docs.rs/redactor/badge.svg)](https://docs.rs/redactor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A PDF redaction library and CLI tool with secure text removal using MuPDF.  Redacts Verizon bills so you can expense them without leaking your call metadata.

## Use Case

Originally built for redacting **Verizon phone bills** before submitting them to employer expense reimbursement systems like **Concur**, or **Expensify**. 

When submitting phone bills for work expense reimbursement, you typically need to:
- ✅ Keep the billing amounts visible (for verification)
- ❌ Remove your account number (privacy/security)
- ❌ Remove personal phone numbers (privacy)
- ❌ Remove call detail information (times, locations, destinations)
- ❌ Remove other personal contact information

This tool ensures your sensitive information is **physically removed** from the PDF (not just blacked out), so it cannot be extracted by the expense system or anyone who views the document.

**Perfect for**: Freelancers, remote workers, and employees who need to submit redacted bills for work expenses while maintaining privacy.

## Features

- **Secure Redaction**: Physically removes text from PDFs (not just visual overlay)
- **Type3 Font Support**: Handles complex PDF encodings via MuPDF
- **Phone Number Detection**: Automatic NANP phone number redaction
- **Verizon Account Numbers**: Specialized detection for 9-5 format accounts
- **Call Detail Redaction**: Automatically redacts time, origination, and destination columns
- **Pattern Matching**: Literal strings and regex patterns
- **CLI & Library**: Use as a command-line tool or Rust library

## Installation

### As a CLI Tool

```bash
cargo install redactor
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
redactor = "0.2"
```

## Quick Start

### CLI Usage

#### Redact Verizon Bill for Expense Report

```bash
# Redact account number, phone numbers, and call details (recommended for expense reports)
redactor --input verizon-bill.pdf --output for-concur.pdf --verizon
```

This command will:
1. Find and remove your Verizon account number (e.g., `123456789-00001`)
2. Remove all phone numbers from the document
3. Remove call detail information (times like "10:26 PM", locations, destinations)
4. Preserve billing amounts and other expense-relevant information

#### Other Common Uses

```bash
# Redact only phone numbers
redactor --input document.pdf --output redacted.pdf --phones

# Redact custom patterns (e.g., email addresses)
redactor --input doc.pdf --output out.pdf --pattern "your.email@example.com"

# Extract text to verify what's in the PDF
redactor extract --input document.pdf --output text.txt
```

### Library Usage

```rust
use redactor::{RedactionService, RedactionTarget};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = RedactionService::with_secure_strategy();
    
    service.redact(
        Path::new("input.pdf"),
        Path::new("output.pdf"),
        &[RedactionTarget::PhoneNumbers]
    )?;
    
    Ok(())
}
```

## Security

This library uses MuPDF's built-in redaction system to **physically remove** text from PDFs, making it unextractable. This is more secure than visual-only redaction methods that just draw black boxes over text.

### Why This Matters for Expense Reports

Many expense systems (Concur, Expensify, etc.) can extract text from PDFs for automated processing. Simple "black box" redaction doesn't actually remove the text - it's still embedded in the PDF and can be extracted. This tool ensures your account numbers and personal phone numbers are truly gone before you submit to your employer.

### Verification

```bash
# After redaction, verify text is gone
redactor extract --input redacted.pdf
# Your account number and phone numbers should NOT appear in output
```

## Supported Patterns

### Phone Numbers (NANP)
- `(555) 123-4567`
- `555-987-6543`
- `555.111.2222`
- `+1 555 234 5678`

### Verizon Accounts
- `123456789-00001` (9-5 format)
- `12345678900001` (14 digits)
- Context-aware detection

### Custom Patterns
- Literal strings
- Regular expressions
- Multiple patterns per run

## Command Reference

### Default Mode: Redaction

```bash
redactor [OPTIONS] --input <FILE> --output <FILE>

Options:
  -i, --input <FILE>       Input PDF file
  -o, --output <FILE>      Output PDF file
  -p, --pattern <TEXT>     Pattern to redact (repeatable)
      --phones             Redact phone numbers
      --verizon            Redact Verizon account + phones + call details
      --all                Redact all text
  -v, --verbose            Verbose output
```

### Extract Subcommand

```bash
redactor extract --input <FILE> [--output <FILE>]

Options:
  -i, --input <FILE>       Input PDF file
  -o, --output <FILE>      Output text file (stdout if omitted)
```

## Examples

### Expense Report Workflow

```bash
# 1. Download your Verizon bill (e.g., January-2026.pdf)
# 2. Redact sensitive information
redactor --input January-2026.pdf --output January-2026-redacted.pdf --verizon

# 3. (Optional) Verify redaction by extracting text
redactor extract --input January-2026-redacted.pdf
# Your account number and phone numbers should NOT appear in the output

# 4. Upload January-2026-redacted.pdf to Concur/Expensify
```

### Redact Multiple Pattern Types

```bash
redactor \
  --input sensitive.pdf \
  --output clean.pdf \
  --phones \
  --pattern "SSN: [0-9-]+" \
  --pattern "CONFIDENTIAL"
```

### Library: Custom Redaction Strategy

```rust
use redactor::{RedactionService, RedactionTarget, SecureRedactionStrategy};

let service = RedactionService::new(
    SecureRedactionStrategy::new()
        .with_verbose(true)
        .with_max_hits(500)
);

service.redact(input, output, &targets)?;
```

### Library: Pattern Matching

```rust
use redactor::domain::{PhoneNumberMatcher, PatternMatcher};

let matcher = PhoneNumberMatcher::new();
let phones = matcher.extract_all("Call (555) 234-5678 or 555-987-6543");
// phones: ["(555) 234-5678", "555-987-6543"]
```

## Performance

- **Unit tests**: <0.1s (15+ tests)
- **Integration tests**: ~0.5s (10+ tests)
- **Full test suite**: <2s (30+ tests)
- **Redaction**: ~50-80ms per page (typical)

## Architecture

```
redactor/
├── src/
│   ├── domain/          # Business logic (phone, account detection)
│   ├── redaction/       # Redaction strategies (secure, visual)
│   ├── error.rs         # Custom error types
│   ├── lib.rs           # Library API
│   └── main.rs          # CLI application
└── tests/
    ├── common/          # Shared test utilities
    ├── unit/            # Fast unit tests
    ├── integration_test.rs
    └── cli_integration_test.rs
```

## Development

### Prerequisites

- Rust 1.70+
- MuPDF development libraries

### Building

```bash
git clone https://github.com/yourusername/redactor
cd redactor
cargo build --release
```

### Testing

```bash
# All tests
cargo test

# Specific test suite
cargo test --test integration_test

# With output
cargo test -- --nocapture
```

### Linting

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Limitations

- Requires MuPDF system libraries
- Best results with standard PDF fonts
- Complex annotations may require additional handling
- Scanned PDFs (images) require OCR preprocessing

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgments

- [MuPDF](https://mupdf.com/) for PDF processing
- [lopdf](https://github.com/J-F-Liu/lopdf) for PDF manipulation
- [pdf-extract](https://github.com/jrmuizel/pdf-extract) for text extraction

## Security Notice

This tool is designed for legitimate redaction purposes. Users are responsible for:
- Verifying redaction completeness
- Complying with applicable laws and regulations
- Testing output before distribution
- Understanding PDF structure limitations

Always verify redacted PDFs before sharing sensitive documents.
