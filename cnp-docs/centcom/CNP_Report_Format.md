---
date_created: 2025-10-18T22-02-58
date_updated: 2025-10-18T22-16-53
timestamp: 1760824978588
title: CNP_Report_Format
id: 1bed405e-fd51-4c34-bd3e-6be6ba3c2c3d
hash: 291809e40690528b3fac6c5233643b59949338fbbec97605735813af50f381c2
---
# CNP Report Format Specification

**Purpose**: Define the human-readable report output format for CNP reporting tools

**Extends**: CNP_Reporting_Grammar.md (adds `report` to canonical format options)

**Applies to**: `rpt`, and other tools that generate structured reports for human consumption

**Related Specifications**: CNP_Base_Grammar.md, CNP_Reporting_Grammar.md, CNP_Tree_Syntax.md

---

## 🎯 Overview

The report format is designed for human readability first, providing structured, well-formatted output for terminal display. Unlike `table` (Markdown), `json`, `tree`, or `debug` formats, `report` is optimized for:

- **Quick scanning** - Clear visual hierarchy with sections and separators
- **Context preservation** - Titles, summaries, and metadata inline
- **Terminal-friendly** - Dynamic width formatting, optional color support
- **Consistency** - Predictable structure across different data types
- **Rich context** - Supports multi-section reports with summaries and aggregations

---

## 📦 Format Keyword Integration

Per CNP_Base_Grammar.md, the `FORMAT:` route accepts canonical format types. The `report` format extends this list:

**Updated Canonical Formats**:
`json`, `csv`, `yaml`, `cnp`, `table`, `tree`, `report`

### Format Selection Syntax

```bash
rpt sys FORMAT:report                    # Explicit selection
rpt sys FORMAT:report INTO:system.txt    # With file output
rpt disk                                 # Implicit (uses scope default)
```

---

## 📊 Format Comparison Matrix

Understanding when to use each format:

| Format   | Purpose              | Structure        | Metadata | Summaries | Export | Parseable | Terminal-Optimized |
|----------|---------------------|------------------|----------|-----------|--------|-----------|-------------------|
| `report` | Terminal analysis   | Multi-section    | ✓✓✓      | ✓✓✓       | ✗      | ✗         | ✓✓✓               |
| `table`  | Documentation       | Markdown table   | ✓✓       | ✗         | ✓✓     | ✓✓        | ✓                 |
| `tree`   | Hierarchies         | Indented tree    | ✓        | ✗         | ✗      | ✓         | ✓✓                |
| `json`   | Machine processing  | Structured JSON  | ✓✓✓      | ✗         | ✓✓✓    | ✓✓✓       | ✗                 |
| `yaml`   | Config/readable     | Structured YAML  | ✓✓✓      | ✗         | ✓✓     | ✓✓✓       | ✗                 |
| `csv`    | Spreadsheet export  | Comma-separated  | ✓✓       | ✗         | ✓✓✓    | ✓✓✓       | ✗                 |
| `cnp`    | Tool interchange    | CNP file format  | ✓✓✓      | ✓         | ✓✓✓    | ✓✓✓       | ✗                 |

### When to Use Each Format

**report**: Interactive terminal sessions requiring quick scanning and context
- Default for: `rpt sys`, `rpt disk`, `rpt net`
- Use when: Humans need to quickly understand system state or analyze data
- Example: `rpt sys FORMAT:report`

**table**: Documentation, README files, Markdown content
- Default for: None (explicit only)
- Use when: Output needs to be embedded in documentation
- Example: `rpt disk FORMAT:table INTO:docs/storage.md`

**tree**: Hierarchical file/directory structures
- Default for: `rpt files` (when showing directory structure via `dsc` or `look`)
- Use when: Displaying filesystem hierarchies or nested structures
- Example: `rpt files . FORMAT:tree`

**json/yaml/csv**: Automation, scripting, data pipelines
- Default for: Programmatic queries
- Use when: Output will be processed by other tools or scripts
- Example: `rpt sys FORMAT:json | jq '.cpu.cores'`

