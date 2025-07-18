# SMV Manual

## NAME
smv - smart move and rename tool with CNP grammar support

## SYNOPSIS
**smv** \[*COMMAND*\] \[*PATH*\] \[*FILTERS*\] \[*ROUTES*\] \[*FLAGS*\]

**smv** \[*options*\] \[*files*\]

## DESCRIPTION
SMV (Smart Move) is a powerful file operation tool that extends the functionality of the standard Unix `mv` command with intelligent filename transformations and full CNP (Canopy) ecosystem integration.

SMV operates in two modes:
1. **CNP Grammar Mode** - Advanced operations using CNP filters, routes, and tool delegation
2. **Legacy Mode** - Drop-in replacement for standard `mv` command

## CNP GRAMMAR MODE

### Basic Syntax
```
smv <COMMAND> <PATH> [FILTERS] [ROUTES] [FLAGS]
```

### Commands
- **snake** - Convert filenames to snake_case
- **kebab** - Convert filenames to kebab-case  
- **pascal** - Convert filenames to PascalCase
- **camel** - Convert filenames to camelCase
- **title** - Convert filenames to Title Case
- **lower** - Convert filenames to lowercase
- **upper** - Convert filenames to UPPERCASE
- **clean** - Clean up spaces and special characters
- **CHANGE** "old" **INTO** "new" - Replace substring in filenames
- **REGEX** "pattern" **INTO** "replacement" - Replace using regex

### Filters
- **NAME:** value - Match filenames containing value
- **TYPE:** file|folder|symlink|other - Filter by file type
- **EXT:** extension - Filter by file extension (e.g., EXT:md)
- **SIZE>** value | **SIZE<**value - Filter by file size (e.g., SIZE>1MB, SIZE<500KB)
- **DEPTH>** value | **DEPTH<**value - Filter by directory depth
- **MODIFIED>** date | **MODIFIED<**date - Filter by modification date (YYYY-MM-DD)
- **ACCESSED>** date | **ACCESSED<**date - Filter by access date (YYYY-MM-DD)

### Semantic Groups
- **FOR: notes** - Markdown, text, and documentation files
- **FOR: media** - Images, videos, and audio files  
- **FOR: scripts** - Shell, Python, Rust, and other script files
- **FOR: projects** - Source directories and project folders
- **FOR: configs** - Configuration files (yaml, json, toml, etc.)

### Routes
- **TO:** tool - Delegate operation to another CNP tool
  - **TO:** say - Use SAY for natural language processing
  - **TO:** dff - Use DFF for duplicate file finding
  - **TO:** xfd - Use XFD for interactive selection
  - **TO:** dsc - Use DSC for ultra-fast discovery
- **INTO:** filename - Save output to file
- **FORMAT:** type - Format output (json, csv, yaml, text)

### Flags
- **-r** - Recursive (process subdirectories)
- **-p** - Preview (show changes without applying)
- **-f** - Force (skip confirmations)
- **-i** - Interactive mode
- **-T** - Terminal UI mode
- **-u** - Undo last operation

## EXAMPLES

### Basic Transformations
```bash
# Convert markdown files to snake_case with preview
smv snake . EXT:md -p

# Clean up media filenames recursively
smv clean . FOR:media -r

# Convert config files to title case
smv title . FOR:configs -p
```

### Advanced Filtering
```bash
# Transform large text files only
smv kebab . TYPE:file EXT:txt SIZE>1MB -p

# Process recently modified scripts
smv clean . FOR:scripts MODIFIED>2024-01-01 -r

# Work with specific filename patterns
smv pascal . NAME:draft TYPE:file -p
```

### Tool Delegation
```bash
# Use SAY for complex word segmentation
smv snake . EXT:epub TO:say split_and_titlecase

# Find duplicates with DFF
smv organize . FOR:media TO:dff find_duplicates

# Interactive selection with XFD
smv clean . TYPE:file TO:xfd interactive_select
```

### Output Routing
```bash
# Save file list to text file
smv clean . FOR:scripts INTO:cleaned_files.txt

# Output results as JSON
smv title . TYPE:file FORMAT:json -p

# Generate CSV report with file metadata
smv snake . EXT:md FORMAT:csv
```

### String Replacement
```bash
# Replace substring in filenames
smv CHANGE "old" INTO "new" . -p

# Use regex for complex patterns
smv REGEX "\\d+" INTO "XXX" . -r
```

## LEGACY MODE

SMV works as a drop-in replacement for the standard `mv` command:

```bash
# Move files
smv file.txt /path/to/destination/

# Rename files
smv old_name.txt new_name.txt

# Move multiple files
smv file1.txt file2.txt destination_directory/
```

### Legacy Options
- **-i**, **--interactive** - Launch interactive REPL interface
- **-p**, **--preview** - Preview changes without applying them
- **-r**, **--recursive** - Process subdirectories recursively
- **-f**, **--force** - Skip confirmations
- **-T**, **--tui** - Launch terminal UI mode
- **-u**, **--undo** - Undo the last operation
- **--exclude** PATTERNS - Comma-separated patterns to exclude
- **--max-history-size** SIZE - Maximum number of operations in history
- **-h**, **--help** - Print help
- **-V**, **--version** - Print version

