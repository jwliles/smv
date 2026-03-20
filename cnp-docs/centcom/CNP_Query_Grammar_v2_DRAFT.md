---
date_created: 2025-11-14
status: DRAFT
title: CNP Query Grammar v2 - Semantic Phase-Based Execution (DRAFT)
pilot_implementation: rpt v0.1.0
---

# CNP Query Grammar v2: Semantic Phase-Based Execution

**Status**: DRAFT - Piloted in RPT
**Purpose**: Define position-independent, semantic phase-based argument parsing for CNP Query tools
**Pilot Tool**: `rpt` (reporting tool)
**Supersedes**: Positional argument parsing in CNP_Query_Grammar v1

---

## 🎯 Executive Summary

CNP Query Grammar v2 introduces **semantic phase-based execution** where arguments are parsed by their semantic type (not position) and executed in dependency order. This provides:

- **Position independence** - Keywords work in any order
- **Performance optimization** - Early filtering reduces wasted work
- **Better UX** - Users can append/modify commands naturally
- **Smart defaults** - Missing scope/target inferred from context

**Key Change**: Arguments are categorized into **execution phases** based on their semantic meaning, then executed in optimal dependency order regardless of input position.

---

## 🔄 Migration from v1

### What Changed

**v1 (Positional)**:
```bash
rpt files . EXT:rs FORMAT:json  # Order matters
```

**v2 (Semantic)**:
```bash
rpt tree . EXT:rs FORMAT:json   # 'files' → 'tree'
rpt tree . FORMAT:json EXT:rs   # Order doesn't matter (same result)
rpt tree EXT:rs . FORMAT:json   # Still works!
```

### Breaking Changes

1. **`files` → `tree` subcommand** - More semantic name for hierarchical structure
2. **Global type filter flags** - `-f` (files only) and `-d` (dirs only) must come before subcommand
3. **Position independence** - Keywords can appear in any order (within reason)

### Backward Compatibility

- All existing keyword syntax preserved (`:` operator, `FORMAT:`, `EXT:`, etc.)
- Existing commands work if you replace `files` with `tree`
- Default behavior unchanged (no scope → tree, no target → `.`)

---

## 🧱 Execution Phases

Arguments are parsed into **semantic phases** and executed in this order:

### Phase Order (Execution Dependency)

```
Phase  Category          Keywords/Flags                Purpose
─────  ────────────────  ───────────────────────────  ────────────────────────────
0      META              -p, -v, -h                   Execution context (parallel)
1      SCOPE             tree, sys, disk, net         Domain selection
2      TARGET            path, IN:folder1,folder2     Where to look
3      TYPE_FILTER       -f, -d, TYPE:                What kind (files/dirs/both)
4      TRAVERSAL         DEPTH:, HIDDEN:, -r          How to walk the tree
5      EXCLUSIONS        EXCL:pattern                 What to skip during walk
       ────────────────  ─────── SCAN HAPPENS ──────  ────────────────────────────
6      CONTENT_FILTER    EXT:, NAME:, SIZE:, MORE:    Item properties
7      COMPUTED          HASH:, DUPLICATE:            Expensive operations
8      RESULT_FILTER     --empty, SHOW:               Post-scan filtering
9      AGGREGATION       GROUP:, SORT:, LIMIT:        Transform results
10     OUTPUT            FORMAT:, INTO:, TO:          Present results
```

### Why This Order?

**Dependency Analysis** (from our design discussion):

```
SCOPE (no deps)
  ↓
TARGET (needs SCOPE)
  ↓
TYPE_FILTER (needs TARGET)
  ↓ ↓
  ↓ TRAVERSAL (needs TARGET + TYPE_FILTER)
  ↓   ↓
EXCLUSIONS (needs TYPE_FILTER)
    ↓
    [SCAN EXECUTION]
    ↓
CONTENT_FILTER → COMPUTED → RESULT_FILTER → AGGREGATION → OUTPUT
```

**Key Insight**: TYPE_FILTER must come before EXCLUSIONS because:
- `-d EXCL:*.txt` → Type filter makes `*.txt` exclusion meaningless (optimization)
- `-f EXCL:node_modules` → Type filter makes directory exclusion still relevant (skip traversal)