**cnp**: Cross-tool workflows and session capture
- Default for: Delegation and replay scenarios
- Use when: Capturing entire command context for replay or tool chaining
- Example: `rpt sys FORMAT:cnp TO:monitoring-tool`

---

## 🧱 Structure

### Basic Report Layout

```
## REPORT TITLE
[optional metadata line]

Column1        Column2        Column3        Column4
value1         value2         value3         value4
value1         value2         value3         value4
------------------------------------------------------------

[optional summary section]
Total: X items
Summary metric: Y
```

### Multi-Section Reports

```
## MAIN REPORT TITLE

### Section 1
Data for section 1
------------------------------------------------------------

### Section 2
Data for section 2
------------------------------------------------------------

## SUMMARY
Aggregated totals and key metrics
```

---

## 📝 Formatting Rules

### Headers

- **Top-level**: `## UPPERCASE TITLE`
- **Sub-sections**: `### Title Case`
- Followed by blank line before data

### Separators

**Width Calculation**:
1. Detect terminal width via `$COLUMNS` or `tput cols`
2. Default to 60 characters if detection fails
3. Minimum: 40 characters (for narrow terminals)
4. Maximum: terminal width - 2 (leave margins)

**Rendering**:
```bash
# Dynamic separator (default 60 chars)
------------------------------------------------------------

# Terminal-aware (80 columns detected)
--------------------------------------------------------------------------------

# Narrow terminal (40 chars minimum)
----------------------------------------
```

**Usage Rules**:
- Separate logical sections, not individual rows
- End sections with separator when followed by more content
- Use consistent separator width throughout a single report

### Columns

- **Left-aligned** for text/paths
- **Right-aligned** for numbers
- **Consistent width** within each column
- **Minimum padding**: 2 spaces between columns

### Column Alignment Edge Cases

**Mixed-Type Column Handling**:
When a column contains both text and numeric values:

```text
Type           Value
String         "config.txt"
Number         42
String         "data.json"
Number         1337
```

**Rule**: Determine column type by majority. If ≥60% of values are numeric, right-align the entire column.

**Tiebreaker**: For exactly 50/50 splits, prefer **left-align** (text default) to avoid misalignment of mixed content.

**Zero and Null Values**:
```text
Size        Status
1.2MB       Active
0B          Empty
--          Unknown
```

- Zero values: Display as "0B" (with appropriate unit)
- Null/missing: Display as "--" (em dash, U+2014 in UTF-8 terminals)
  - **Fallback**: Use ASCII "--" (two hyphens) if UTF-8 not supported
  - Detection: Check `$LANG` or `$LC_ALL` environment variables
- Empty strings: Display as "(empty)" if meaningful, otherwise omit row

### Data Values

- **Sizes**: Human-readable with units (KB, MB, GB, TB)
- **Percentages**: One decimal place (15.6%)
- **Paths**: Shortened with ~ for home, ./ for relative
- **Timestamps**: ISO 8601 or relative ("2h ago")
- **Boolean**: "Yes"/"No" or checkmarks (✓/✗) if color-enabled

### Summary Sections

- Appear after main data
- Include totals, averages, or key insights
- May be indented or prefixed with summary marker

---

## 🎨 Visual Examples

### Disk Usage Report

```text
## DISK USAGE REPORT
Scanned: 2025-10-18 13:45:23

Filesystem         Mount Point                        Size     Used    Avail   Use%
/dev/nvme1n1p2     /                                 914GB    143GB    724GB    16%
/dev/nvme1n1p1     /efi                                2GB    129MB      2GB     6%
tmpfs              /tmp                               16GB    139MB     16GB     1%
tmpfs              /run/user/1000                      3GB    104KB      3GB     0%
------------------------------------------------------------

Total Physical: 916GB
Total Used: 143GB (15.6%)
Available: 726GB
```

### System Information Report

```text
## SYSTEM INFORMATION

### Hardware
CPU: AMD Ryzen 9 5950X (32 cores)
Memory: 64GB (48GB used, 16GB free)
Disk: 916GB total, 143GB used

### Software
OS: Arch Linux
Kernel: 6.17.2-arch1-1
Uptime: 12 days, 4 hours
------------------------------------------------------------

Load Average: 2.4, 2.1, 1.8
Running Processes: 342
```