## INTERACTIVE MODES

### REPL Interface
Launch the interactive Read-Eval-Print Loop:
```bash
smv -i
```

Commands available in REPL:
- **ls** [pattern] - List files
- **cd** directory - Change directory
- **preview** transform files - Show transformation preview
- **apply** transform files - Apply transformation
- **undo** - Revert last operation
- **help** - Show help
- **quit** - Exit program

### Terminal UI Mode
Launch the full-screen terminal interface:
```bash
smv -T
```

Features:
- File explorer with Vim-style navigation (hjkl, gg, G)
- Visual selection mode for multiple files
- Fuzzy search integration
- Operation queue with preview
- Real-time transformation preview

## FILE TRANSFORMATIONS

| Transform | Description | Example |
|-----------|-------------|---------|
| clean | Clean up spaces and special characters | `My File (1).txt` → `My File 1.txt` |
| snake | Convert to snake_case | `My-File.txt` → `my_file.txt` |
| kebab | Convert to kebab-case | `My_File.txt` → `my-file.txt` |
| title | Convert to Title Case | `my_file.txt` → `My File.txt` |
| camel | Convert to camelCase | `my_file.txt` → `myFile.txt` |
| pascal | Convert to PascalCase | `my_file.txt` → `MyFile.txt` |
| lower | Convert to lowercase | `MyFile.txt` → `myfile.txt` |
| upper | Convert to UPPERCASE | `myFile.txt` → `MYFILE.TXT` |

## SIZE UNITS

Size filters support the following units:
- **B** - Bytes
- **KB** - Kilobytes (1024 bytes)
- **MB** - Megabytes (1024 KB)
- **GB** - Gigabytes (1024 MB) 
- **TB** - Terabytes (1024 GB)

Examples: `SIZE>1MB`, `SIZE<500KB`, `SIZE>2GB`

## DATE FORMATS

Date filters use YYYY-MM-DD format:
- **MODIFIED>2024-01-01** - Files modified after January 1, 2024
- **ACCESSED<2023-12-31** - Files accessed before December 31, 2023

## SAFETY FEATURES

### Backups
SMV automatically creates backups of modified files in `~/.config/smv/backups/`. This enables the undo functionality to work across program sessions.

### Conflict Detection  
SMV will not overwrite existing files unless explicitly instructed, preventing accidental data loss.

### Undo Functionality
```bash
# Command-line undo
smv -u

# REPL undo
smv> undo

# Interactive mode undo
smv -i
smv> undo
```

## CNP ECOSYSTEM INTEGRATION

SMV is part of the CNP (Canopy) ecosystem and integrates with:

- **DSC** - Ultra-fast file discovery (replaces find/fd)
- **XFD** - Interactive fuzzy search (replaces fzf/skim)
- **SAY** - Natural language processing and grammar translation
- **DFF** - Duplicate file finder
- **SKL** - Advanced search and pattern matching (replaces grep/ripgrep)

Tool delegation allows SMV to leverage specialized capabilities:
```bash
# Use SAY for intelligent word boundary detection
smv snake . EXT:epub TO:say split_and_titlecase

# Use DFF for duplicate detection before organizing
smv organize . FOR:media TO:dff find_duplicates
```

## FILES

- **~/.config/smv/backups/** - Automatic file backups for undo functionality
- **~/.config/smv/history** - Command history for interactive mode

## EXIT STATUS

SMV exits with status:
- **0** - Success
- **1** - General error
- **2** - Parse error
- **3** - File operation error

## EXAMPLES WITH COMPLEX WORKFLOWS

### Organizing Media Collection
```bash
# Find and clean up all media files
smv clean . FOR:media -r -p

# Organize by type and remove duplicates
smv organize . FOR:media TO:dff find_duplicates

# Generate report of processed files
smv title . FOR:media FORMAT:csv INTO:media_report.csv
```

### Code Project Cleanup
```bash
# Clean up script filenames
smv snake . FOR:scripts -r -p

# Process config files separately
smv kebab . FOR:configs -p

# Find large files that might need attention
smv clean . SIZE>10MB TYPE:file FORMAT:json
```

### Document Management
```bash
# Standardize document names
smv title . FOR:notes NAME:draft -p

# Clean up old temporary files
smv clean . MODIFIED<2023-01-01 NAME:temp -p

# Export processed file list
smv snake . EXT:md INTO:processed_docs.txt
```

## SEE ALSO

**mv**(1), **find**(1), **rename**(1), **dsc**(1), **xfd**(1)

## AUTHOR

SMV is part of the CNP (Canopy) ecosystem developed as a modern file operation suite.

## REPORTING BUGS

Report bugs to the SMV issue tracker.

## COPYRIGHT

This is free software; see the source for copying conditions.