---

## 📦 Phase Details

### Phase 0: META (Execution Context)

Affects **how** the command executes, not **what** it returns.

**Flags**:
- `-p, --preview` - Preview mode (dry-run, no side effects)
- `-v, --verbose` - Verbose output with extra detail
- `-h, --help` - Show help information

**Characteristics**:
- Applied globally to all phases
- Position-independent (can appear anywhere)
- No dependencies on other phases

### Phase 1: SCOPE (Domain Selection)

Determines **what domain** to query.

**Keywords**: `tree`, `sys`, `disk`, `net`, `env`, `stats`

**Examples**:
```bash
rpt tree    # Filesystem tree scope
rpt sys     # System information scope
rpt disk    # Disk usage scope
```

**Default**: If missing and tree-specific keywords present → `tree`

### Phase 2: TARGET (Location)

Specifies **where** to look.

**Syntax**:
- Single path: `.`, `/home/user`, `./src`
- Multiple paths: `IN:src,tests,docs`

**Examples**:
```bash
rpt tree .              # Current directory
rpt tree /tmp           # Specific path
rpt tree IN:src,tests   # Multiple paths
```

**Default**: For tree scope, if missing → `.` (current directory)

### Phase 3: TYPE_FILTER (Item Type)

Filters by **what kind** of items to include.

**Flags**:
- `-f, --files-only` - Only files (not directories)
- `-d, --dirs-only` - Only directories (not files)
- No flag - Both files and directories

**Characteristics**:
- **Global flags** (must come before subcommand)
- **Mutually exclusive** (enforced by clap)
- **Applied during scan** (performance optimization)

**Examples**:
```bash
rpt -f tree .           # Files only
rpt -d tree .           # Directories only
rpt tree .              # Both (default)
```

**Implementation Note**: This is Phase 3 because it must inform both TRAVERSAL and EXCLUSIONS phases.

### Phase 4: TRAVERSAL (Walk Strategy)

Controls **how** to traverse the tree.

**Keywords**:
- `DEPTH:N` - Maximum depth (0 = current level only)
- `DEPTH:N1,N2,N3` - Multiple depths for multi-folder mode
- `HIDDEN:true|false` - Include hidden files/directories
- `-r` - Recursive (unlimited depth, redundant with DEPTH)

**Examples**:
```bash
rpt tree . DEPTH:0          # Current directory only
rpt tree . DEPTH:2          # Up to 2 levels deep
rpt tree . HIDDEN:true      # Include hidden items
rpt tree IN:src,tests DEPTH:2,1  # src at depth 2, tests at depth 1
```

### Phase 5: EXCLUSIONS (Skip Patterns)

Specifies **what to skip** during traversal.

**Syntax**:
- Global: `EXCL:pattern1,pattern2`
- Folder-specific: `EXCL:folder:pattern`

**Examples**:
```bash
rpt tree . EXCL:node_modules,target          # Skip these directories
rpt tree . EXCL:.git,*.tmp                   # Skip .git and temp files
rpt tree IN:src,tests EXCL:src:*.bak         # Skip .bak files only in src
```

**Performance**: Applied during walk to avoid scanning excluded subtrees.

### Phase 6: CONTENT_FILTER (Item Properties)

Filters by **item characteristics** after collection.

**Keywords**:
- `EXT:ext1,ext2` - File extensions
- `NAME:pattern` - Name matching (future)
- `SIZE:>1MB` or `MORE:1MB` - Minimum size
- `SIZE:<10MB` or `LESS:10MB` - Maximum size
- `MODIFIED:>7d` - Modified time (future)

**Examples**:
```bash
rpt tree . EXT:rs,toml              # Rust and TOML files
rpt tree . MORE:1MB                 # Files larger than 1MB
rpt tree . EXT:log LESS:100KB       # Small log files
```

### Phase 7: COMPUTED (Expensive Operations)

Operations requiring file reading or computation.