### File Statistics Report

```text
## FILE STATISTICS REPORT
Path: /home/user/projects
Scanned: 2,847 files in 156 directories

### By Extension
Extension      Count      Total Size      Avg Size
.rs            1,243      45.2MB          37KB
.toml          89         234KB           3KB
.md            234        5.6MB           24KB
.json          45         890KB           20KB
------------------------------------------------------------

### By Size Range
Range          Count      Percentage
< 1KB          456        16.0%
1KB - 10KB     1,234      43.3%
10KB - 100KB   987        34.7%
> 100KB        170        6.0%
------------------------------------------------------------

Total Files: 2,847
Total Size: 51.9MB
Largest File: src/parser.rs (456KB)
```

---

## 🔗 Displaying Delegated Filter Results

When reporting tools receive delegated filters (per CNP_Keyword_Routing.md), the report format displays integrated results with clear attribution:

### Delegation in Report Headers

```bash
# Command with delegated filter
rpt files . SIZE:>1MB FORMAT:report
```

**Output**:
```text
## FILE REPORT
Filters: SIZE > 1MB (delegated to INX)
Scanned: 2025-10-18 14:23:45

Path                    Size        Modified
src/large-file.rs      2.3MB       2025-10-15
assets/video.mp4       45.8MB      2025-10-12
------------------------------------------------------------

Total: 2 files
Combined Size: 48.1MB
```

### Multi-Delegation Example

```bash
# Multiple delegated filters
rpt files . SIZE:>1MB HASH:sha256 DUPLICATE:true FORMAT:report
```

**Output**:
```text
## FILE REPORT
Filters Applied:
  - SIZE > 1MB (delegated to INX)
  - HASH: sha256, DUPLICATE: true (delegated to CMP)
Generated: 2025-10-18 14:25:12

Path                    Size        Hash (first 8)    Duplicates
src/large-file.rs      2.3MB       a1b2c3d4          3 copies
assets/video.mp4       45.8MB      e5f6g7h8          2 copies
------------------------------------------------------------

Duplicate Groups:
  Group 1: src/large-file.rs, backup/large-file.rs, old/large-file.rs
  Group 2: assets/video.mp4, media/video-copy.mp4
------------------------------------------------------------

Total: 2 unique files, 5 total copies
Combined Size: 48.1MB
Wasted Space: 25.4MB (duplicates)
```

### Transparency Requirements

1. Always show which filters were delegated in the report header
2. Include tool names for delegated operations
3. Display combined results in a unified format
4. Maintain section separators between different data types

---

## 🎯 Default Format by Reporting Scope

Per CNP_Reporting_Grammar.md, each scope has an appropriate default format:

| Scope   | Default Format | Rationale                                    | Override Example                |
|---------|----------------|----------------------------------------------|---------------------------------|
| `sys`   | `report`       | Rich context for system analysis             | `rpt sys FORMAT:json`           |
| `files` | `tree`         | Hierarchical structure is primary            | `rpt files . FORMAT:report`     |
| `disk`  | `report`       | Summary statistics are essential             | `rpt disk FORMAT:table`         |
| `net`   | `report`       | Multi-interface context matters              | `rpt net FORMAT:yaml`           |

### Format Selection Logic

```bash
# Uses scope default (report)
rpt sys

# Explicit override
rpt sys FORMAT:table

# Context-sensitive selection
rpt files .           # Uses tree (default for files scope)
rpt files . GROUP:ext # Uses report (aggregation implies tabular)
```

### When Report is the Default

The report format SHOULD be default for:
- System information (`rpt sys`)
- Disk usage (`rpt disk`)
- Network information (`rpt net`)
- Environment variables (`rpt env`)
- Any command that returns structured tabular data for human review

### When Other Formats are Default

- **File listings**: Use `tree` format (hierarchical structure)
- **Export operations**: Require explicit `FORMAT:` specification
- **Machine consumption**: Require explicit `FORMAT:json` or `FORMAT:csv`

---

## 📤 Route Integration Examples

### Standard Routes with Report Format

