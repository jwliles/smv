# CNP Query Grammar Specification v1.0

**Purpose**: Grammar specification for query-oriented CNP tools  
**Extends**: CNP-Base-Grammar.md  
**Tools**: dsc, xfd, skl, inx (and similar discovery/analysis tools)

---

## 🎯 Query Grammar Philosophy

Query Grammar is designed for tools that **discover and analyze** files and directories. This grammar uses a declarative "what you want" approach optimized for:

- **Expressive filtering** through declarative syntax
- **Minimal flag usage** (filters handle most customization)
- **Composable queries** that can be chained and modified
- **Fast discovery** with index-aware optimizations

---

## 🧱 Query Grammar Structure

```
<tool> <path> <filters> <routes> <flags>
```

### Positional Semantics

| Segment | Description | Examples |
|---------|-------------|----------|
| `path` | Starting point for operation (default: `.`) | `./src`, `~/docs`, `/var/log` |
| `filters` | UPPERCASE declarative filters | `EXT:rs`, `SIZE>1MB`, `NAME:config` |
| `routes` | Output directives | `TO:smv`, `INTO:file.txt`, `FORMAT:json` |
| `flags` | Minimal modifiers | `-p`, `-h`, `-r` |

All segments are parsed **left-to-right**. Positional clarity is **mandatory**.

---

## 🧠 Query Grammar Filters

### Declarative Filter Syntax
```
KEYWORD:value
KEYWORD>value  
KEYWORD<value
```

### Core Filter Keywords

| Filter | Syntax | Meaning | Example |
|--------|--------|---------|---------|
| `NAME:` | `NAME:pattern` | File/directory name matching | `NAME:config` |
| `EXT:` | `EXT:extension` | File extension filter | `EXT:rs` |
| `TYPE:` | `TYPE:type` | File type (`file`, `dir`, `symlink`) | `TYPE:file` |
| `MORE:` | `MORE:value` | File size greater than | `MORE:1MB` |
| `LESS:` | `LESS:value` | File size less than | `LESS:100KB` |
| `DEEPER:` | `DEEPER:N` | Directory depth greater than | `DEEPER:3` |
| `SHALLOWER:` | `SHALLOWER:N` | Directory depth less than | `SHALLOWER:2` |
| `NEWER:` | `NEWER:date` | Modified after date | `NEWER:2024-01-01` |
| `OLDER:` | `OLDER:date` | Modified before date | `OLDER:2023-12-31` |

### Advanced Filter Keywords

| Filter | Syntax | Meaning | Example |
|--------|--------|---------|---------|
| `WHERE:` | `WHERE:expression` | Groups filters logically | `WHERE:(EXT:rs OR EXT:md)` |
| `FOR:` | `FOR:semantic` | Shorthand semantic filters | `FOR:scripts`, `FOR:configs` |
| `CONTAINS:` | `CONTAINS:text` | Content search (when supported) | `CONTAINS:TODO` |
| `HASH:` | `HASH:value` | Content hash matching | `HASH:sha256:abc123` |

### Filter Value Standards
- **Size Units**: `B`, `KB`, `MB`, `GB` (1024-based)
- **Time Formats**: ISO 8601 (`YYYY-MM-DD`) or relative (`1h`, `30m`, `2d`)
- **Patterns**: Support glob (`*.txt`) and regex based on tool capability
- **Boolean Logic**: `AND`, `OR`, `NOT` for complex expressions

---

## 🎛️ Query Grammar Flags

Query Grammar uses **minimal flags** since filters handle most customization:

### Universal Query Flags

| Flag | Long Form | Description | Required |
|------|-----------|-------------|----------|
| `-p` | `--paths` | Output paths only (no metadata) | ✅ Yes |
| `-r` | `--recursive` | Include subdirectories | ✅ Yes |

### Optional Query Flags

| Flag | Long Form | Description | Tool-Specific |
|------|-----------|-------------|---------------|
| `-n` | `--limit` | Limit number of results | Optional |
| `-s` | `--sort` | Sort output by criteria | Optional |
| `-j` | `--json` | JSON output format | Optional |
| `-c` | `--count` | Show count only | Optional |

### Flag Stacking
```bash
# Minimal stacking - order usually doesn't matter
dsc . EXT:rs -pr     # Paths, recursive
xfd . NAME:config -n 10  # With limit
```

---

## 🧩 Integration with Action Grammar

Query tools can delegate to Action Grammar tools via routes:

