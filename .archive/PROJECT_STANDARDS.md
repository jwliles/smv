# SMV Project Standards

This document defines the technical standards, architecture, and coding guidelines for the SMV project. It serves as a reference for maintaining consistency across the codebase.

## Code Architecture

### Overall Structure

The codebase is organized into modules with clear separation of concerns:

```
src/
├── bin/         # Binary entry points
├── cli/         # Command-line interface code
├── core/        # Core functionality and file operations
├── transformers/ # Filename transformation logic
├── history.rs   # History and undo functionality
├── repl.rs      # Interactive REPL implementation
└── lib.rs       # Library exports and documentation
```

### Key Components

- **Transformers**: All file name transformation functionality
- **Core**: File operations, path handling, and utility functions
- **CLI**: Command-line argument parsing and execution
- **History**: Operation tracking and undo capabilities
- **REPL**: Interactive shell for user operations

### Design Principles

1. **Modularity**: Components should be well-encapsulated with clear interfaces
2. **Single Responsibility**: Each module should have one primary responsibility
3. **Testability**: Code should be structured to enable comprehensive testing
4. **Error Handling**: Use Result types for error propagation
5. **Documentation**: All public APIs must be documented

## Coding Standards

### Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use idiomatic Rust constructs (`Option`, `Result`, iterators)
- Prefer immutable variables (`let` vs `let mut`)
- Use pattern matching over if-let when applicable
- Follow standard Rust naming conventions:
  - `snake_case` for variables, functions, and modules
  - `PascalCase` for types, traits, and enums
  - `SCREAMING_SNAKE_CASE` for constants

### Function Design

- Keep functions focused and concise
- Use descriptive function names that indicate what they do
- Document parameters and return values
- Limit function complexity (cyclomatic complexity)

### Error Handling

- Use proper error types and avoid panicking
- Provide meaningful error messages
- Use context in errors (e.g., filename in file operation errors)
- Consider using the `thiserror` crate for custom error types

### Documentation

- All public APIs must have doc comments
- Include examples for non-trivial functionality
- Document potential errors and edge cases
- Keep code comments current with implementation

## Testing Philosophy

### Unit Testing

- Test small units of functionality in isolation
- Focus on testing public interfaces
- Use mock objects for external dependencies

### Integration Testing

- Test multiple components working together
- Test the CLI interface with actual command invocations
- Verify file operations with temporary files

### Property-Based Testing

- Consider using property-based testing for transformation functions
- Ensure that transformations maintain expected invariants

### Test Coverage

- Aim for >80% test coverage
- Prioritize testing complex logic and edge cases
- Include regression tests for fixed bugs

## Versioning and Compatibility

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backward compatible manner
- **PATCH** version for backward compatible bug fixes

### API Stability

- Public API changes must be carefully considered
- Document deprecated features before removal
- Provide migration paths for breaking changes

## Performance Considerations

- Consider performance implications when processing large numbers of files
- Use benchmarks to track performance metrics
- Prefer efficient algorithms and data structures
- Avoid unnecessary memory allocations

## Security Considerations

- Handle file paths safely to prevent path traversal vulnerabilities
- Validate user input before processing
- Be cautious with file system operations on potentially malicious input
- Consider permissions and ownership when modifying files

## Dependencies

- Choose dependencies carefully, considering maintenance status
- Keep external dependencies to a necessary minimum
- Regularly update dependencies for security fixes
- Document the purpose of each dependency

## File Format Guidelines

### Rust Source Files (.rs)

- Start with a file-level doc comment explaining purpose
- Group imports logically
- Place `use` statements at the top of the file
- Include tests in a `mod tests` submodule

### Documentation Files (.md)

- Use consistent formatting and hierarchy
- Include examples where appropriate
- Keep content updated with implementation
- Use links for cross-references

## Configuration Management

- Store user configuration in standard locations following XDG spec
- Use appropriate serialization formats (TOML, JSON)
- Provide sensible defaults
- Validate configuration on load

## Continuous Integration

- Run tests on multiple platforms (Linux, macOS, Windows)
- Check formatting with rustfmt
- Run clippy for linting
- Verify documentation builds correctly