**Keywords** (future/delegated):
- `HASH:sha256` - Compute file hashes (delegates to `cmp`)
- `DUPLICATE:true` - Find duplicates (delegates to `cmp`)

**Note**: These operations are typically **delegated** to specialized tools per CNP_Keyword_Routing.md.

### Phase 8: RESULT_FILTER (Post-Processing)

Filters applied **after** all items collected.

**Keywords**:
- `--empty` - Show only empty files/directories
- `SHOW:empty` - Alternative syntax for empty filter

**Examples**:
```bash
rpt tree . --empty              # Find empty files/dirs
rpt tree . SHOW:empty           # Same as above
```

### Phase 9: AGGREGATION (Transform)

Transforms the **result set**.

**Keywords**:
- `GROUP:ext` - Group by extension
- `GROUP:dir` - Group by directory
- `SORT:size` - Sort by size
- `SORT:name` - Sort by name
- `LIMIT:N` - Limit result count

**Examples**:
```bash
rpt tree . GROUP:ext            # Group files by extension
rpt tree . SORT:size LIMIT:10   # Top 10 largest
```

### Phase 10: OUTPUT (Presentation)

Controls **how results** are presented.

**Keywords**:
- `FORMAT:json|tree|csv|yaml|table` - Output format
- `INTO:file.txt` - Save to file
- `TO:tool` - Delegate to another tool

**Examples**:
```bash
rpt tree . FORMAT:json                  # JSON output
rpt tree . FORMAT:csv INTO:files.csv    # Save as CSV
rpt tree . FORMAT:json TO:jq            # Pipe to jq (future)
```

---

## 🎯 Position Independence Examples

**All of these produce identical results**:

```bash
# Standard order
rpt tree . DEPTH:2 EXT:rs FORMAT:json

# Keywords reordered
rpt tree . FORMAT:json EXT:rs DEPTH:2

# Path at end
rpt tree DEPTH:2 EXT:rs FORMAT:json .

# Mixed order
rpt tree EXT:rs . DEPTH:2 FORMAT:json
```

**With type filter flags**:

```bash
# Files only, different orders
rpt -f tree . EXT:rs FORMAT:json
rpt -f tree EXT:rs . FORMAT:json
rpt -f tree FORMAT:json . EXT:rs

# All produce the same filtered result
```

---

## 🧠 Smart Defaults

### Scope Inference

If SCOPE is missing, it's inferred from keywords:

```bash
# Has tree keywords (EXT:, DEPTH:) → infers 'tree' scope
rpt . EXT:rs DEPTH:2
# Executes as:
rpt tree . EXT:rs DEPTH:2

# Has no tree keywords → error (no scope specified)
rpt FORMAT:json
# Error: No scope specified
```

**Tree Keywords** (trigger tree scope inference):
- TYPE_FILTER: `-f`, `-d`
- TRAVERSAL: `DEPTH:`, `HIDDEN:`, `-r`
- EXCLUSIONS: `EXCL:`
- CONTENT_FILTER: `EXT:`, `NAME:`, `SIZE:`, `MORE:`, `LESS:`
- Path argument present

### Target Inference

For `tree` scope, if TARGET is missing → defaults to `.` (CWD)

```bash
# Missing target
rpt tree EXT:rs
# Executes as:
rpt tree . EXT:rs

# Missing scope AND target
rpt -f EXT:rs
# Executes as:
rpt tree . -f EXT:rs
```

---

## 🔧 Implementation Details

### Parsing Algorithm

```rust
fn execute_query(args: Vec<String>) -> Result<Output> {
    // 1. Parse all arguments into phase buckets (position-independent)
    let parsed = ParsedArgs::parse_from_args(&args, &cli_flags);

    // 2. Apply smart defaults (scope and target inference)
    let resolved = resolve_defaults(parsed);

    // 3. Validate resolved command
    validate(&resolved)?;

    // 4. Execute in dependency order (phases 1-10)
    execute_phases(resolved)
}
```

### Phase Bucket Structure