**INTO: File Output**
```bash
rpt sys FORMAT:report INTO:system-snapshot.txt
# Saves formatted report to file
```

**TO: Tool Delegation**
```bash
# Report generation → Action tool
rpt disk USAGE:>80% FORMAT:report TO:alert SEVERITY:high

# Query → Report → Action chain
xfd . SIZE:>1GB | rpt files . FORMAT:report | smv mv ./archive/
```

**FORMAT: Chain with Other Formats**
```bash
# Generate report, also export JSON for automation
rpt sys FORMAT:report INTO:human.txt
rpt sys FORMAT:json INTO:machine.json

# Convert CNP format to report
cnp --from-cnp=data.cnp FORMAT:report
```

### Preview Mode Integration

```bash
# Universal -p flag with report format
rpt sys -p                              # Preview report (no file output)
rpt disk FORMAT:report -p               # Same as above
rpt files . SIZE:>1GB FORMAT:report -p  # Preview before action
```

### Complex Workflow Examples

```bash
# Multi-step with different formats
rpt sys FORMAT:json | \
  jq '.memory.available_gb < 2' | \
  rpt disk FORMAT:report TO:alert

# Session capture with report display
rpt sys FORMAT:report | tee session.log
rpt sys FORMAT:cnp INTO:session.cnp     # Also save as CNP for replay

# Pipe through pager
rpt disks FORMAT:report | less

# Search formatted output
rpt sys FORMAT:report | grep CPU

# Convert JSON to report for human review
rpt env FORMAT:json | rpt FORMAT:report
```

---

## 🔧 Implementation Guidelines

### Column Width Calculation

1. Measure maximum width needed for each column
2. Add 2-space minimum padding between columns
3. Right-align numeric columns
4. Truncate paths/strings with ... if needed for terminal width

**Truncation Strategy**:
- **Paths**: Middle truncation with ellipsis (`/home/.../project/file.txt`)
  - **Algorithm**:
    - Preserve first 15% and last 40% of path
    - Minimum preserved: First 10 chars, last 20 chars
    - Replace middle with `...` (3 chars)
    - Example: `/very/long/path/to/deeply/nested/project/file.txt` (52 chars)
      → `/very/long.../file.txt` (when width = 30)
- **Text**: End truncation (`This is a long text stri...`)
  - Preserve as much as possible from start
  - Add `...` at truncation point
- **Priority**: Preserve critical columns (names, values) over metadata columns

### Unit Conversion

**Bytes → Human-readable**:
- < 1KB: show bytes (e.g., `512B`)
- < 1MB: show KB with 1 decimal (e.g., `45.2KB`)
- < 1GB: show MB with 1 decimal (e.g., `128.5MB`)
- ≥ 1GB: show GB with 1 decimal (e.g., `2.3GB`)
- ≥ 1TB: show TB with 1 decimal (e.g., `1.5TB`)

**Edge Cases**:
- Negative numbers: Display with sign (`-5.2GB`)
- Zero values: `0B` (not `0KB`)
- Very large: Auto-scale to appropriate unit (>1000GB → TB)

**Other Units**:
- Percentages: Always 1 decimal place (e.g., `15.6%`)
- Time durations: Use most appropriate unit (seconds, minutes, hours, days)

### Terminal Width Handling

- Detect terminal width via `$COLUMNS` environment variable or `tput cols`
- Default to 80 columns if not detected
- Intelligently wrap or truncate if content exceeds width
- Preserve critical columns (names, values) over metadata columns
- Minimum viable width: 40 characters (gracefully degrade below this)

### Character Encoding Detection

**UTF-8 Support**:
- Check `$LANG`, `$LC_ALL`, or `$LC_CTYPE` for UTF-8 encoding
- Use Unicode characters (em dash, checkmarks, box drawing) only if UTF-8 detected
- **Fallback to ASCII**: Replace Unicode with ASCII equivalents:
  - `—` (em dash) → `--` (two hyphens)
  - `✓` (checkmark) → `Y` or `*`
  - `✗` (cross) → `N` or `-`
  - Box drawing → `+`, `-`, `|` characters

