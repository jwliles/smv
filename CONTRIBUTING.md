# Contributing to AFN Projects

This document outlines the contribution guidelines for all AFN projects. Following these standards ensures code quality, maintainability, and project consistency.

## Gitflow Workflow

We follow the Gitflow workflow for all AFN projects:

1. **Main Branch**: Always contains production-ready code
   - Never commit directly to main
   - Protected by requiring pull request reviews

2. **Develop Branch**: Integration branch for features
   - All feature branches merge into develop
   - Should be in a working state at all times

3. **Feature Branches**: Where new features are developed
   - Branch from: `develop`
   - Name format: `feature/short-description`
   - Merge back into: `develop` (via pull request)

4. **Release Branches**: Prepare for production releases
   - Branch from: `develop`
   - Name format: `release/x.y.z`
   - Merge back into: `main` AND `develop` (when finalized)
   - Only bug fixes, documentation and release-oriented tasks

5. **Hotfix Branches**: Emergency fixes for production
   - Branch from: `main`
   - Name format: `hotfix/x.y.z` or `hotfix/issue-description`
   - Merge back into: `main` AND `develop`

6. **Pull Requests**:
   - Required for merging into `main` and `develop`
   - Must pass all checks (tests, linting)
   - Require at least one code review approval

## Testing Requirements

All code contributions must include tests:

1. **Test Coverage**:
   - Write tests for all new functionality
   - Maintain or improve the project's test coverage percentage
   - No pull requests will be approved with failing tests

2. **Test Types**:
   - **Unit Tests**: Test individual functions and methods
   - **Integration Tests**: Test interactions between components
   - **End-to-End Tests**: Test complete user workflows (when applicable)

3. **Test-Driven Development**:
   - Write tests before implementing features when possible
   - Tests should verify both expected functionality and edge cases

4. **Running Tests**:
   - All tests must pass locally before submitting a PR
   - CI pipeline will verify tests on each PR

## Documentation Standards

Code must be well-documented:

1. **Code Comments**:
   - Add descriptive comments for non-obvious code
   - Document public APIs thoroughly
   - Explain complex algorithms or business logic
   - Use standard documentation formats (e.g., rustdoc for Rust projects)

2. **Function Documentation**:
   - Document parameters, return values, and errors
   - Include examples where appropriate
   - Explain side effects or preconditions

3. **Module/Package Documentation**:
   - Include module-level documentation explaining purpose and contents
   - Document usage patterns and examples

4. **Project Documentation**:
   - Maintain up-to-date README.md with:
     - Project overview
     - Installation instructions
     - Basic usage examples
   - Include CHANGELOG.md for tracking version changes

5. **Documentation Updates**:
   - Update documentation when changing existing code
   - Documentation changes should be part of the same PR as code changes

## Code Style and Quality

1. **Linting and Formatting**:
   - Follow project-specific linting rules
   - Use automated formatting tools
   - PRs must pass all linting checks

2. **Code Reviews**:
   - All code must be reviewed before merging
   - Address review comments promptly
   - Reviewers should check for adherence to these guidelines

3. **Commit Messages**:
   - Write clear, descriptive commit messages
   - Format: `type(scope): short description`
   - Types: feat, fix, docs, style, refactor, test, chore
   - Reference issue numbers when applicable

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Set up the development environment following README instructions
4. Create a feature branch from `develop`
5. Make your changes, adhering to guidelines
6. Write or update tests
7. Update documentation
8. Submit a pull request

Thank you for contributing to AFN projects!