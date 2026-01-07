# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