**Detection Algorithm**:
```rust
fn supports_utf8() -> bool {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_CTYPE"))
        .or_else(|_| std::env::var("LANG"))
        .map(|locale| locale.contains("UTF-8") || locale.contains("utf8"))
        .unwrap_or(false)
}
```

### Color Support (Optional)

**Detection Priority**:
1. `--no-color` flag (explicit override)
2. `NO_COLOR` environment variable
3. TTY detection (`isatty()`)
4. Default OFF for pipes and redirects

**ANSI Color Codes**:
- Headers: Bold (`\033[1m`)
- Warnings: Yellow (`\033[33m`)
- Errors: Red (`\033[31m`)
- Success: Green (`\033[32m`)
- Dim/metadata: Gray (`\033[90m`)

**Graceful Degradation**:
- Automatically disable if non-TTY output detected
- Preserve all information without color codes
- Use alternative indicators (symbols) when color unavailable

---

## ⚠️ Error and Edge Case Handling

### Empty Result Sets

```text
## DISK USAGE REPORT
Scanned: 2025-10-18 14:30:00

No disks found matching criteria.
```

### Single-Row Results

```text
## SYSTEM INFORMATION
CPU: AMD Ryzen 9 5950X (32 cores)
Memory: 64GB (48GB used, 16GB free)
------------------------------------------------------------
```

**Note**: Separator still included for format consistency.

### Data Overflow/Truncation

```text
## FILE REPORT
Path: /very/long/path/that/exceeds/term...
Showing 100 of 1,247 files (use LIMIT: to adjust)
------------------------------------------------------------
```

### Permission/Access Errors

```text
## DISK USAGE REPORT
⚠ Warning: Could not access /restricted/path (Permission denied)

Accessible disks shown below:
------------------------------------------------------------
```

### Partial Failures

```text
## SYSTEM INFORMATION

### Hardware
CPU: AMD Ryzen 9 5950X (32 cores)
Memory: 64GB (48GB used)
⚠ Disk info unavailable (service not running)
------------------------------------------------------------
```

### Minimum Width Graceful Degradation

When terminal width < 40 characters:
```text
## DISK REPORT
(Terminal too narrow
for report format)

Suggestions:
• Resize terminal
• Use FORMAT:json
• Use FORMAT:csv
• Try --compact mode
```

**Recommendation**: Report format requires minimum 40-character width. For narrow terminals or pipes, use:
- `FORMAT:json | jq` for processing
- `FORMAT:csv` for spreadsheet import
- `--compact` flag (if supported by tool) for denser output

---

## ⚙️ Constraints

1. **Not for Piping**: Report format includes headers/separators unsuitable for pipelines. Use `FORMAT:json` or `FORMAT:csv` for piping.

2. **Fixed Width Assumption**: Assumes terminal display, not suitable for variable-width parsing

3. **Locale-Dependent Formatting**: Number and date formatting respects system locale
   - **Decimal separators**: May use `.` (en_US) or `,` (de_DE) based on `$LC_NUMERIC`
   - **Thousands separators**: May use `,` or `.` or space depending on locale
   - **Date formats**: ISO 8601 recommended for consistency, but may vary
   - **Note**: Tools should allow `--locale=C` to force POSIX/ASCII formatting

4. **No Schema**: Structure varies by data type, not programmatically parseable

5. **Terminal Output Only**: Optimized for TTY display, not for file processing

6. **Performance Considerations**:
   - Reports with >10,000 rows may be slow to render
   - Consider using `LIMIT:N` modifier to cap results
   - Tools SHOULD warn when generating reports >1,000 rows
   - Tools MAY implement pagination for large datasets
   - Example: `rpt files . -r | head -n 1000` for manual limiting

7. **Accessibility**:
   - Report format is **screen reader friendly** (linear text structure)
   - Better for accessibility than `tree` format (which uses visual indentation)
   - Avoid Unicode box-drawing characters in accessibility mode
   - Use `--plain` or `--ascii` flags to ensure compatibility
   - Headers provide context for assistive technologies

**For machine consumption, always use `FORMAT:json`, `FORMAT:csv`, or `FORMAT:cnp`.**

