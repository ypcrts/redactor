# redactor

[![Crates.io](https://img.shields.io/crates/v/redactor.svg)](https://crates.io/crates/redactor)
[![Documentation](https://docs.rs/redactor/badge.svg)](https://docs.rs/redactor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/ypcrts/redactor/actions/workflows/rust.yml/badge.svg)](https://github.com/ypcrts/redactor/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/ypcrts/redactor/graph/badge.svg)](https://codecov.io/gh/ypcrts/redactor)

A PDF redaction library and CLI tool with secure text removal using MuPDF. Redacts Verizon bills so you can expense them without leaking your call metadata.

## Use Case

Originally built for redacting **Verizon phone bills** before submitting them to employer expense reimbursement systems like **Concur** or **Expensify**. 

When submitting phone bills for work expense reimbursement, you typically need to:
- ✅ Keep the billing amounts visible (for verification)
- ❌ Remove your account number (privacy/security)
- ❌ Remove personal phone numbers (privacy)
- ❌ Remove call detail information (times, locations, destinations)
- ❌ Remove other personal contact information

This tool ensures your sensitive information is **physically removed** from the PDF (not just blacked out), so it cannot be extracted by the expense system or anyone who views the document.

Perfect for freelancers, remote workers, and employees who need to submit redacted bills for work expenses while maintaining privacy.

## Features

- **Secure Redaction**: Physically removes text from PDFs (not just visual overlay)
- **Type3 Font Support**: Handles complex PDF encodings via MuPDF
- **Phone Number Detection**: Automatic NANP phone number redaction
- **Verizon Account Numbers**: Specialized detection for 9-5 format accounts
- **Call Detail Redaction**: Automatically redacts time, origination, and destination columns
- **Pattern Matching**: Literal strings and powerful regex patterns
- **Regex Support**: Full regular expression support for custom patterns (SSNs, emails, IPs, URLs, etc.)
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

### Built-in Detectors

**Phone Numbers (NANP)**
- `(555) 123-4567`
- `555-987-6543`
- `555.111.2222`
- `+1 555 234 5678`

**Verizon Accounts**
- `123456789-00001` (9-5 format)
- `12345678900001` (14 digits)
- Context-aware detection

### Custom Patterns

**Literal Strings**
- Exact text matching
- Case-sensitive by default
- Multiple patterns supported

**Regular Expressions**

Full regex support for custom pattern matching:

```rust
// Social Security Numbers
RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())

// Email Addresses
RedactionTarget::Regex(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}".to_string())

// IP Addresses
RedactionTarget::Regex(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}".to_string())

// URLs
RedactionTarget::Regex(r"https?://[^\s]+".to_string())

// Credit Cards
RedactionTarget::Regex(r"\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{3,4}".to_string())

// Custom ID formats
RedactionTarget::Regex(r"[A-Z]{2}\d{6}".to_string())

// Case-insensitive patterns
RedactionTarget::Regex(r"(?i)CONFIDENTIAL".to_string())
```

**Features:**
- Full Rust regex syntax support
- Pattern validation with clear error messages
- Case-insensitive matching (`(?i)` flag)
- Combine multiple regex patterns
- Mix regex with built-in detectors

**Important Notes:**
- Word boundaries (`\b`) may not work reliably due to PDF text extraction
- Use patterns without word boundaries for best results
- Example: Use `\d{3}-\d{2}-\d{4}` instead of `\b\d{3}-\d{2}-\d{4}\b`

## Regex Pattern Guide

The library provides full regular expression support for custom pattern matching, powered by Rust's regex crate. Patterns are validated before processing.

### Common Pattern Examples

```rust
use redactor::{RedactionService, RedactionTarget};

let service = RedactionService::with_secure_strategy();

// Social Security Numbers (XXX-XX-XXXX)
RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())

// Email Addresses
RedactionTarget::Regex(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}".to_string())

// Phone Numbers (custom format)
RedactionTarget::Regex(r"\d{3}[-.]?\d{3}[-.]?\d{4}".to_string())

// IP Addresses
RedactionTarget::Regex(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}".to_string())

// Credit Card Numbers
RedactionTarget::Regex(r"\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{3,4}".to_string())

// URLs (http/https)
RedactionTarget::Regex(r"https?://[^\s]+".to_string())

// Currency Amounts
RedactionTarget::Regex(r"\$[\d,]+\.\d{2}".to_string())

// Dates (YYYY-MM-DD)
RedactionTarget::Regex(r"\d{4}-\d{2}-\d{2}".to_string())

// Custom IDs (e.g., AB123456)
RedactionTarget::Regex(r"[A-Z]{2}\d{6}".to_string())

// Case-insensitive matching
RedactionTarget::Regex(r"(?i)CONFIDENTIAL".to_string())
```

### Important Considerations

**Word Boundaries**

PDF text extraction often concatenates text without spaces, making `\b` word boundaries unreliable:

- ❌ May not work: `\b\d{3}-\d{2}-\d{4}\b`
- ✅ Better: `\d{3}-\d{2}-\d{4}`

**Error Handling**
Invalid regex patterns return clear error messages:

```rust
let result = service.redact(
    input,
    output,
    &[RedactionTarget::Regex(r"[invalid(".to_string())]
);
// Error: "Invalid regex pattern: ..."
```

**Performance**

- Regex compilation happens once per pattern
- Text extraction occurs once per regex target
- Efficient pattern matching using Rust's optimized regex engine
- Respects `max_hits` limit to prevent performance issues

### Testing Regex Patterns

The library includes 22+ integration tests covering basic pattern matching, multiple patterns, case-insensitive patterns, invalid patterns (error handling), no matches (graceful handling), combining regex with built-in detectors, and edge cases.

Run regex pattern tests:
```bash
cargo test --test regex_patterns_test
```

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
# Combine built-in detectors with literal patterns
redactor \
  --input sensitive.pdf \
  --output clean.pdf \
  --phones \
  --pattern "SSN: [0-9-]+" \
  --pattern "CONFIDENTIAL"
```

### Redact with Regex Patterns

```bash
# Redact email addresses
redactor \
  --input document.pdf \
  --output redacted.pdf \
  --pattern '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}'

# Redact Social Security Numbers
redactor \
  --input document.pdf \
  --output redacted.pdf \
  --pattern '\d{3}-\d{2}-\d{4}'

# Combine regex with built-in detectors
redactor \
  --input bill.pdf \
  --output clean.pdf \
  --verizon \
  --pattern '\d{3}-\d{2}-\d{4}'
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

### Library: Regex Pattern Redaction

```rust
use redactor::{RedactionService, RedactionTarget};
use std::path::Path;

let service = RedactionService::with_secure_strategy();

// Redact Social Security Numbers
service.redact(
    Path::new("input.pdf"),
    Path::new("output.pdf"),
    &[RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string())]
)?;

// Multiple regex patterns
service.redact(
    Path::new("input.pdf"),
    Path::new("output.pdf"),
    &[
        RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string()), // SSN
        RedactionTarget::Regex(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}".to_string()), // Email
        RedactionTarget::Regex(r"\$[\d,]+\.\d{2}".to_string()), // Currency
    ]
)?;

// Combine regex with built-in detectors
service.redact(
    Path::new("input.pdf"),
    Path::new("output.pdf"),
    &[
        RedactionTarget::PhoneNumbers,
        RedactionTarget::VerizonAccount,
        RedactionTarget::Regex(r"\d{3}-\d{2}-\d{4}".to_string()),
    ]
)?;
```

## Performance

- Unit tests: <0.1s (15+ tests)
- Integration tests: ~0.5s (10+ tests including regex patterns)
- Full test suite: <2s (50+ tests)
- Redaction: ~50-80ms per page (typical)
- Regex compilation: <1ms per pattern (cached during operation)

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
git clone https://github.com/ypcrts/redactor
cd redactor
cargo build --release
```

### Testing

The test suite is organized into unit, integration, and end-to-end layers, each serving a distinct purpose.

#### Running Tests

```bash
# All tests
cargo test

# Unit tests only (fastest)
cargo test --lib

# Integration tests
cargo test --test integration_test

# CLI/E2E tests
cargo test --test cli_integration_test

# Regex pattern tests
cargo test --test regex_patterns_test

# Specific test
cargo test test_phone_normalization

# With output
cargo test -- --nocapture

# Advanced options
cargo test --release                    # Run in release mode
cargo test -- --test-threads=1          # Single-threaded (for debugging)
cargo test -- --show-output             # Show all output
cargo test -- --ignored                 # Run ignored tests
```

#### Test Structure

```
tests/
├── common/                    # Shared utilities
│   ├── assertions.rs         # Custom assertions
│   ├── fixtures.rs           # Test PDF builders
│   └── pdf_helpers.rs        # PDF inspection utilities
├── unit/                      # Fast unit tests
│   ├── domain_tests.rs       # Business logic tests
│   └── pattern_tests.rs      # Regex/pattern tests
├── integration_test.rs        # Integration tests
└── cli_integration_test.rs    # CLI/E2E tests
```

#### Writing Tests

**Unit Test Example**

```rust
// tests/unit/domain_tests.rs
use redactor::domain::PhoneNumberMatcher;

#[test]
fn test_phone_normalization() {
    let matcher = PhoneNumberMatcher::new();
    let result = matcher.normalize("(555) 234-5678");
    assert_eq!(result, Some("5552345678".to_string()));
}
```

**Integration Test Example**

```rust
// tests/integration_test.rs
use common::*;
use redactor::{RedactionService, RedactionTarget};
use tempfile::TempDir;

#[test]
fn test_phone_redaction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input.pdf");
    let output = temp_dir.path().join("output.pdf");
    
    // Use builder pattern for test data
    TestPdfBuilder::new()
        .with_phone("(555) 234-5678")
        .with_content("Contact information")
        .build(&input)?;
    
    // Execute redaction
    let service = RedactionService::with_secure_strategy();
    service.redact(&input, &output, &[RedactionTarget::PhoneNumbers])?;
    
    // Use custom assertions
    assert_valid_pdf(&output);
    assert_redacted(&output, "555");
    assert_preserved(&output, "Contact");
    
    Ok(())
}
```

#### Test Utilities

The test suite includes shared utilities in `tests/common/`:

**Custom Assertions**

```rust
use common::*;

assert_redacted(pdf_path, "sensitive-data");
assert_preserved(pdf_path, "normal-content");
assert_valid_pdf(pdf_path);
assert_all_redacted(pdf_path, &["secret1", "secret2"]);
```

**Test Fixtures**

```rust
use common::*;

// Builder pattern for test PDFs
TestPdfBuilder::new()
    .with_title("Test Document")
    .with_verizon_account("123456789-00001")
    .with_phone("(555) 234-5678")
    .with_content("Additional content")
    .build(path)?;
```

**PDF Helpers**

```rust
use common::*;

let text = extract_text(pdf_path)?;
let count = count_pattern_in_pdf(pdf_path, "pattern")?;
let phone_count = count_phones_in_pdf(pdf_path)?;
let has_pattern = pdf_contains_any(pdf_path, &["p1", "p2"])?;
```

#### Test Coverage

The suite covers:
- Phone number detection (NANP formats, edge cases)
- Verizon account detection (9-5 format, 14-digit format)
- Pattern matching (literal strings, regex)
- Secure redaction (physical text removal verification)
- Combined redaction (multiple targets simultaneously)
- Error handling (missing files, invalid PDFs, corrupted PDFs)
- CLI interface (end-to-end workflows)

#### Troubleshooting

**Tests fail to compile:**
```bash
cargo clean
cargo build --tests
```

**Test PDFs not found:**
```bash
cargo test --test generate_pdfs
```

**Slow test execution:**
```bash
cargo test --release
cargo test unit::  # Run only unit tests
```

### Code Coverage

Code coverage is tracked using `cargo-llvm-cov`. Reports are automatically generated on every PR and push to main.

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report (terminal output)
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Generate LCOV report (for CI/Codecov)
cargo llvm-cov --lcov --output-path lcov.info
```

**Alternative: Tarpaulin**

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --all-features --workspace --out html

# View report
open tarpaulin-report.html
```

**Coverage Status:**
- 199+ passing tests across unit, integration, property-based, CLI, and edge case categories
- Estimated coverage: ~95%

### Mutation Testing

Mutation testing complements code coverage by validating test quality. It systematically introduces small bugs (mutations) into the source code and verifies that tests detect them. While coverage shows *what code is executed*, mutation testing reveals *whether tests actually validate behavior*.

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing (5-15 minutes)
cargo mutants --all

# Generate HTML report
cargo mutants --all --html
open mutants-out/html/index.html
```

**How it works:**
1. Mutations are introduced (operator changes, logic flips, return value modifications)
2. Tests run against each mutation
3. Results are classified: killed (good), survived (needs attention), timeout, or build failure
4. Mutation score calculated: `killed / (killed + survived) × 100%`

**Target metrics:**
- Mutation score >85% indicates strong test quality
- Surviving mutations highlight areas needing stronger test coverage
- Weekly runs via GitHub Actions with reports uploaded as artifacts

**Configuration:**
Mutation testing is configured via `mutants.toml` in the project root, which excludes test code, FFI bindings, and CLI entry points from mutation.

### Benchmarks

Benchmarks measure performance of critical operations using the Criterion framework.

```bash
# Install criterion (if not already installed)
cargo install cargo-criterion

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench phone_detection

# Generate HTML report
cargo bench -- --save-baseline my-baseline
```

**Creating Benchmarks**

Create benchmarks in `benches/` directory:

```rust
// benches/redaction_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use redactor::domain::PhoneNumberMatcher;

fn benchmark_phone_detection(c: &mut Criterion) {
    let matcher = PhoneNumberMatcher::new();
    let text = "Call (555) 234-5678 for information";
    
    c.bench_function("phone_detection", |b| {
        b.iter(|| {
            matcher.extract_all(black_box(text))
        });
    });
}

criterion_group!(benches, benchmark_phone_detection);
criterion_main!(benches);
```

**Performance Targets**

| Operation | Target | Typical |
|-----------|--------|---------|
| Phone detection (small text) | <5µs | ~1-2µs |
| Account detection | <10µs | ~5µs |
| Pattern variant generation | <1µs | ~0.5µs |
| PDF text extraction (1 page) | <50ms | ~20-30ms |
| Secure redaction (1 page) | <100ms | ~50-80ms |

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

Contributions are welcome. To get started:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

### Adding Tests

When adding new functionality:

1. Determine the appropriate test layer (unit/integration/e2e)
2. Use existing utilities from `tests/common/`
3. Follow the naming convention: `test_<feature>_<scenario>`
4. Include error case testing
5. Ensure tests are isolated and deterministic

**Test Guidelines:**
- Keep unit tests fast (<1ms per test)
- Maintain isolation (no shared state between tests)
- Ensure determinism (same result every time)
- Use clear, descriptive test names
- Leverage shared utilities to avoid duplication

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgments

- [MuPDF](https://mupdf.com/) for PDF processing
- [lopdf](https://github.com/J-F-Liu/lopdf) for PDF manipulation
- [pdf-extract](https://github.com/jrmuizel/pdf-extract) for text extraction

## Security Notice

This tool is designed for legitimate redaction purposes. Users are responsible for verifying redaction completeness, complying with applicable laws and regulations, testing output before distribution, and understanding PDF structure limitations.

Always verify redacted PDFs before sharing sensitive documents.