### Delegation Examples
```bash
# Find then act
dsc . EXT:tmp TO:smv rm -f
xfd . SIZE>100MB TO:smv mv ./archive/ -p

# Query then process  
skl . NAME:backup TO:arc compress -r
```

### Filter Embedding in Action Tools
Action tools can embed Query filters:
```bash
# Action tool using Query syntax
smv rm . EXT:log TYPE:file -rp
edt . FIND:old REPLACE:new EXT:rs -a
```

---

## 📤 Route Integration

Query tools MUST support Universal Routes from CNP-Base-Grammar:

### Standard Routes
```bash
# File output
dsc . EXT:rs INTO:rust-files.txt

# Format conversion
xfd . SIZE>1MB FORMAT:json INTO:large-files.json

# Tool delegation
skl . FOR:configs TO:edt backup
```

### Chain Examples
```bash
# Multi-step workflows
dsc . EXT:log TO:xfd CONTAINS:ERROR TO:smv mv ./logs/errors/
```

---

## 🧠 Command Examples by Tool

### dsc (Directory Statistics)
```bash
# Basic usage
dsc .                           # Show directory statistics
dsc . -p                        # List all paths
dsc ./src EXT:rs -p             # List Rust files

# Complex filtering
dsc . EXT:rs SIZE>10KB -p
dsc /var/log TYPE:file MODIFIED>7d -p
dsc . FOR:configs NAME:local -r
```

### xfd (Content Search)
```bash
# Content search
xfd . CONTAINS:TODO EXT:rs -p
xfd ./docs CONTAINS:"broken link" -n 5

# Pattern matching  
xfd . NAME:test* EXT:js -r
xfd . MODIFIED>1d CONTAINS:error -p
```

### skl (Fuzzy Finding)
```bash
# Fuzzy finding
skl config                      # Find files matching "config"
skl . EXT:md CONTAINS:readme -p
skl ./src NAME:util* TYPE:file

# Advanced queries
skl . WHERE:(EXT:js OR EXT:ts) SIZE<100KB
```

### inx (Indexing)
```bash
# Index operations
inx scan .                      # Scan current directory
inx . MODIFIED>1h -p           # Recently modified files
inx status                     # Index status

# Query index
inx . TYPE:file SIZE>1GB -p
inx ./src EXT:rs DEPTH<3
```

---

## 🔍 Query Optimization

### Index Awareness
Query Grammar tools SHOULD:
- **Use index data** when available (via inx)
- **Fall back to live scanning** when index is stale
- **Update index** opportunistically during queries
- **Prefer indexed metadata** for performance

### Performance Guidelines
- **Filter early** - apply fastest filters first
- **Limit results** - use `-n` for large result sets
- **Stream output** - don't buffer entire result sets
- **Cache filters** - compile regex/glob patterns once

---

## ⚠️ Query Grammar Rules

### Filter Processing Order
1. **Path resolution** and validation
2. **Type and extension** filters (fastest)
3. **Size and time** filters (indexed metadata)
4. **Name and content** filters (potentially expensive)
5. **Route processing** (delegation/output)

### Error Handling
- **Invalid filters** - suggest corrections
- **Missing paths** - clear error messages
- **Permission errors** - skip with warnings
- **Empty results** - explain why (too restrictive filters)

### Safety Considerations
- **Large directories** - warn about performance
- **Recursive operations** - respect system limits
- **Content search** - limit file size scanning
- **Result limits** - prevent memory exhaustion

---

## 🔗 Related Specifications

- **CNP-Base-Grammar.md**: Foundation that this extends
- **CNP-Action-Grammar.md**: Complementary imperative grammar
- **CNP-Keyword-Routing.md**: Delegation to Action tools

---

## 📝 Grammar Extensions

### Adding New Query Tools

New query-oriented tools SHOULD:
1. **Follow the path → filters → routes → flags structure**
2. **Implement core filter keywords** consistently
3. **Support Universal Routes** for delegation
4. **Minimize flag usage** in favor of filters
5. **Document tool-specific filter extensions**

### Filter Extension Guidelines
- **Use UPPERCASE** for new filter keywords
- **Follow existing syntax patterns** (`:`, `>`, `<`)
- **Avoid conflicts** with universal keywords
- **Document performance characteristics**
- **Test with complex combinations**

---

**Version**: 1.0  
**Converted from**: CNP-Filter-Grammar.md  
**Last Updated**: 2025-07-13