---

## 🔄 Integration with CNP Grammar

### With CNP Reporting Grammar

```bash
rpt disks FORMAT:report           # Explicit (same as default)
rpt sys FORMAT:report INTO:out.txt  # Save formatted report
rpt net -p FORMAT:report          # Preview with report format
```

### With Other Tools

```bash
rpt disks FORMAT:report | less    # Page through report
rpt sys FORMAT:report | grep CPU  # Search formatted output
rpt env FORMAT:json | rpt FORMAT:report  # Convert JSON to report
```

### With Universal Routes

```bash
# Delegation chain
xfd . EXT:log SIZE:>100MB TO:rpt FORMAT:report

# Export and delegate
rpt sys FORMAT:report INTO:report.txt TO:alert

# Format conversion pipeline
rpt sys FORMAT:cnp | rpt FORMAT:report
```

---

## 📋 Required Updates to CNP_Reporting_Grammar.md

To fully integrate this specification, the CNP_Reporting_Grammar.md file requires the following updates:

### 1. Update Canonical Format List (Line ~107)

**Before**:
```yaml
# Canonical format options: json, csv, yaml, cnp, table, tree
```

**After**:
```yaml
# Canonical format options: json, csv, yaml, cnp, table, tree, report
```

### 2. Add Format Specifications Section (After line ~110)

Insert new section documenting format purposes and characteristics:

```markdown
### Format Specifications

| Format   | Purpose              | Specification                     |
|----------|---------------------|-----------------------------------|
| report   | Terminal analysis    | CNP_Report_Format.md              |
| table    | Markdown tables      | (inline spec in grammar)          |
| tree     | Hierarchies          | CNP_Tree_Syntax.md                |
| json     | Machine processing   | Standard JSON                     |
| yaml     | Config files         | Standard YAML                     |
| csv      | Spreadsheet export   | RFC 4180                          |
| cnp      | Tool interchange     | CNP_File_Format.md                |
```

### 3. Update Reporting Scopes Section

Add default format column to scope definitions:

```markdown
| Scope | Description | Data Sources | Default Format | Example |
|-------|-------------|--------------|----------------|---------|
| `sys` | System information and metrics | CPUinfo, meminfo, processes | `report` | `rpt sys` |
| `files` | File and directory reporting | Filesystem traversal, metadata | `tree` | `rpt files .` |
| `disk` | Storage and disk usage | Mount points, disk space, I/O | `report` | `rpt disk` |
| `net` | Network interfaces and connectivity | Network interfaces, routing | `report` | `rpt net` |
```

### 4. Add Delegation Display Notes (Integration Section)

Document how delegated filters appear in report format output:

```markdown
### Report Format Delegation Display

When using `FORMAT:report` with delegated filters, the report header clearly
indicates which filters were handled by other tools:

```text
## FILE REPORT
Filters: SIZE > 1MB (delegated to INX), HASH: sha256 (delegated to CMP)
Generated: 2025-10-18 14:25:12
...
```

This transparency ensures users understand the complete processing pipeline.
```

---

## 🧠 Future Extensions

### Planned Enhancements

- **Color themes**: User-configurable color schemes via config file
- **Compact mode**: Denser output for small terminals (`--compact`)
- **Detail levels**: `--detail=minimal|normal|verbose` for granularity control
- **Chart support**: Simple ASCII bar charts for distributions
- **Grouping**: Automatic grouping of similar rows
- **Watch mode**: Live-updating reports with `-w` flag
- **Diff reports**: Before/after comparison view
- **Export variants**: `FORMAT:report-markdown` for documentation

### Consideration for v2.0

- **Streaming mode**: Progressive output for large datasets
- **Custom templates**: User-defined report layouts
- **Multi-column layout**: Side-by-side section display for wide terminals
- **Interactive mode**: Navigate large reports with keyboard controls
- **Bookmark support**: Mark and return to specific report sections

---

## 📚 Examples by Use Case

### Development Workflow

```bash
# Find large Rust files modified recently
rpt files ./src -r EXT:rs SIZE:>10KB MODIFIED:>7d FORMAT:report
```

