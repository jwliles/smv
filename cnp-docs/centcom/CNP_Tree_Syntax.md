# Tree Output Format Specification for `dsc`

This document defines the canonical format and output variants for structured directory tree reporting in the `dsc` tool. The goal is to provide reproducible, extensible, and machine-friendly representations of filesystem structure across multiple output modes.

------

## 🎯 Purpose

The tree output mode (`-t` and its variants) in `dsc` provides a consistent view of the directory layout rooted at the current working directory (CWD). All tree formats share the same underlying data structure and encode:

- Entry type (directory or file)
- Tree level (0-based depth)
- Relative path
- Optional human-friendly comment
- Optional structured metadata (e.g., size, mtime)

------

## 🧱 Canonical Internal Structure

```rust
struct TreeEntry {
    kind: EntryKind,             // File or Directory
    level: usize,                // 0-based depth from root
    path: String,                // Relative path using `/` separators
    comment: Option<String>,     // Human-readable annotation
    metadata: Option<TreeMeta>,  // Optional structured metadata
}

enum EntryKind { File, Directory }

struct TreeMeta {
    size_bytes: Option<u64>,
    mtime: Option<SystemTime>,
    mode: Option<u32>,
    hash: Option<String>,
}
```

All output formats are deterministic serializations of this structure.

------

## 🧩 Section Delimitation

All tree output formats are divided into two explicit sections:

```text
## SECTION: DIRECTORIES
... (D entries)

## SECTION: FILES
... (F entries)
```

This allows for clean parsing, splicing, and consistent ordering regardless of output format.

------

## 📜 Text Format (`-tr`) — RDM Style

### Example:

```text
## SECTION: DIRECTORIES
D0 - src
D1 - src/bin

## SECTION: FILES
F1 - Cargo.toml               # Project manifest
F2 - src/bin/main.rs          # Entry point
```

### Rules:

- `D` / `F` prefix + level (e.g., `D1`, `F2`)
- Full path from root using `/`
- Optional comment aligned after two spaces and `#`
- Sorted: section ➜ level ➜ path

------

## 🟡 YAML Format (`-ty`)

### Example:

```yaml
- type: dir
  level: 1
  path: src/bin
  comment: Rust binaries

- type: file
  level: 2
  path: src/bin/main.rs
  comment: Entry point
```

### Notes:

- Field names: `type`, `level`, `path`, `comment`
- Metadata fields included if `--annotate` is used

------

## 🟠 JSON Format (`-tj`)

### Example:

```json
[
  { "type": "dir", "level": 1, "path": "src/bin", "comment": "Rust binaries" },
  { "type": "file", "level": 2, "path": "src/bin/main.rs", "comment": "Entry point" }
]
```

------

## 🔵 Markdown Format (`-tm`)

### Example:

```markdown
- 📁 src/bin _(Rust binaries)_
  - 📄 main.rs _(Entry point)_
```

- Output is intended for human consumption
- Comments shown in parentheses or italics
- Icons can be disabled with `--no-icons`

------

## ⚙️ Tree Metadata Extension (via `--annotate`)

Users may request metadata fields using:

```bash
dsc -tr --annotate=size,mtime
```

This populates the `metadata` field in all structured formats. For text, metadata may be emitted inline as tagged comments:

```text
F2 - src/bin/main.rs  # size=1042B, modified=2024-06-01
```

------

## 📏 Output Determinism

All tree outputs must:

- Begin at inferred CWD (no `.` or root name prefix)
- Emit directories before files
- Sort entries by: section ➜ level ➜ path (lexicographically)
- Always use `/` as path separator
- Preserve original casing

------

## 🧪 Future Format Support

- `-tf` — flat linear list (no indentation)
- `-td` — directories only
- `-tR` — alias for `-tr`
- `-tyc` — YAML + comment export

------

## ✅ Summary

The `dsc` tree output system is designed for structural reporting, reproducible exports, and flexible format emission across CLI, CI, documentation, and scripting use cases. All formats are unified under one internal model and guaranteed to contain consistent information.
