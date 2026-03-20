# ✅ Implemented Tools

## Published

### `smv` – Smart Move - on v0.4.0

* Meant to `mv`
* Advanced file and folder renamer/mover
* Supports previews, dry-runs, interactive and batch modes
* Extension filters, snake/kebab/camel case transformers

### `pathmaster` – PATH Manager - on V0.2.9

* CLI utility for safely managing the system PATH environment variable
* Add/remove/list PATH entries, validate and flush broken ones
* Backup/restore system with timestamped snapshots
* Interactive shell detection and configurable persistence

## In Development

### `skl` – Skeleton (Content Searcher) - in MVP state

*Fast recursive content searcher for files within the filesystem or INX index.*

* Meant to replace `fzf`
* Supports filters:

  * `CONTAINS:"text"` — string match
  * `REGEX:"pattern"` — regex match
  * `LNUM:N` — match line numbers
  * `EXT:` / `TYPE:` — combine with file filters
* Optional output modes:

  * Highlighted text in terminal
  * JSON for scripting
  * Context lines (`--before`, `--after`)
* Uses `say` for advanced pattern matching
* May fall back to `dsc` when outside index boundaries

### `inx` – Indexer - Needs work; There are many bugs

* Sole indexer in Canopy
* Maintains a metadata store of filesystem state
* Used by all read-only tools for querying and reporting

### `xfd` – Extra Find - in MVP state

* Meant to replace `ripgrep`
* Find and filter tool for files and folders
* Must only operate on INX data (not raw FS scan)
* Supports name/extension/depth/type filters, and soon regex (`say`) integration

### `dsc` – Discovery - in MVP state

* Ultra-fast live file and directory scanner
* Can operate independently or power INX fallback
* Drop-in `fd` replacement with extended filters and stats
* Beats `fd` on every scanning benchmark

### `edt` – Structural Editor - in MVP state

8 Meant to repalce `sed`
* Intelligent, interactive multi-file content editor
* Designed for structural find-and-replace operations
* Supports per-match review, case propagation, and Tree-sitter integration
* Includes both TUI and GUI interfaces, undo history, and syntax awareness

### `rpt` – System Reporter - in MVP state

* Comprehensive system introspection tool (like neofetch but much more powerful)
* Single unified interface for all system information: disks, network, mounted devices, hardware, memory, processes
* Atomic breakdowns - drill down from high-level overview to specific components
* Multiple output formats: JSON, Markdown, CSV, terminal-friendly
* Replaces dozens of scattered Unix tools (df, du, lsblk, ifconfig, mount, free, lscpu, etc.)
* Examples: `rpt disk`, `rpt network --interface=eth0`, `rpt mount`, `rpt system --format=json`

## 🧪 In Planning

### `say` – Matching Engine (formerly Speakeasy) - in Planning

* Query language and pattern matcher used across Canopy tools
* Human-readable alternative to regular expressions
* Future: fuzzy and phonetic match extensions
* Used by `xfd`, `skl`, and any tool needing complex filters

### `cmp` – Compare - in Planning

* Detects file and directory differences across snapshots
* Compares structure, names, and optionally hashes (if INX has that data)

### `arc` – Archive - in Planning

* Zip/tar/7z compatible archival utility
* May include restore + diff + verify options
* Possible hooks into `smv`/`mkr` to archive pre-move/post-restore states

### `dff` – Duplicate File Finder - in Planning

* Compares files by size, name, and optionally hash
* Integrates with INX and optionally `say` for advanced matching

### `mkr` – Maker - Overlaps a bit with SMV; Not Started

* Structured directory and file creator
* Uses `.df` templates to generate folder/file trees
* Supports isolated creation (only dirs, only files, or named subset)

### `cnp` – Unified Shell & Coordinator - in Planning

* REPL-style interactive shell environment
* Hosts and dispatches all CNP tools (`smv`, `xfd`, `inx`, etc.)
* Provides undo history, preview engine, and region-based scopes
* Tools can also be run standalone or via `cnp <tool>`
* Offers CLI, TUI, and GUI interfaces, all with shared state and APIs

---

This document will be updated continuously as the Canopy suite evolves.

## Canopy Tools Overview

This document outlines the official tools included in the Canopy suite (CLI name: `cnp`). Each tool adheres to the Canopy philosophy: composability, clarity, and speed. All tools must respect index boundaries (INX) and share a common interface logic where possible.

---

### ✅ Implemented Tools

#### `smv` – Smart Move

* Advanced file and folder renamer/mover.
* Supports previews, dry-runs, interactive and batch modes.
* Extension filters, snake/kebab/camel case transformers.

#### `mkr` – Maker

* Structured directory and file creator.
* Uses `.df` templates to generate folder/file trees.
* Supports isolated creation (only dirs, only files, or named subset).

#### `inx` – Indexer

* Sole indexer in Canopy.
* Maintains a metadata store of filesystem state.
* Used by all read-only tools for querying and reporting.

#### `xfd` – Extra Find

* Find and filter tool for files and folders.
* Must only operate on INX data (not raw FS scan).
* Supports name/extension/depth/type filters, and soon regex (`say`) integration.

---

### 🧪 In Development

#### `skl` – Skeleton

* **Goal:** A replacement for `tree`, `ls`, and `inxi` (for filesystem insights).
* **INX-backed only**: never touches the filesystem directly.
* **Features:**

  * Tree or flat view of files and folders.
  * Displays name, size, type, full path, owner, permissions, created/modified/accessed times.
  * Can output in tree, table, JSON, or `.df` format.
  * `--summary` mode: high-level breakdowns by size, type, extension, etc.
  * `--template` mode: outputs a `mkr`-ready `.df` structure.
  * `--json` mode: machine-readable output for scripting.
  * Optional color and icon theming in future.

#### `say` – This is just short for speakeasy and was a different form of speak (formerly Speakeasy-Regex)

* Query language and matching engine.
* Can be used by `xfd`, `skl`, or any tool needing fuzzy or complex pattern matching.
* Replaces regular expressions with human-friendly matching DSL.
* Extensions like fuzzy logic and phonetic matching may be built into the core or deferred until later.

#### `look`

* Lightweight, fast system and directory statistics tool.
* Inspired by `inxi` + `info.rb`.
* Will eventually be merged into or called by `skl --summary`.
* Built in Crystal (not Rust).

#### `rpt` – Reporter

* Converts output from INX/XFD/SMV into stylized reports.
* Possible output formats: Markdown, CSV, Canopy-native.
* May integrate with REPL for inspection.

#### `cmp` – Compare

* Detects file and directory differences across snapshots.
* Compares structure, names, and optionally hashes (when INX has that data).

#### `arc` – Archive

* Zip/tar/7z compatible archival utility.
* May include restore + diff + verify options.
* Possible hooks into `smv`/`mkr` to archive pre-move/post-restore states.

#### `dff` – Duplicate File Finder

* Compares files by size, name, and optionally hash.
* Integrates with INX and optionally `say` for advanced matching.

---

### 🔮 Long-Term or Experimental Concepts

* **`cnp repl`**

  * Evaluate, inspect, and preview Canopy toolchains interactively.
  * May be superseded by a fully capable TUI in the future.
  * Kept here for completeness but deprioritized.

---

This document will be updated continuously as the Canopy suite evolves.
