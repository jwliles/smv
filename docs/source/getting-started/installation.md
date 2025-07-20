# Installation

## Prerequisites

- Rust 1.70 or later
- Cargo package manager

## Install from Crates.io

```bash
cargo install smv
```

## Install from Source

```bash
git clone https://github.com/jwliles/smv
cd smv
cargo install --path .
```

## Verify Installation

```bash
smv --version
```

## Quick Test

```bash
# Test with preview mode (safe)
smv snake . -p
```

You should see a preview of snake_case transformations for files in the current directory.