# Quick Start

Get up and running with SMV in 5 minutes.

## Basic File Transformations

### Transform to snake_case
```bash
# Preview transformation (safe)
smv snake . -p

# Apply transformation
smv snake . -F
```

### Transform to kebab-case
```bash
smv kebab . -p
```

### Transform specific file types
```bash
# Only transform .txt files
smv snake . EXT:txt -p
```

## File Operations with Filters

### Remove files by extension
```bash
# Preview deletion of all .log files
smv rm . EXT:log -p

# Actually delete them
smv rm . EXT:log -F
```

### Find and replace in filenames
```bash
# Replace "old" with "new" in all filenames
smv CHANGE "old" INTO "new" . -p
```

## Key Concepts

### Default Behavior
- **Files only**: SMV operates on files by default, excluding directories
- **Use `-e`** to include directories: `smv snake . -e`

### Safety First
- **Always use `-p`** (preview) first to see what will happen
- **Use `-F`** (force) to actually apply changes
- **Recursive**: Add `-r` to process subdirectories

### CNP Filters
- `EXT:txt` - files with .txt extension
- `TYPE:file` - files only
- `SIZE>1MB` - files larger than 1MB
- `NAME:test*` - files starting with "test"

## Next Steps

- Read [Basic Usage](basic-usage.md) for detailed examples
- Explore [Commands Overview](../commands/overview.md) for all available operations
- Learn about [CNP Filters](../features/cnp-filters.md) for powerful file selection