# Basic Usage

Learn the fundamentals of SMV through practical examples.

## Command Structure

```bash
smv [COMMAND] [TARGET] [FILTERS] [FLAGS]
```

- **COMMAND**: Transformation or operation (snake, kebab, mv, rm, etc.)
- **TARGET**: Directory or file pattern (default: current directory)
- **FILTERS**: CNP filters like EXT:txt, TYPE:file, SIZE>1MB
- **FLAGS**: Modifiers like -p (preview), -r (recursive), -F (force)

## File Transformations

### Case Transformations

```bash
# Transform files to different cases
smv snake .          # snake_case
smv kebab .          # kebab-case  
smv pascal .         # PascalCase
smv camel .          # camelCase
smv title .          # Title Case
smv lower .          # lowercase
smv upper .          # UPPERCASE
```

### String Replacements

```bash
# Replace substring in filenames
smv CHANGE "old_prefix" INTO "new_prefix" .

# Remove prefix (replace with empty string)
smv CHANGE "IMG_" INTO "" .

# Use with filters
smv CHANGE "test" INTO "spec" . EXT:js
```

## File Operations

### Moving and Copying

```bash
# Basic file operations
smv mv source.txt destination.txt
smv cp file.txt backup.txt

# With preview
smv mv *.txt backup/ -p
```

### Deletion with Filters

```bash
# Delete by extension
smv rm . EXT:log -p
smv rm . EXT:tmp -F

# Delete large files
smv rm . SIZE>100MB -p

# Delete by name pattern
smv rm . NAME:*backup* -p
```

## Working with Filters

### Extension Filters
```bash
smv snake . EXT:txt          # Only .txt files
smv snake . EXT:md EXT:txt   # Multiple extensions
```

### Type Filters
```bash
smv snake . TYPE:file        # Files only (default anyway)
smv snake . TYPE:dir -e      # Directories only
```

### Size Filters
```bash
smv rm . SIZE>1MB            # Files larger than 1MB
smv rm . SIZE<1KB            # Files smaller than 1KB
```

### Name Patterns
```bash
smv snake . NAME:test*       # Files starting with "test"
smv snake . NAME:*backup     # Files ending with "backup"
smv snake . NAME:*temp*      # Files containing "temp"
```

## Combining Filters

```bash
# Multiple filters work together (AND logic)
smv rm . EXT:log SIZE>10MB NAME:*old* -p

# Transform large JavaScript test files
smv camel . EXT:js SIZE>1KB NAME:*test* -p
```

## Flags and Modifiers

### Essential Flags
- `-p` - Preview mode (show what would happen)
- `-F` - Force mode (actually apply changes)
- `-r` - Recursive (include subdirectories)
- `-e` - Everything (include directories)

### Combining Flags
```bash
smv snake . -rpe             # Recursive, preview, everything
smv rm . EXT:tmp -rF         # Recursive, force
```

## Safety Practices

### Always Preview First
```bash
# GOOD: Preview first
smv rm . EXT:log -p
# Then apply if looks correct
smv rm . EXT:log -F

# BAD: Applying directly without preview
smv rm . EXT:log -F  # Risky!
```

### Use Specific Filters
```bash
# GOOD: Specific targeting
smv rm ./temp EXT:tmp -p

# BAD: Too broad
smv rm . -F  # Dangerous!
```

### Test on Small Sets
```bash
# Start with a specific directory
smv snake ./test_dir -p

# Then expand to larger scope
smv snake . -rp
```

## Common Workflows

### Organizing Downloads
```bash
# Preview organization
smv snake ~/Downloads -p

# Apply to specific file types
smv snake ~/Downloads EXT:pdf EXT:txt -F
```

### Cleaning Project Directories
```bash
# Remove build artifacts
smv rm . NAME:*build* TYPE:dir -rp
smv rm . EXT:o EXT:tmp -rp

# Standardize source file naming
smv snake ./src EXT:rs -rp
```

### Batch File Renaming
```bash
# Remove camera prefixes
smv CHANGE "IMG_" INTO "" ~/Photos -p

# Standardize naming convention
smv snake . EXT:jpg EXT:png -p
```