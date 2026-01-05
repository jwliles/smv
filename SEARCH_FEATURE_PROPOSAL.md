# **Feature Proposal: Multi-Source Move with `FROM:`/`TO:` Filters**

## **Overview**

Add explicit `FROM:` and `TO:` filters to SMV’s `-move` (and compatible scopes) to enable **multi-source file selection** using **regex (default)** or **glob (`-g`)**, while keeping CNP grammar consistent and highly readable.

---

## **Goals**

* Support **multi-source moves** without ambiguous positional arguments.
* Use **`FROM:`** for comma-separated source paths/patterns.
* Use **`TO:`** for explicit destination path.
* Maintain **prefix consistency** (`-move`, `-sort`, `-snake`, etc.).
* Keep **regex default** and **glob optional** for power users.

---

## **Canonical Syntax**

```bash
smv -move EXT:png FROM:a/,b/ TO:c/ -r
```

* **Scope**: `-move`
* **Filter**: `EXT:png` (matches `.png` files)
* **Source**: `FROM:a/,b/` (two directories, comma-separated)
* **Destination**: `TO:c/`
* **Modifier**: `-r` (recursive)

---

## **Behavior**

### **Regex vs Glob**

* **Regex**: Default pattern matching for `FROM:` sources and filters.
* **Glob**: Opt-in with `-g` flag for `*.png`, `?file`, etc.

### **Multi-Source**

* Comma-separated paths within `FROM:` (no spaces required).
* Filters like `EXT:`, `NAME:`, `SIZE>` apply **across all sources**.
* Deduplication applied if overlapping patterns match the same file.

### **Destination Handling**

* `TO:` specifies exactly one destination path.
* `-cd` modifier creates the destination if missing.
* Destination defaults to current directory (`.`) if omitted (future option).

### **Modifiers**

* `-r` — Recurse into subdirectories
* `-g` — Interpret patterns as glob instead of regex
* `-i` — Case-insensitive match
* `-p` — Preview (dry-run)
* `-cd` — Create destination directory

---

## **Examples**

### **Basic Multi-Source Move**

```bash
smv -move EXT:png FROM:a/,b/ TO:c/
```

### **Recursive Regex**

```bash
smv -move NAME:'^img\d+' FROM:a/,b/ TO:c/ -r
```

### **Glob Multi-Source**

```bash
smv -move EXT:png FROM:a/*.png,b/*.png TO:c/ -gr
```

### **Sort by First Letter**

```bash
smv -sort BY:L1 EXT:png FROM:a/,b/ TO:sorted/ -cd
```

### **Combine Filters**

```bash
smv -move EXT:png SIZE>1MB FROM:~/Downloads/,~/Desktop/ TO:~/Pictures/ -ri
```

---

## **Integration with CNP Grammar**

### **Scope**

* All SMV operations are invoked via **prefixed scopes**:

  * `-move`
  * `-sort`
  * `-snake`
  * `-template`
  * (future) `-undo`

### **Filters**

* `EXT:`, `NAME:`, `SIZE>`, `WHERE:`, etc. apply to sources listed in `FROM:`.

### **Routes**

* `FROM:` — Explicitly define one or more sources (comma-separated).
* `TO:` — Explicitly define a single destination.
* Compatible with other route keywords (`INTO:`, `FORMAT:`) if extended later.

---

## **Implementation Notes**

* **Parsing**:

  * Recognize `FROM:` and `TO:` as special route keywords.
  * Split `FROM:` value by comma into list of source patterns.
  * Validate `TO:` as destination (create with `-cd`).

* **Matching**:

  * Apply filters to each source individually, merge results.
  * Deduplicate paths if overlapping matches.

* **Execution**:

  * Preview mode (`-p`) shows source → destination mapping.
  * Respect `force`/overwrite behavior from current SMV move logic.

* **Extensibility**:

  * `EXCLUDE:` can be added later for negative selection.
  * Could integrate with `BY:` for hierarchical sorts.

---

## **Man Page Integration (Draft)**

### **NAME**

```
smv -move — Move and organize files from multiple sources using filters
```

### **SYNOPSIS**

```
smv -move [FILTERS] FROM:<sources> TO:<destination> [FLAGS]
```

### **DESCRIPTION**

Move files or directories from one or more source paths into a destination path using declarative filters. Supports regex (default) or glob (`-g`) patterns.

### **EXAMPLES**

```
# Move all .png files from a/ and b/ into c/
smv -move EXT:png FROM:a/,b/ TO:c/

# Recursively move regex-matched files
smv -move NAME:'^img\d+' FROM:a/,b/ TO:c/ -r

# Use glob matching
smv -move EXT:png FROM:a/*.png,b/*.png TO:c/ -gr

# Sort into lettered directories
smv -sort BY:L1 EXT:png FROM:a/,b/ TO:sorted/ -cd
```

---

## **Advantages**

* **Explicit & readable**: Clear intent with `FROM:`/`TO:` semantics.
* **Consistent**: Aligns with existing CNP keyword-based grammar.
* **Flexible**: Supports regex (powerful) and glob (convenient).
* **Extensible**: Easily adds `EXCLUDE:` or nested `BY:` logic later.
