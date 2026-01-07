# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-07

### Added
- **Verizon Call Details Redaction**: Automatically redact time, origination, and destination columns from call detail tables in Verizon bills
- `VerizonCallDetailsMatcher` domain matcher for detecting and extracting call detail information
- `VerizonCallDetails` redaction target for comprehensive bill privacy
- Dynamic call detail table detection across all pages (not just specific page numbers)
- Comprehensive integration tests for call detail redaction with physical verification
- End-to-end test using real Verizon bill format

### Changed
- `--verizon` flag now includes call detail redaction in addition to account numbers and phone numbers
- Updated `SecureRedactionStrategy` to handle call detail column patterns
- Improved pattern matching for location-based data (city, state format)

### Fixed
- Clippy warnings: replaced `map_or(false, ...)` with `is_some_and(...)`
- Clippy warnings: replaced `len() > 0` and `len() >= 1` with `!is_empty()`
- Clippy warnings: changed `&PathBuf` parameters to `&Path` in legacy API functions

### Security
- All redactions use MuPDF's secure physical removal - no data remains in PDF structure
- Verified that 900+ instances can be redacted from multi-page bills
- Enhanced privacy protection for expense report submissions

## [0.1.0] - 2025-12-XX

### Added
- Initial release with secure PDF redaction using MuPDF
- Phone number detection and redaction (NANP format)
- Verizon account number detection (9-5 format)
- CLI tool with `redact` and `extract` commands
- Library API with `RedactionService` and strategy pattern
- Type3 font support via MuPDF integration
- Comprehensive test suite with unit and integration tests
- Pattern matching for literal strings and custom patterns
- Legacy API compatibility layer

### Features
- Secure physical text removal (not just visual overlay)
- Multiple redaction targets per operation
- Verbose output mode for debugging
- Text extraction for verification
- Support for complex PDF encodings

[0.2.0]: https://github.com/yourusername/redactor/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/redactor/releases/tag/v0.1.0
