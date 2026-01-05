---
date_created: 2025-10-02T11-25-26
date_updated: 2025-08-30T02-49-20
timestamp: 1759404326318
title: installation
id: ba8d3253-e406-4739-8ae1-75f09b7bc0cb
hash: e28b33b9dd13ed625fbe33997d8fd79c4cd5c779e4d09389726275e7ae04428c
---
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