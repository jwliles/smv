Here’s a **draft specification for the `.cnd` format**, based on everything we’ve discussed. It’s structured so it can evolve later (binary compression, alternate encodings) but gives us a **clear initial schema** for development.

------

# **CNP Content Delta (`.cnd`) Specification – Draft v1**

------

## **Purpose**

- `.cnd` files capture **content-level changes** (token and AST diffs) for files tracked by CNP.
- Each `.cnd` represents **one patch layer** in the content history.
- `.cnd` layers are linked to `.cnp` metadata and can be **stacked sequentially** to reconstruct the full file content history.

------

## **File Characteristics**

- **Format**: Human-readable (YAML/TOML-like key-value with structured sections).
- **Compression**: Optional (gzip/zstd) for storage efficiency; uncompressed default.
- **Granularity**: Token-level changes, with optional AST node metadata for semantic operations.
- **Linkage**: Referenced from `.cnp` via `content_delta:` field (patch ID or relative path).

------

## **Schema Overview**

### **Header**

- Provides file version, linkage, and basic metadata.

### **Changes**

- Ordered list of content modifications (range-based).

------

### **Example**

```text
version: 1
patch_id: 1a2b3c4d
timestamp: 2025-07-26T14:52:00Z
file_id: f89b0a
linked_cnp: /ledger/2025-07-26/scan123.cnp
tool:
  name: edt
  command: "replace foo -> bar"
hash_before: a3b5f6
hash_after: d4e7c8

changes:
  - range:
      start_line: 10
      end_line: 12
      start_byte: 234
      end_byte: 310
    type: modify
    old_tokens:
      - "foo"
      - "="
      - "42"
    new_tokens:
      - "bar"
      - "="
      - "84"
    ast_node: variable_assignment

  - range:
      start_line: 20
      end_line: 20
      start_byte: 451
      end_byte: 472
    type: add
    new_tokens:
      - "// added comment"
    ast_node: comment
```

------

## **Field Definitions**

### **Header Fields**

- `version` *(int)*
   Format version (starting at 1). Allows forward compatibility.
- `patch_id` *(string)*
   Unique identifier for this content delta (UUID or hash).
- `timestamp` *(ISO-8601)*
   UTC timestamp when delta was generated.
- `file_id` *(string)*
   Unique identifier linking to `.cnp` file entry.
- `linked_cnp` *(string)*
   Path or ID of `.cnp` metadata file this `.cnd` layer belongs to.
- `tool` *(object)*
  - `name`: Tool that generated the change (`edt`, `smv`, etc.).
  - `command`: Optional CLI or operation description.
- `hash_before` / `hash_after` *(string)*
   Content hashes (SHA-256 or BLAKE3) of file before/after patch application.

------

### **Change Entries**

- `range` *(object)*
  - `start_line` / `end_line`: 1-based inclusive lines affected.
  - `start_byte` / `end_byte`: Byte offsets for precise patching.
- `type` *(enum)*
  - `add`, `remove`, `modify`
- `old_tokens` *(array)*
   Token list removed or modified (may be omitted for pure additions).
- `new_tokens` *(array)*
   Token list added or modified (may be omitted for pure deletions).
- `ast_node` *(string, optional)*
   Tree-Sitter node type (e.g., `function_declaration`, `comment`).

------

## **Operational Notes**

### **Layering**

- Each `.cnd` is **append-only** and represents one patch.
- To reconstruct file content:
  - Start from baseline snapshot (first `.cnd` or initial file).
  - Apply patches in chronological order.

### **Overlap Resolution**

- If multiple `.cnd` patches affect the same range:
  - Default: chronological override (later patches replace earlier).
  - Centcom merge UI can present conflicts for manual resolution.

### **Version Control**

- Periodic compaction allowed:
  - Merge multiple `.cnd` patches into new baseline snapshot.
  - Update `.cnp` to point to new baseline.

### **Integration**

- `.cnp` handles **metadata deltas** (path, permissions, xattrs).
- `.cnd` handles **content deltas** (tokens/AST).
- INX stores and queries both; Centcom visualizes combined history.

------

## **Future Extensions**

- **Binary mode**: For high-volume deployments (store token diffs in binary form).
- **Semantic tagging**: Enrich diffs with higher-level info (e.g., “renamed function”).
- **Selective indexing**: Allow partial content diffs (comments only, code only).
- **Compression layers**: Merge `.cnd` patches periodically for long-lived files.

------

Do you want me to **also draft how `.cnd` and `.cnp` interact in the INX database schema** (tables/keys, relationships)?
 Or **focus purely on the file spec for now** and do DB design later?
