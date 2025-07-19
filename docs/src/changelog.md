# Changelog

All notable changes to SMV will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2025-07-19

### Added
- Prefix removal functionality using `CHANGE "prefix" INTO ""` syntax
- CNP grammar compliant prefix removal that integrates with existing substring replacement
- Support for removing common file prefixes (e.g., `IMG_`, `DSC_`, etc.)
- Comprehensive CLI tests for prefix removal scenarios
- Documentation examples showing prefix removal usage

### Changed
- Migrated from Rust Edition 2021 to 2024
- Enhanced `CHANGE` command to detect empty replacement string for prefix removal
- Updated help text and documentation to include prefix removal examples
- Improved error handling in CLI tests

### Fixed
- Fixed clippy warning by using `strip_prefix` instead of manual string slicing
- Fixed non-exhaustive pattern match for `RemovePrefix` variant in preview view
- Resolved CLI test failures for TUI and force flag scenarios

### Technical
- Added `RemovePrefix(String)` variant to `TransformType` enum
- Implemented `remove_prefix` function using Rust's `strip_prefix` method
- Extended CLI test suite with 11 comprehensive test cases
- Updated Cargo.toml edition field to "2024"

## [0.4.0] - Previous Release

### Added
- Complete MVP features
- CNP grammar support
- File transformation capabilities
- Interactive modes (REPL, TUI)
- Comprehensive command structure

### Removed
- Dependency on dsc-rs crate