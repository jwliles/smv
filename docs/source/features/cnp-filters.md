# CNP Filters

CNP filters provide powerful, declarative file selection using a simple UPPERCASE syntax.

## Filter Syntax

Filters use a consistent `KEYWORD:value` or `KEYWORD>value` format:

```bash
smv snake . EXT:txt TYPE:file SIZE>1MB
```

## Available Filters

### Extension Filters

Target files by extension:

```bash
EXT:txt          # Files with .txt extension
EXT:md           # Files with .md extension
EXT:rs           # Files with .rs extension
```

**Examples:**
```bash
# Transform only markdown files
smv kebab . EXT:md -p

# Remove log and temp files
smv rm . EXT:log EXT:tmp -p

# Multiple extensions
smv snake . EXT:js EXT:ts EXT:jsx -p
```

### Type Filters

Filter by file type:

```bash
TYPE:file        # Files only (default behavior anyway)
TYPE:dir         # Directories only (requires -e flag)
TYPE:symlink     # Symbolic links only
```

**Examples:**
```bash
# Transform directory names (note: requires -e)
smv kebab . TYPE:dir -e -p

# Remove broken symlinks
smv rm . TYPE:symlink -p
```

### Size Filters

Filter by file size:

```bash
SIZE>1MB         # Files larger than 1MB
SIZE<100KB       # Files smaller than 100KB
SIZE>0           # Non-empty files
```

**Supported units:** B, KB, MB, GB, TB

**Examples:**
```bash
# Remove large temporary files
smv rm . EXT:tmp SIZE>10MB -p

# Transform small config files
smv snake . EXT:conf SIZE<1KB -p

# Find empty files
smv rm . SIZE>0 -p  # This would show non-empty files
```

### Name Pattern Filters

Match files by name patterns:

```bash
NAME:test*       # Files starting with "test"
NAME:*backup     # Files ending with "backup"
NAME:*temp*      # Files containing "temp"
NAME:config      # Exact match for "config"
```

**Examples:**
```bash
# Transform test files
smv camel . NAME:*test* EXT:js -p

# Remove backup files
smv rm . NAME:*backup* -p

# Transform files starting with "old_"
smv CHANGE "old_" INTO "new_" . NAME:old_* -p
```

### Date Filters (Planned)

Filter by modification or access time:

```bash
MODIFIED>2024-01-01    # Modified after date
MODIFIED<2024-01-01    # Modified before date
ACCESSED>2024-01-01    # Accessed after date
```

## Combining Filters

Multiple filters work together with **AND logic**:

```bash
# Large JavaScript test files
smv camel . EXT:js SIZE>1KB NAME:*test* -p

# Remove old large log files
smv rm . EXT:log SIZE>10MB NAME:*old* -p

# Transform specific config files
smv kebab . EXT:conf TYPE:file NAME:*local* -p
```

## Advanced Filter Patterns

### Semantic Groups (Planned)

Predefined filter combinations for common use cases:

```bash
FOR:scripts      # Common script extensions (.sh, .py, .rb, .js, etc.)
FOR:media        # Common media files (.jpg, .png, .mp4, etc.)
FOR:configs      # Common config files (.conf, .ini, .yaml, etc.)
FOR:docs         # Common document files (.md, .txt, .pdf, etc.)
```

### Complex Patterns

```bash
# Files that match multiple criteria
smv snake . EXT:js EXT:ts NAME:*component* SIZE>1KB -p

# Clean up build artifacts
smv rm . NAME:*build* NAME:*dist* TYPE:dir -e -p

# Transform source files but not tests
smv camel . EXT:rs NAME:*src* -p
```

## Filter Examples by Use Case

### Project Cleanup
```bash
# Remove temporary and cache files
smv rm . EXT:tmp EXT:cache NAME:*temp* -rp

# Remove large build artifacts
smv rm . NAME:*build* NAME:*dist* NAME:*target* TYPE:dir SIZE>100MB -ep
```

### Code Organization
```bash
# Standardize test file naming
smv snake . EXT:js EXT:ts NAME:*test* NAME:*spec* -rp

# Transform component files
smv pascal . EXT:jsx EXT:vue NAME:*component* -p
```

### Media Management
```bash
# Remove large old photos
smv rm . EXT:jpg EXT:png SIZE>10MB MODIFIED<2023-01-01 -p

# Standardize image naming
smv kebab . EXT:jpg EXT:png EXT:gif -p
```

### Document Processing
```bash
# Transform markdown documentation
smv kebab . EXT:md NAME:*doc* NAME:*readme* -p

# Remove empty text files
smv rm . EXT:txt SIZE>0 -p  # Shows non-empty files
```

## Best Practices

### Start Specific
```bash
# GOOD: Start with specific filters
smv snake ./src EXT:rs -p

# Then expand scope
smv snake . EXT:rs -rp
```

### Use Preview Mode
```bash
# ALWAYS preview first
smv rm . SIZE>100MB -p

# Then apply if correct
smv rm . SIZE>100MB -F
```

### Combine Filters Thoughtfully
```bash
# Logical combinations
smv snake . EXT:js NAME:*test* -p   # JavaScript test files

# Avoid conflicting filters
smv snake . TYPE:file TYPE:dir -p   # This doesn't make sense
```