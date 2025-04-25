# Contributing to SMV

Thank you for your interest in contributing to SMV (Smart Move)! This document guides you through the contribution process and outlines our project standards.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up the development environment**:
   ```bash
   cargo build
   cargo test
   ```

## Contribution Workflow

### 1. Choose an Issue

- Pick an issue from the [issue tracker](https://github.com/jwliles/smv/issues)
- Comment on the issue to let others know you're working on it
- If there's no issue for your contribution, create one first to discuss the change

### 2. Create a Branch

We follow a Gitflow-based workflow:

```bash
# Start from latest dev branch
git checkout dev
git pull origin dev

# Create a feature branch
git checkout -b feature/descriptive-name

# For bug fixes
git checkout -b fix/issue-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Adding or improving tests
- `chore/` - Maintenance tasks

### 3. Make Your Changes

- Write tests for new functionality
- Keep commits focused and atomic
- Use [Conventional Commits](https://www.conventionalcommits.org/) format for commit messages:
  ```
  <type>(<scope>): <description>
  ```
  Examples:
  ```
  feat(transforms): Add pipeline transformation support
  fix(repl): Fix crash when applying transformation to empty filename
  docs(readme): Update installation instructions
  ```

### 4. Submit a Pull Request

1. Push your branch to your fork
2. Create a pull request to the `dev` branch of the main repository
3. Fill in the PR template with details about your changes
4. Link any related issues using GitHub keywords (Fixes #123, Closes #456)

## Project Standards

### Code Architecture

The codebase is organized into modules with clear separation of concerns:

```
src/
├── bin/          # Binary entry points
├── cli/          # Command-line interface code
├── core/         # Core functionality and file operations
├── transformers/ # Filename transformation logic
├── history.rs    # History and undo functionality
├── repl.rs       # Interactive REPL implementation
└── lib.rs        # Library exports and documentation
```

### Coding Standards

- Follow standard Rust conventions and idioms
- All public APIs must have doc comments
- Include examples for non-trivial functionality
- Use proper error handling with meaningful messages

### Formatting and Linting

- Format code with rustfmt: `cargo fmt`
- Run clippy to catch common issues: `cargo clippy -- -D warnings`
- Run tests before submitting: `cargo test`

### Testing Requirements

- Unit tests for transformation functions and core components
- Integration tests for CLI functionality
- Tests should cover both success and error paths

### Commit Standards

We follow Conventional Commits with these types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code changes that don't fix bugs or add features
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates, etc.

### Documentation

When updating code that affects user-facing functionality:
- Update README.md with examples if needed
- Update man page for new command options
- Include inline documentation for API changes

## Need Help?

If you need help with your contribution, you can:
- Ask questions in the related issue
- Reach out to the maintainers

Thank you for contributing to SMV!