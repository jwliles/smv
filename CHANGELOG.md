# Changelog

All notable changes to SMV will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup
- Standard `mv` command functionality
- File name transformations:
  - clean: Remove special characters and normalize spaces
  - snake_case: Convert to snake_case format
  - kebab-case: Convert to kebab-case format
  - Title Case: Convert to Title Case format
  - camelCase: Convert to camelCase format
  - PascalCase: Convert to PascalCase format
  - lowercase: Convert to lowercase
  - UPPERCASE: Convert to uppercase
- Preview mode to see changes before applying
- Undo capability to revert the last operation
- Interactive REPL mode with command processing
- Recursive directory operation support
- File filtering by extension or pattern
- History management for tracking operations
- Automatic file backups for safety
- Conflict detection to prevent data loss
- Colored terminal output for better readability
- Better project organization with modules
- Trait-based transformation system for extensibility

### Changed
- Switched from derive-based clap API to builder API to avoid proc-macro issues
- Updated dependency versions for better compatibility
- Modified project structure to follow Rust best practices
- Enhanced documentation with detailed build instructions

### Fixed
- Build issues with proc-macro compilation on certain platforms
- Documentation inconsistencies and outdated information

## [0.1.0] - 2024-03-14
- Initial development version

[unreleased]: https://github.com/jwliles/smv/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jwliles/smv/releases/tag/v0.1.0