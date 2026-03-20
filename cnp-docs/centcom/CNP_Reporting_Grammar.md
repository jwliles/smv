---
date_created: 2025-10-02T11-25-26
date_updated: 2025-11-14
timestamp: 1763183000000
title: CNP_Reporting_Grammar
id: 2f251282-3f32-46d1-be8e-32be499b60b7
hash: 7727cddd8f54ccc31d6513de5b31bd4be1a0d5afec71b8c7cc754b6c369889c9
---
# CNP Reporting Grammar Specification v1.0

**Purpose**: Grammar specification for reporting and diagnostics CNP tools
**Extends**: CNP-Base-Grammar.md, CNP-Query-Grammar.md
**Tools**: rpt (and similar reporting/analytics tools)
**Reviewed against**: CNP DSC/XFD grammars (2025-07)

> **⚠️ DRAFT EVOLUTION NOTICE (2025-11-14)**
>
> A new semantic phase-based execution model is being piloted in `rpt` v0.1.0. This model introduces:
> - **Position-independent keyword parsing** - Keywords work in any order
> - **Semantic phase execution** - Arguments executed by dependency order, not input position
> - **Type filter flags** (`-f` files-only, `-d` dirs-only) with early filtering
> - **Smart defaults** - Missing scope/target inferred from context
> - **`files` → `tree` rename** - More semantic subcommand name
>
> **See**: [CNP_Query_Grammar_v2_DRAFT.md](./CNP_Query_Grammar_v2_DRAFT.md) for complete specification.
>
> **Status**: DRAFT - Awaiting feedback before adoption
> - If adopted: This grammar will be updated to reflect the new model
> - If not adopted: RPT will revert to positional parsing
>
> Current grammar below describes the **v1 positional model** still in effect for non-tree scopes.

---

## 🎯 Reporting Grammar Philosophy

Reporting Grammar extends Query Grammar for tools that **aggregate, analyze, and present** information from various sources. This grammar maintains CNP consistency while adding reporting-specific capabilities:

- **Inherits core token rules** (uppercase keywords, positional parsing) from CNP-Base-Grammar
- **Follows Query Grammar structure** with reporting-specific extensions
- **Delegation transparency** for filters owned by other tools (SIZE→INX, HASH→CMP)
- **Multi-source integration** combining filesystem, system, and network data
- **Rich presentation formats** from simple trees to complex reports

---

## 🧱 Reporting Grammar Structure

```
rpt scope [filters] [modifiers] [routes] [flags]
```

### Positional Semantics (Simplified CNP Compatible)

| Segment | Description | Examples |
|---------|-------------|----------|
| `scope` | Single reporting domain | `sys`, `files`, `disk`, `net` |
| `filters` | Standard CNP filters (delegated as needed) | `EXT:rs`, `SIZE:>1MB`, `TYPE:file` |
| `modifiers` | Reporting-specific aggregation | `GROUP:ext`, `SORT:size`, `LIMIT:10` |
| `routes` | Universal CNP routes | `TO:smv`, `INTO:report.json`, `FORMAT:table` |
| `flags` | Universal and reporting flags | `-p`, `-r`, `-h`, `-v` |

**Default Behavior**: `rpt` without scope defaults to `rpt files` (current directory file reporting)

---

## 🗂️ Reporting Scopes

### Core Reporting Scopes (Single Domain Commands)

| Scope | Description | Data Sources | Example |
|-------|-------------|--------------|---------|
| `sys` | System information and metrics | CPUinfo, meminfo, processes | `rpt sys COLUMNS:memory,cpu,disk` |
| `files` | File and directory reporting | Filesystem traversal, metadata | `rpt files . GROUP:ext` |
| `disk` | Storage and disk usage | Mount points, disk space, I/O | `rpt disk SORT:usage` |
| `net` | Network interfaces and connectivity | Network interfaces, routing | `rpt net FORMAT:table` |

**Scope Behavior**: Each scope operates on its specific domain. `files` scope can take paths as targets, others operate on system-wide data.

---

## 🔧 Reporting Modifiers

### Aggregation Modifiers (CNP Standard)

| Modifier | Syntax | Meaning | Example |
|----------|--------|---------|---------|
| `GROUP:` | `GROUP:field` | Group results by field | `GROUP:ext`, `GROUP:size` |
| `SORT:` | `SORT:field` | Sort results by field | `SORT:modified`, `SORT:name` |
| `LIMIT:` | `LIMIT:N` | Limit number of results | `LIMIT:10`, `LIMIT:100` |

