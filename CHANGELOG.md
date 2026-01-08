# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-08

### Added
- **Full Regex Pattern Support**: Complete regular expression support for custom redaction patterns
  - Redact SSNs, emails, IP addresses, URLs, credit cards, and any custom patterns
  - 22+ comprehensive integration tests for regex patterns
  - Pattern validation with clear error messages
  - Case-insensitive matching support with `(?i)` flag
  - Combine regex patterns with built-in detectors (phones, Verizon accounts)
- New `RedactionTarget::Regex(String)` variant for regex-based redaction
- Comprehensive regex pattern guide in README with common examples
- **CI/CD Infrastructure**:
  - GitHub Actions workflow for automated testing
  - Code coverage tracking with llvm-cov
  - Mutation testing with cargo-mutants
  - Automated coverage reports and badges
- Test infrastructure improvements:
  - Updated `tests/common/pdf_helpers.rs` to use modern API
  - Updated `tests/common/assertions.rs` to use modern API
  - New `tests/regex_patterns_test.rs` with 22 integration tests
  - New `tests/coverage_gaps_test.rs` for coverage analysis
  - New `tests/error_tests.rs` for error handling
  - New `tests/property_based_tests.rs` for property-based testing
  - New `tests/secure_redaction_edge_cases_test.rs` for edge cases
- `mutants.toml` configuration for mutation testing

### Changed
- **BREAKING**: Removed legacy API functions (use `RedactionService` instead):
  - Removed `redact_phone_numbers_in_pdf()`
  - Removed `extract_text_from_pdf()` (use `RedactionService::extract_text()`)
  - Removed `find_verizon_account_number()`
  - Removed `generate_account_patterns()`
  - Removed `get_phone_number_pattern()`
  - Removed `redact_verizon_account_in_pdf()`
  - Removed `extract_text_from_page_content()`
- Removed legacy integration tests that used old API
- Cleaned up test suite to use only modern API
- Updated documentation to reflect new API patterns

### Removed
- **BREAKING**: Removed `RedactionTarget::AllText` variant
- Removed `--all` CLI flag (use `RedactionTarget::Regex(r".+")` for full-page redaction)
- Removed legacy test files:
  - `tests/integration_test.rs` (1500+ lines of legacy tests)
  - `tests/generate_pdfs.rs` (legacy PDF generators)
  - `tests/unit/pattern_tests.rs` (legacy pattern tests)

### Fixed
- Improved error messages for invalid regex patterns
- Better handling of PDF text extraction edge cases
- Fixed word boundary issues in regex patterns (documented in README)

### Documentation
- Merged regex patterns feature documentation into main README
- Added "Regex Pattern Guide" section with common examples
- Updated all examples to use modern API
- Added performance notes for regex compilation and text extraction
- Documented PDF text extraction considerations (word boundaries, layout)

## [0.2.1] - 2026-01-07

### Fixed
- Fixed MuPDF thread-safety race condition in test suite that caused intermittent crashes
- Added global mutex to serialize MuPDF operations across parallel tests
- Tests now run reliably without `fz_ft_lock` assertion failures

## [0.2.0] - 2026-01-07

### Added
- Call detail redaction for Verizon bills (time, origination, destination columns)
- `VerizonCallDetailsMatcher` for detecting and extracting call metadata
- `RedactionTarget::VerizonCallDetails` for call detail table redaction
- Comprehensive integration tests for call detail redaction
- End-to-end verification of physical redaction in PDFs

### Changed
- **BREAKING**: Simplified CLI - redaction is now the default command
- **BREAKING**: Removed `redact` subcommand; use flags directly (e.g., `redactor --input file.pdf --verizon`)
- Updated `--verizon` flag to include call detail redaction
- Improved documentation with usage examples and limitations

### Fixed
- Fixed clippy warnings for better code quality
- Changed `&PathBuf` to `&Path` in function signatures
- Removed regex creation in loops for better performance
- Fixed `len() > 0` to `!is_empty()` patterns

### Security
- All test data uses synthetic information only
- No real user data in tests, examples, or documentation

## [0.1.0] - Initial Release

### Added
- Phone number redaction with US format support
- Verizon account number redaction
- Type3 font support via MuPDF
- CLI with `extract` and `redact` commands
- Pattern-based and secure redaction strategies
- Comprehensive test suite
