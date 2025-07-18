# DSC CLI Grammar Specification

This document defines the **DSC command-line interface grammar**, a universal syntax model originally developed for the `dsc` tool but suitable for use across any file-oriented, declarative CLI application.

## 🎯 Design Goals

- **Consistent syntax** across all tools
- **Readable and composable** CLI expressions
- **FIFO semantics** for grouped flags (order matters)
- **Minimalist but expressive** for power users
- **Declarative structure**, not just option parsing

---

## 🧱 Command Structure

A typical DSC CLI command follows this form:

```
<TOOL> <PATH> <FILTERS> <ROUTES> <FLAGS>
```

### Positional Semantics

| Segment   | Description                                         |
| --------- | --------------------------------------------------- |
| `PATH`    | The starting point for the operation (default: `.`) |
| `FILTERS` | UPPERCASE declarative filters (e.g. `EXT:rs`)       |
| `ROUTES`  | Directives like `TO:`, `INTO:`, or `FORMAT:`        |
| `FLAGS`   | Short, ordered modifiers like `-hp`, `-csic`        |

All segments are parsed left-to-right. Positional clarity is **mandatory**.

---

## 🎛️ Short Flags (Stackable)

Flags are **initial-based**, compact, and stackable like `-rp`, `-csic`. They must be processed **in order** (FIFO behavior).

### Flag Behavior Table

| Flag | Name               | Description                      |
| ---- | ------------------ | -------------------------------- |
| `h`  | `--hidden`         | Include hidden files             |
| `ni` | `--no-ignore`      | Disable ignore rules             |
| `p`  | `--paths`          | Output paths only                |
| `f`  | `--follow`         | Follow symlinks                  |
| `cs` | `--case-sensitive` | Force case-sensitive matching    |
| `ic` | `--ignore-case`    | Force case-insensitive matching  |
| `r`  | `--regex`          | Interpret query as regex         |
| `g`  | `--glob`           | Interpret query as glob          |
| `t`  | `--type`           | Filter by type (file, dir, etc.) |
| `e`  | `--extension`      | Match file extensions            |

If a long-form flag is used, it overrides any matching short form in a flag group.

---

## 🧠 Filters (Declarative UPPERCASE Syntax)

Filters are declarative key-value statements:

```
KEYWORD:value
KEYWORD<value
KEYWORD>value
```

### Examples

- `EXT:md` — files with .md extension
- `SIZE>1MB` — files larger than 1MB
- `MODIFIED<2024-01-01` — modified before that date

### Reserved Keywords

| Filter           | Meaning                                         |
| ---------------- | ----------------------------------------------- |
| `NAME:`          | Partial or exact match by name                  |
| `TYPE:`          | `file`, `folder`, `symlink`, etc.               |
| `EXT:`           | File extension                                  |
| `SIZE>`, `<`     | File size threshold                             |
| `DEPTH>`, `<`    | Directory depth filters                         |
| `MODIFIED>`, `<` | Modification timestamp filters                  |
| `WHERE:`         | Groups filters logically                        |
| `FOR:`           | Shorthand semantic filters (e.g. `FOR:scripts`) |

---

## 📤 Route Operators

| Keyword   | Description                                    |
| --------- | ---------------------------------------------- |
| `TO:`     | Pipe output to another tool (e.g., `TO:smv`)   |
| `INTO:`   | Write output to a file (e.g., `INTO:list.txt`) |
| `FORMAT:` | Change output encoding (e.g., `FORMAT:json`)   |

Routes are interpreted **after filters**, and are ordered.

---

## 📦 Examples

```sh
# Hidden markdown files larger than 1MB, paths only
smv . EXT:md SIZE>1MB -hp

# All .rs files piped to SMV for preview renaming
xfd ./src EXT:rs TO:smv -rp

# Search by name and route to file
xfd . NAME:report INTO:matches.txt FORMAT:csv -p

# Use predefined filter group
xfd . FOR:scripts NAME:backup -r
```

---

## ⚠️ Reserved Flag Rules

- Flag collisions must be documented and avoided across tools
- Each tool may extend the CLI spec, but must not redefine shared short flags
- Help output must include expanded flag group meaning (e.g. `-csic = --case-sensitive --ignore-case`)

---

## 📚 Extending the Grammar

Future grammar enhancements should:

- Reuse the existing structure and token rules
- Avoid case ambiguity in keyword parsing
- Be reviewed in a central DSC CLI Grammar changelog

All tools implementing the DSC grammar must adhere to this specification unless explicitly exempted.