```rust
struct ParsedArgs {
    // Phase 1: Scope
    scope: Option<String>,

    // Phase 2: Target
    target: Option<String>,

    // Phase 3: Type Filter
    type_filter: Option<TypeFilter>,  // FilesOnly | DirsOnly | Both

    // Phase 4: Traversal
    depth: Option<usize>,
    depth_list: Option<Vec<usize>>,
    hidden: bool,
    recursive: bool,

    // Phase 5: Exclusions
    global_excludes: Vec<String>,
    folder_excludes: HashMap<String, Vec<String>>,

    // Phase 6: Content Filters
    extensions: Vec<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,

    // Phase 7: Computed (delegated)
    // Phase 8: Result Filters
    show_empty: bool,
    show_filter: Option<String>,

    // Phase 9: Aggregation
    group_by: Option<String>,

    // Phase 10: Output
    format: Option<String>,
    into: Option<String>,
    to: Option<String>,

    // Phase 0: Meta (applied to all)
    preview: bool,
    verbose: bool,
}
```

### Argument Categorization

Arguments are categorized by their **delimiter pattern**:

```rust
if arg.starts_with("--") || arg.starts_with("-") {
    // Flag: -f, -d, -r, -p, -v, --empty
    parse_flag(arg)
} else if arg.contains(':') {
    // Keyword: EXT:, FORMAT:, DEPTH:, EXCL:, etc.
    let (key, value) = arg.split_once(':').unwrap();
    categorize_keyword(key, value)
} else if is_path(arg) {
    // Path target
    parse_target(arg)
} else {
    // Scope or error
    parse_scope_or_error(arg)
}
```

---

## ⚙️ Validation Rules

After defaults applied, validate the resolved command:

### Type Filter Validation

```rust
// Type filters only valid for tree scope
if type_filter.is_some() && scope != Scope::Tree {
    return Err("Type filters (-f, -d) only valid for tree scope");
}

// Mutual exclusivity enforced by clap
if cli.files_only && cli.dirs_only {
    return Err("Cannot use -f and -d together");
}
```

### Scope-Specific Validation

```rust
// EXT only valid for tree scope
if !extensions.is_empty() && scope != Scope::Tree {
    return Err("EXT: only valid for tree scope");
}

// COLUMNS only valid for sys/disk/net scopes
if !columns.is_empty() && scope == Scope::Tree {
    return Err("COLUMNS: not valid for tree scope");
}
```

---

## 📊 Performance Benefits

### Early Filtering (Phase 3)

**Type filter applied during traversal, not after**:

```rust
// Phase 3: TYPE_FILTER - Applied during walk
match type_filter {
    TypeFilter::FilesOnly if is_dir => return None,  // Skip early
    TypeFilter::DirsOnly if !is_dir => return None,  // Skip early
    _ => {}
}
```

**Performance impact**:
- Don't collect metadata for filtered items
- Don't compute `is_empty` for filtered items
- Don't allocate FileInfo structs for filtered items

**Example**:
```bash
# Without early filtering (old way)
rpt tree . | filter files only
# Collects 1000 files + 500 dirs → filters → 1000 files (wasted 500 dir ops)

# With early filtering (new way)
rpt -f tree .
# Collects 1000 files (skips 500 dirs during walk) → 1000 files
```

### Exclusion Optimization (Phase 5)

**Exclusions applied during traversal**:

```bash
# Skip entire subtrees
rpt tree . EXCL:node_modules,target
# Doesn't even descend into node_modules/ or target/
```

### Smart Exclusion with Type Filters

**Type-aware exclusion optimization**:

```bash
# Dirs only + file pattern exclusion → no-op
rpt -d tree . EXCL:*.txt
# Tool can optimize away *.txt pattern (only dirs being collected)

# Files only + dir exclusion → still applies
rpt -f tree . EXCL:node_modules
# Still skip traversing node_modules/ (performance win)
```

---

## 🎓 Usage Patterns

### Basic Queries

```bash
# All files in current directory
rpt tree .

# Files only
rpt -f tree .

# Directories only
rpt -d tree .

# Specific extension
rpt tree . EXT:rs

# Multiple filters
rpt -f tree . EXT:rs,toml DEPTH:2
```

### Complex Queries

