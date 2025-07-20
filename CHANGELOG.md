# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-01-20

### Added
- **Split functionality**: New `split TRANSFORMATION` command for camelCase/PascalCase boundary detection
  - `smv split snake` - Split camelCase/PascalCase then convert to snake_case
  - `smv split kebab` - Split camelCase/PascalCase then convert to kebab-case  
  - `smv split title` - Split camelCase/PascalCase then convert to Title Case
  - `smv split camel` - Split camelCase/PascalCase then convert to camelCase
  - `smv split pascal` - Split camelCase/PascalCase then convert to PascalCase
  - `smv split lower` - Split camelCase/PascalCase then convert to lowercase
  - `smv split upper` - Split camelCase/PascalCase then convert to UPPERCASE
  - `smv split sentence` - Split camelCase/PascalCase then convert to Sentence case
  - `smv split start` - Split camelCase/PascalCase then convert to Start Case
  - `smv split studly` - Split camelCase/PascalCase then convert to StudlyCaps
- Advanced regex-based camelCase/PascalCase boundary detection
- Comprehensive unit tests for split functionality
- CLI integration tests for split commands
- Updated help text with split examples
- Enhanced README.md with split documentation
- Updated man page with split command reference

### Examples
```bash
# Split camelCase files and convert to snake_case
smv split snake featureWishList.md
# featureWishList.md → feature_wish_list.md

# Split PascalCase files and convert to kebab-case  
smv split kebab UserSettings.json
# UserSettings.json → user-settings.json

# Preview split transformation
smv split title apiEndpoint.ts -p
# Preview: apiEndpoint.ts → ApiEndpoint.ts
```

### Technical Details
- Regex pattern `([a-z])([A-Z])|([A-Z]+)([A-Z][a-z])` for boundary detection
- Handles complex cases like `XMLDocument` → `XML Document`
- Falls back to regular transformations when no boundaries detected
- Preserves file extensions during transformation
- Full integration with existing CNP grammar and filtering system

## [0.4.2] - Previous Release
- Base functionality with standard transformations
- CNP grammar support
- Interactive and TUI modes