### Presentation Modifiers

| Modifier | Syntax | Meaning | Example |
|----------|--------|---------|---------|
| `COLUMNS:` | `COLUMNS:field1,field2` | Select output columns | `COLUMNS:name,size,modified` |
| `STYLE:` | `STYLE:format` | Presentation style | `STYLE:compact`, `STYLE:detailed` |

**Note on Operators**: All modifiers use `:` as canonical operator per CNP-Base-Grammar. Size/time comparisons use `SIZE:>1MB`, `MODIFIED:>1d` syntax.

---

## 🧩 Integration with Other Grammars

### Filter Integration & Delegation
Reporting tools accept all standard CNP filters but delegate processing per CNP-Keyword-Routing.md:

```bash
# Standard filters (delegated as needed)
rpt . NAME:config EXT:toml TYPE:file     # NAME, EXT, TYPE handled by rpt
rpt . SIZE:>1KB MODIFIED:>1d            # SIZE, MODIFIED delegated to inx
rpt . HASH:sha256 DUPLICATE:true        # HASH, DUPLICATE delegated to cmp

# Delegation transparency example
rpt . SIZE:>1MB
# [CNP] ⚠️ Detected out-of-scope keyword: SIZE
# [CNP] ⏩ Delegating SIZE filter to INX
# [CNP] 🧠 Equivalent pipeline: inx . SIZE:>1MB | rpt . [remaining filters]
```

### Delegation to Action Tools
Reports can delegate to Action Grammar tools:

```bash
# Report then act (delegation pipeline)
rpt . EXT:tmp SIZE:>100MB TO:smv rm -f
rpt system COLUMNS:disk FORMAT:json TO:alert THRESHOLD:90%
```

**Filter Ownership**: See CNP-Keyword-Routing.md for complete delegation matrix.

---

## 📤 Route Integration

Reporting tools MUST support Universal Routes from CNP-Base-Grammar:

### Standard Routes with Canonical Formats
```bash
# Canonical format options: json, csv, yaml, cnp, table, tree
rpt system FORMAT:json INTO:system-report.json
rpt . GROUP:ext FORMAT:table             # Table format for grouped data
rpt disk FORMAT:csv INTO:disk-usage.csv

# CNP format export
rpt . EXT:rs FORMAT:cnp INTO:file-report.cnp

# Tool delegation
rpt disk SORT:usage TO:alert
rpt . SIZE:>1GB TO:smv mv ./archive/
```

### Preview Support
```bash
# Universal preview flag
rpt system -p                           # Preview system report
rpt . GROUP:ext SORT:size -p            # Preview grouped file report
```

---

## 🚩 Universal Flags

All reporting tools MUST implement CNP Base Grammar universal flags:

```bash
# Universal flags (from CNP-Base-Grammar.md)
rpt -h                                   # Help and usage information
rpt -v                                   # Version information
rpt sys -p                              # Preview mode (dry-run, no output files)
rpt files . -r                          # Recursive processing (where applicable)
```

**Note**: The `-r` flag behavior depends on context:
- File operations: Recursively traverse directories
- System operations: Include detailed subsystem information
- Network operations: Include all interface details

---

## 🎯 Comprehensive Examples

### Basic Reporting
```bash
# System information
rpt sys                                 # All system info (default: yaml)
rpt sys COLUMNS:memory                  # Only memory info
rpt sys COLUMNS:cpu,memory,disk         # Specific columns

# File reporting (defaults to current directory)
rpt files                               # Current directory report
rpt files ./src EXT:rs                  # Rust files only
rpt files ./logs MODIFIED:>7d SIZE:<1MB # Recent small files
```

### Advanced Usage with Delegation
```bash
# Size filtering delegates to INX
rpt files . SIZE:>1MB EXT:log,tmp FORMAT:json INTO:large-files.json
# [CNP] ⏩ Delegating SIZE filter to INX
# [CNP] 🧠 Pipeline: inx . SIZE:>1MB | rpt files . EXT:log,tmp FORMAT:json INTO:large-files.json

# Hash operations delegate to CMP
rpt files . HASH:sha256 DUPLICATE:true FORMAT:table
# [CNP] ⏩ Delegating HASH,DUPLICATE to CMP
# [CNP] 🧠 Pipeline: cmp . HASH:sha256 DUPLICATE:true | rpt files . FORMAT:table

# Complex delegation chain
rpt files . SIZE:>100MB EXT:tmp TO:smv rm -f
# Step 1: SIZE delegated to INX
# Step 2: Results passed to RPT for EXT filtering
# Step 3: Final results delegated to SMV for removal
```

