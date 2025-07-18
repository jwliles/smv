# Contributing Guide

Thank you for your interest in contributing! This document outlines the development workflow, style guide, and expectations for contributors across all tools in the Canopy (CNP) suite.

---

## 📌 Code of Conduct

By participating in this project, you agree to foster a respectful, inclusive, and collaborative environment.

For full behavioral expectations, please see our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Git Workflow

We follow a GitFlow-style branching strategy across all projects:

### Main Branches

* `main` – Stable production code. Only updated via release branches or hotfixes.
* `develop` – Active development. Feature branches are merged here.

### Supporting Branches

* `feature/short-description` – New features (branched from `develop`)
* `bugfix/short-description` – Bug fixes (branched from `develop`)
* `hotfix/short-description` – Emergency fixes (branched from `main`, merged into both `main` and `develop`)
* `release/x.y.z` – Prepares the next version (branched from `develop`, merged into both `main` and `develop`)

### Branch Naming Convention

Use hyphenated lowercase names:

```bash
git checkout -b feature/add-syntax-highlighting develop
```

---

## ✅ Contribution Checklist

Before opening a pull request:

1. ✅ Code builds and runs without errors
2. ✅ All tests pass locally
3. ✅ Code is formatted: `cargo fmt`
4. ✅ Linting passes: `cargo clippy`
5. ✅ You’ve written or updated tests
6. ✅ You’ve updated documentation where needed
7. ✅ Commit messages follow conventions (see below)
8. ✅ Pull request description explains the change and links relevant issues

---

## 🧺 Testing Standards

* All new code **must** include appropriate unit tests
* **All CLI flags, subcommands, and options must be tested**. If users can type it, we must test it.
* Maintain or improve overall test coverage:

  * CLI behavior: **100%** test coverage expected
  * Logic-level code: **80% minimum**
* Run coverage check:

  ```bash
  cargo tarpaulin --out Html
  # or
  cargo llvm-cov --workspace --html
  ```

### Test Types

* **Unit Tests**: Single-function or module-level logic
* **Integration Tests**: Modules working together
* **E2E Tests** (optional): End-user workflows

### CLI Testing Tools

Use [`assert_cmd`](https://docs.rs/assert_cmd), [`clap::Command::debug_assert`](https://docs.rs/clap/latest/clap/struct.Command.html#method.debug_assert), and similar tools to test CLI surfaces.

```rust
use assert_cmd::Command;

#[test]
fn runs_with_verbose_flag() {
    let mut cmd = Command::cargo_bin("your_tool").unwrap();
    cmd.arg("--verbose").assert().success();
}
```

---

## 🔄 CI/CD Standards

All pull requests are validated with CI using GitHub Actions:

* Format: `cargo fmt --check`
* Lint: `cargo clippy -- -D warnings`
* Test: `cargo test`
* Coverage: `cargo llvm-cov` or `cargo tarpaulin`
* Build: `cargo build --release`

---

## 🧹 Code Style

* Follow Rust’s official guidelines
* Use idiomatic Rust
* Run formatters and linters:

  ```bash
  cargo fmt
  cargo clippy
  ```
* Follow project-specific conventions where applicable (e.g., naming, module layout)

---

## 📜 Commit Message Convention

Use conventional commits:

```
type(scope): short description
```

**Types**:

* `feat` – New features
* `fix` – Bug fixes
* `docs` – Documentation only
* `style` – Formatting, no code change
* `refactor` – Code changes that aren’t fixes/features
* `test` – Adding or updating tests
* `chore` – Build or tool changes

**Example**:

```bash
git commit -m "feat(parser): add support for nested syntax blocks"
```

---

## 📤 Publishing & Releases

(For crates with release support)

### Versioning

Follow [Semantic Versioning](https://semver.org):

* **Patch** (0.1.0 → 0.1.1): Fixes
* **Minor** (0.1.0 → 0.2.0): New non-breaking features
* **Major** (0.x.x → 1.0.0): Breaking changes

### Publishing Process

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
git commit -m "chore: bump version to 0.4.0"
git tag -a v0.4.0 -m "Version 0.4.0"
git push && git push --tags
cargo publish
```

---

## 📚 Documentation

Update the following when you add or change features:

* `README.md`
* Public API doc comments (`///`)
* Example/test files
* Syntax/reference documentation (if applicable)
* `CHANGELOG.md`

All code should include:

* Function and argument documentation
* Return values and side effects
* Usage examples for complex functions

### Good vs. Poor Documentation Examples

**Good:**

````rust
/// Calculates the area of a circle.
///
/// # Arguments
/// * `radius` - A floating-point number representing the radius.
///
/// # Returns
/// The area as `f64`.
///
/// # Example
/// ```
/// let a = circle_area(2.0);
/// assert_eq!(a, 12.566);
/// ```
````

**Poor:**

```rust
// calc circle
fn c(r: f64) -> f64 {
```

---

## 🧺 Development Setup

To get started:

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/project-name
cd project-name

# Setup
cargo build
cargo test
```

Create a branch:

```bash
git checkout -b feature/my-new-feature develop
```

---

## ❓ Merge Conflicts & Troubleshooting

* Always pull the latest changes before merging:

  ```bash
  git pull origin develop
  ```
* Resolve conflicts manually and commit resolved state
* Run tests again after resolving conflicts

If you encounter setup issues, check the `README.md` or open an issue for help.

---

## ⏳ Pull Request Reviews

* PRs should include a clear description of the changes
* Link to related issues when possible
* Expect a response within 3 business days
* Be open to constructive feedback and iterate as needed

---

## 🙏 Thank You!

Your contributions help make this project better. Whether it’s reporting bugs, suggesting improvements, writing docs, or submitting code — we’re glad to have you involved.
