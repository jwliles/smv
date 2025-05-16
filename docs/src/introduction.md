# Introduction to SMV

SMV (Smart Move) is a powerful, Rust-based drop-in replacement for the standard Unix `mv` command with intelligent filename transformation capabilities.

## Why SMV?

SMV extends the basic functionality of the standard `mv` command with additional features for intelligent file renaming and organization. It aims to solve common file management challenges:

- Batch renaming files with consistent patterns
- Converting between different filename conventions (camelCase, snake_case, etc.)
- Previewing changes before applying them
- Safely reverting accidental changes

## Key Features

- **Drop-in replacement** for the standard Unix `mv` command
- **Interactive REPL Interface** with command history and tab completion
- **Smart File Renaming** with multiple transformation strategies
- **Preview Mode** - See changes before they're applied
- **Batch Processing** - Apply transformations to multiple files at once
- **Undo Functionality** - Safely revert changes
- **Safety Features** - File backups and conflict detection
- **Terminal UI** - Visual file management interface (Coming Soon)

## Transformation Capabilities

SMV provides several transformation strategies for filenames:

- Convert to snake_case (`document-name.pdf` → `document_name.pdf`)
- Convert to kebab-case (`document_name.pdf` → `document-name.pdf`) 
- Convert to Title Case (`document_name.pdf` → `Document Name.pdf`)
- Convert to camelCase (`document_name.pdf` → `documentName.pdf`)
- Convert to PascalCase (`document_name.pdf` → `DocumentName.pdf`)
- Convert to lowercase or UPPERCASE
- Clean up spaces and special characters

## Interfaces

SMV offers multiple interfaces for different use cases:

- **Command-line mode**: Traditional command-line usage like the standard `mv`
- **Interactive REPL**: Command-line shell with history and suggestions
- **Terminal UI**: File explorer with Vim-style navigation (Coming Soon)

## System Requirements

- Rust (Minimum supported version: 1.85.0)
- GNU/Linux or other free operating system
- Standard system libraries

## License

SMV is released under the MIT License. See the [LICENSE](https://github.com/jwliles/smv/blob/main/LICENSE) file for details.