**Output**:
```text
## FILE REPORT
Path: ./src
Filters: EXT:rs, SIZE > 10KB, MODIFIED > 7 days
Scanned: 2025-10-18 14:45:00

File                        Size      Modified
src/parser/mod.rs          45.2KB    2025-10-15
src/compiler/codegen.rs    128.7KB   2025-10-16
src/utils/helpers.rs       23.4KB    2025-10-17
------------------------------------------------------------

Total: 3 files
Combined Size: 197.3KB
Average Size: 65.8KB
```

### System Monitoring

```bash
# Capture system metrics for monitoring
rpt sys COLUMNS:cpu,memory,disk FORMAT:report INTO:metrics.txt
```

**Output**:
```text
## SYSTEM INFORMATION
Captured: 2025-10-18 14:50:00

### CPU
Model: AMD Ryzen 9 5950X
Cores: 32 (16 physical)
Usage: 24.5%
Load: 2.4, 2.1, 1.8

### Memory
Total: 64GB
Used: 48GB (75%)
Available: 16GB
Swap: 8GB (unused)

### Disk
Total: 916GB
Used: 143GB (15.6%)
Available: 726GB
------------------------------------------------------------

Status: Normal
Next Check: 2025-10-18 15:00:00
```

### Cleanup Preparation

```bash
# Find large old files for cleanup
rpt files ./temp SIZE:>100MB MODIFIED:>30d FORMAT:report -p
```

**Output**:
```text
## CLEANUP PREVIEW
Path: ./temp
Filters: SIZE > 100MB, MODIFIED > 30 days
Mode: Preview (no changes will be made)

File                        Size      Modified      Age
temp/archive.zip           450MB     2025-08-15    64 days
temp/old_backup.tar.gz     890MB     2025-07-20    90 days
temp/cache_dump.db         234MB     2025-09-01    47 days
------------------------------------------------------------

Total: 3 files
Space to Reclaim: 1.54GB
Preview only - use without -p to execute
```

### Security Audit

```bash
# Generate hash inventory for files
rpt files . -r HASH:sha256 FORMAT:report
```

**Output**:
```text
## FILE HASH REPORT
Filters: HASH: sha256 (delegated to CMP)
Path: .
Recursive: Yes
Generated: 2025-10-18 15:00:00

File                        Hash (first 12)       Size
config.json                a1b2c3d4e5f6         2.3KB
README.md                  7g8h9i0j1k2l         5.6KB
src/main.rs                3m4n5o6p7q8r         12.4KB
------------------------------------------------------------

Total Files Hashed: 3
Algorithm: SHA-256
```

---

## 🎓 Summary

The CNP Report Format specification defines a human-optimized output format for terminal-based data analysis and system reporting. Key characteristics:

**Core Principles**:
- Human readability first
- Context preservation through metadata and summaries
- Terminal-aware formatting with dynamic width handling
- Clear visual hierarchy with sections and separators

**Integration**:
- Extends CNP_Reporting_Grammar.md canonical formats
- Supports Universal Routes (TO:, INTO:, FORMAT:)
- Displays delegated filter results transparently
- Scope-specific defaults for optimal user experience

**Best Practices**:
- Use `report` for interactive terminal analysis
- Use `table` for documentation and Markdown export
- Use `json/csv` for machine processing and automation
- Use `cnp` for tool interchange and session replay

**Implementation**:
- Terminal width detection with graceful degradation
- Optional color support with automatic detection
- Comprehensive error and edge case handling
- Unit conversion and formatting standards

This specification provides implementers with clear guidelines for creating consistent, high-quality report output across all CNP reporting tools.

---

**Version**: 1.0
**Status**: Production Ready
**Author**: CNP Ecosystem
**Last Updated**: 2025-10-18
**Next Review**: Before rpt v1.0.0 release

**Related Specifications**:
- CNP_Base_Grammar.md - Universal grammar foundation
- CNP_Reporting_Grammar.md - Reporting tool grammar
- CNP_Keyword_Routing.md - Filter delegation protocol
- CNP_Tree_Syntax.md - Hierarchical tree format
- CNP_File_Format.md - CNP interchange format