### Format Examples
```bash
# Canonical format support (aligned with DSC/XFD)
rpt sys FORMAT:json                     # JSON output
rpt files . GROUP:ext FORMAT:csv INTO:files.csv  # CSV with grouping
rpt disk FORMAT:table                   # Table format for disk usage
rpt files . FORMAT:cnp INTO:snapshot.cnp # CNP format for tool chaining

# Tree format (filesystem context only)
rpt files . FORMAT:tree                 # Tree view of directory structure
```

### Integration Examples
```bash
# Preview mode
rpt sys -p                              # Preview system report without output
rpt files . SIZE:>1GB -p TO:archive     # Preview what would be archived

# Recursive processing
rpt files ./project -r EXT:rs MODIFIED:>1d # Recursive Rust file analysis
rpt sys -r COLUMNS:network              # Detailed network interface info

# Cross-tool workflows
rpt disk USAGE:>80% TO:alert SEVERITY:high
rpt files . MODIFIED:>30d FORMAT:cnp TO:cleanup-tool
rpt sys COLUMNS:memory FORMAT:json | jq '.memory.available_gb'
```

### Real-World Scenarios
```bash
# Development workflow
rpt files ./src -r EXT:rs SIZE:>10KB MODIFIED:>7d FORMAT:table
# Large Rust files modified recently

# System monitoring
rpt sys COLUMNS:cpu,memory,disk FORMAT:json INTO:metrics.json
# Capture key metrics for monitoring dashboard

# Cleanup preparation
rpt files ./temp SIZE:>100MB MODIFIED:>30d FORMAT:cnp TO:cleanup-tool
# Find large old files and delegate to cleanup tool

# Security audit
rpt files . -r HASH:sha256 FORMAT:csv INTO:file-hashes.csv
# Generate hash inventory (delegated to CMP)
```

---

## 🎛️ Reporting Grammar Flags

### Universal Reporting Flags

| Flag | Long Form | Description | Required |
|------|-----------|-------------|----------|
| `-w` | `--watch` | Continuous monitoring mode | Optional |
| `-i` | `--interactive` | Interactive report navigation | Optional |
| `--no-color` | `--no-color` | Disable colored output | Optional |

### Scope-Specific Flags

| Scope | Flag | Description |
|-------|------|-------------|
| `system` | `--live` | Real-time system metrics |
| `files` | `--hidden` | Include hidden files |
| `performance` | `--baseline` | Compare against baseline |

---

## ⚠️ Reporting Grammar Rules

### Scope Resolution
1. **Scope is required** - no default reporting domain
2. **Scope determines available modifiers** - not all modifiers work with all scopes
3. **Invalid scope/modifier combinations** produce helpful errors
4. **Scope-specific help** available via `rpt <scope> -h`

### Data Integration
- **Multiple data sources** may be combined within a scope
- **Inconsistent data** should be handled gracefully
- **Missing data sources** should not prevent partial reports
- **Performance considerations** for large datasets

### Output Consistency
- **Tabular data** should align columns properly
- **Tree structures** should use consistent indentation
- **Time-series data** should use consistent time formats
- **Size units** should be human-readable by default

---

## 🔗 Related Specifications

- **CNP-Base-Grammar.md**: Foundation that this extends
- **CNP-Query-Grammar.md**: Source of filter syntax
- **CNP-Action-Grammar.md**: Delegation target for actions
- **CNP-File-Format.md**: Export format integration

---

## 📝 Grammar Extensions

### Adding New Reporting Scopes

New reporting scopes SHOULD:
1. **Represent a coherent domain** of information
2. **Have clear data sources** and aggregation methods
3. **Support relevant modifiers** for that domain
4. **Integrate well** with existing scopes and tools
5. **Document performance characteristics**

### Modifier Extension Guidelines
- **Use UPPERCASE** for new modifier keywords
- **Follow existing syntax patterns** (`:`, `>`, `<`)
- **Be scope-aware** - not all modifiers apply to all scopes
- **Document aggregation behavior** clearly
- **Test with large datasets**

---

**Version**: 1.0  
**Author**: CNP Ecosystem  
**Last Updated**: 2025-07-26  
**Next Review**: Before rpt v1.0.0 release
