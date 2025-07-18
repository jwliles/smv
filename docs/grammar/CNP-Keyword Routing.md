# CNP Keyword Delegation and Collision Specification

## Purpose

This document formalizes the behavior of keyword handling in the CNP (Canopy) CLI ecosystem. It defines how tools such as `say`, `inx`, `xfd`, `cmp`, and `smv` should handle keyword recognition, delegation, and collision resolution to ensure clarity, modularity, and transparency in user interactions.

---

## Philosophy

* **Transparency**: All delegation and routing of keywords should be disclosed to the user.
* **Single Responsibility**: Each tool should handle only its domain-specific functionality.
* **Composable Design**: Tools should interoperate cleanly, chaining logically when necessary.
* **User-Centric Wording**: Keywords should be intuitive, expressive, and non-arcane.

---

## Core Behavior

### Delegation Contract

If a tool receives a keyword that it recognizes but does **not own**, it must:

1. **Log a clear warning** to the user
2. **Explain what tool will handle the keyword instead**
3. **Show the equivalent command pipeline, if possible**
4. **Perform the delegation silently in CLI pipelines**

### Example Output

```bash
$ say match 'SIZE: >1MB ENDS: .zip'

[CNP] ⚠️ Detected out-of-scope keyword: SIZE
[CNP] ⏩ Delegating SIZE filter to INX
[CNP] 🧠 Equivalent pipeline: inx --size '>1MB' | say match 'ENDS: .zip'
```

---

## Keyword Collision Matrix

| Keyword      | Meaning                             | Primary Tool | Potential Collisions | Collision Type | Delegation Behavior                  |
| ------------ | ----------------------------------- | ------------ | -------------------- | -------------- | ------------------------------------ |
| `IS:`        | Exact name match                    | `say`        | `xfd`                | soft           | Retained by `say`                    |
| `STARTS:`    | Name prefix                         | `say`        | `xfd`                | soft           | Retained by `say`                    |
| `ENDS:`      | Name suffix                         | `say`        | `xfd`, `say`         | soft           | Retained by `say`                    |
| `EXT:`       | File extension                      | `say`        | `xfd`, `inx`         | soft           | Retained by `say`                    |
| `SIZE:`      | File size in bytes                  | `inx`        | `say`                | hard           | Delegated to `inx`                   |
| `MTIME:`     | Modified time                       | `inx`        | `say`                | hard           | Delegated to `inx`                   |
| `CTIME:`     | Creation time                       | `inx`        | `say`                | hard           | Delegated to `inx`                   |
| `PERMS:`     | File permissions                    | `inx`        | `say`                | hard           | Delegated to `inx`                   |
| `DUPLICATE:` | Files with same content             | `cmp`        | `say`, `inx`         | hard           | Delegated to `cmp`                   |
| `HASH:`      | Checksum or content hash            | `cmp`        | `say`                | hard           | Delegated to `cmp`                   |
| `WITH:`      | Presence of related file            | `say`        | `smv`, `rpt`         | soft           | Retained by `say`                    |
| `WITHOUT:`   | Absence of related file             | `say`        | `smv`                | soft           | Retained by `say`                    |
| `OWNER:`     | File ownership                      | `inx`        | `say`                | hard           | Delegated to `inx`                   |
| `CONTENT:`   | File content search                 | (future)     | `say`, `rpt`         | hard           | Currently rejected                   |
| `TYPE:`      | File type (file/dir/symlink)        | `inx`        | `say`                | soft           | Aliased in `say`, validated by `inx` |
| `MATCH:`     | Entry point for expression matching | `say`        | `smv`, `xfd`, `inx`  | soft           | Always parsed by `say`               |

---

## Keyword Precedence and Evaluation Order

In tools like `say`, keywords are written in sentence-style syntax, not positionally or hierarchically. This necessitates a clearly defined precedence model to ensure predictable behavior.

### 🧠 Evaluation Principles

1. **Logical flow is inferred** from keyword type, not position.
2. **Precedence** is based on keyword category, not left-to-right order.
3. **Scope chaining** is supported using Boolean logic (`AND`, `OR`, `WHILE`, `FOR`, `WITHOUT`).
4. **Implicit continuation** is allowed — e.g., multiple values after a keyword like `CONTAINS:` inherit the same keyword until another appears.

### 🏷️ Keyword Categories (by precedence)

| Precedence | Category         | Keywords                          | Notes                      |
| ---------- | ---------------- | --------------------------------- | -------------------------- |
| 1          | Logical Control  | `AND`, `OR`, `WHILE`, `FOR`       | Define boolean flow        |
| 2          | Core Filters     | `IS:`, `STARTS:`, `ENDS:`, `EXT:` | Fundamental name filters   |
| 3          | Metadata Filters | `SIZE:`, `MTIME:`, `OWNER:`, etc. | From delegated tools       |
| 4          | Modifiers        | `WITH:`, `WITHOUT:`               | Post-filters               |
| 5          | Grouping         | Parentheses `(...)`               | Used to force custom order |

### 🔄 Example Interpretation

```bash
say match 'STARTS: report AND (EXT: .md OR .txt) WITHOUT draft'
```

* Applies `STARTS:` first
* Applies grouped `EXT:` logic next
* Applies `WITHOUT:` last as a narrowing constraint

--

## Design Guidelines

* `say` is **never** responsible for stat metadata or filesystem inspection.
* Tools like `inx` and `cmp` may pre-filter input then pipe into `say` for name and logic evaluation.
* All keywords must be documented with their owning tool and known delegates.
* All tools must emit structured output on delegation unless explicitly silenced.

---

## Future Work

* Add `cnp help keywords` command to show live ownership/delegation map
* Define a central `enum KeywordOwner` registry in `cnp-core`
* Add support for opt-in delegation logs or JSON-structured audit output for automation contexts

---

## Summary

This spec ensures that the CNP ecosystem remains modular, user-friendly, and fully transparent. It prevents tool overlap, encourages correct usage, and enhances composability without increasing user burden. All CNP tools must conform to this behavior to maintain consistency and trust.

**End of spec.**
