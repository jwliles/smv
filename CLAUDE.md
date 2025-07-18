# Claude Development Guidelines for SMV

## Contributing Standards
When working on this project, always follow the guidelines in CONTRIBUTING.md:

### Pre-Code Checklist
- Read and understand the task requirements
- Use TodoWrite tool to plan complex tasks
- Follow GitFlow branching strategy (feature/*, bugfix/*, etc.)

### Code Quality Standards
- Run `cargo fmt` after any code changes
- Run `cargo clippy -- -D warnings` and fix all warnings
- Run `cargo test` to ensure all tests pass
- Write comprehensive tests for new functionality:
  - **CLI behavior: 100% test coverage required**
  - **Logic code: 80% minimum coverage**
  - Use `assert_cmd` for CLI testing

### Documentation Requirements
- Update README.md for user-facing changes
- Add/update doc comments (///) for public APIs
- Include usage examples for complex functions
- Update CHANGELOG.md for releases

### Commit Standards
Use conventional commits:
```
type(scope): short description

Types: feat, fix, docs, style, refactor, test, chore
Example: feat(parser): add support for nested syntax blocks
```

### Testing Tools
- Use `assert_cmd::Command` for CLI tests
- Run coverage: `cargo tarpaulin --out Html` or `cargo llvm-cov --workspace --html`
- Test all CLI flags, subcommands, and options

### Build Verification
Before completing any task:
1. `cargo build` - must succeed without errors
2. `cargo fmt` - format all code
3. `cargo clippy` - fix all warnings
4. `cargo test` - all tests must pass
5. Verify documentation is updated

## Project-Specific Notes
- SMV is part of the Canopy (CNP) suite
- Focus on POSIX compatibility and modern file operations
- Maintain backward compatibility when adding features
- Follow existing patterns in codebase architecture