```bash
# Large Rust files, files only, limited depth
rpt -f tree . EXT:rs MORE:10KB DEPTH:3 FORMAT:json

# Directories excluding common build folders
rpt -d tree . EXCL:target,node_modules,build DEPTH:2

# Multiple folders with different depths
rpt tree IN:src,tests,docs DEPTH:2,1,0 FORMAT:tree
```

### Position Independence in Practice

```bash
# Iterative refinement (just append)
rpt tree .
rpt tree . EXT:rs                    # Add extension filter
rpt tree . EXT:rs DEPTH:2            # Add depth limit
rpt tree . EXT:rs DEPTH:2 -f         # Wait, -f must come first!
rpt -f tree . EXT:rs DEPTH:2         # Correct

# Forgot path? Just add it
rpt tree EXT:rs
rpt tree . EXT:rs                    # Add path anywhere
```

---

## 🔮 Future Extensions

### Additional Phases (Planned)

- **Phase 2.5: SCOPE_MODIFIERS** - Modify scope behavior (e.g., `--follow-symlinks`)
- **Phase 6.5: CONTENT_TRANSFORM** - Transform content during filtering (e.g., `CASE:insensitive`)
- **Phase 9.5: STATISTICAL** - Statistical aggregations (e.g., `AVG:size`, `SUM:size`)

### Enhanced Smart Defaults

- **Context-aware defaults** based on current directory
- **History-based inference** from recent commands
- **Project-aware defaults** (e.g., detect project type → default exclusions)

### Multi-Tool Composition

```bash
# Semantic composition across tools
xfd . SIZE:>1GB | rpt tree . FORMAT:table | smv mv ./archive/
# Each tool applies semantic execution independently
```

---

## 📋 Migration Checklist

### For Tool Developers

- [ ] Implement ParsedArgs struct with phase buckets
- [ ] Add position-independent argument parser
- [ ] Implement smart default resolution
- [ ] Add validation for resolved commands
- [ ] Update tests for position independence
- [ ] Document phase execution order
- [ ] Add performance benchmarks for early filtering

### For Users

- [ ] Replace `files` with `tree` in existing scripts
- [ ] Move `-f`/`-d` flags before subcommand
- [ ] Test position independence with your common queries
- [ ] Review exclusion patterns with type filters
- [ ] Update any automation/scripts

---

## ✅ Status & Next Steps

### Pilot Implementation (RPT v0.1.0)

**Completed**:
- ✅ Phase-based parsing and execution
- ✅ Position-independent keyword parsing
- ✅ Smart defaults (scope and target inference)
- ✅ Type filter flags (`-f`, `-d`) with early filtering
- ✅ 23 passing tests including position independence
- ✅ Performance optimization (filter during scan)

**Known Limitations**:
- Smart defaults only for tree scope (sys/disk/net unchanged)
- Global flags must come before subcommand (clap limitation)
- Some clippy warnings about manual string stripping (not related to this change)

### Feedback Required

1. **Position independence**: Does the UX improvement justify the complexity?
2. **Performance gains**: Are early filtering benefits significant enough?
3. **Smart defaults**: Too magical or appropriately helpful?
4. **Flag position**: Accept global flags must come first, or redesign?

### Next Steps

**If Adopted**:
1. Update CNP_Query_Grammar.md to reference this spec
2. Propagate to other Query tools (xfd, inx, dsc)
3. Update all user-facing documentation
4. Create migration guide for existing scripts
5. Benchmark performance improvements

**If Not Adopted**:
1. Revert RPT to positional parsing
2. Keep `-f`/`-d` flags but enforce position
3. Document as experimental feature

---

## 📚 References

- **CNP_Base_Grammar.md** - Foundation grammar rules
- **CNP_Query_Grammar.md** - v1 positional grammar (superseded for tree scope)
- **CNP_Reporting_Grammar.md** - Reporting tool extensions
- **CNP_Keyword_Routing.md** - Delegation rules for keywords

---

**Version**: 2.0-DRAFT
**Author**: CNP Ecosystem
**Status**: Pilot in RPT, awaiting feedback
**Last Updated**: 2025-11-14
**Feedback Deadline**: TBD
