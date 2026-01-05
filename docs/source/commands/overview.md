---
date_created: 2025-10-02T11-25-26
date_updated: 2025-08-30T02-49-20
timestamp: 1759404326268
title: overview
id: 846e5e2c-9157-463e-8a9c-f8e990b1aedf
hash: 8fdbf34e05cd7c0c62fa17540d0a5ea390fd07b8ecedf05642e8dd98aab2fa01
---
# Commands Overview

SMV provides powerful file operations through a consistent command interface.

## Transformation Commands

### Case Transformations
Transform filename casing while preserving file extensions.

| Command | Output Style | Example |
|---------|-------------|---------|
| `snake` | snake_case | `My File.txt` → `my_file.txt` |
| `kebab` | kebab-case | `My File.txt` → `my-file.txt` |
| `pascal` | PascalCase | `my file.txt` → `MyFile.txt` |
| `camel` | camelCase | `my file.txt` → `myFile.txt` |
| `title` | Title Case | `my file.txt` → `My File.txt` |
| `lower` | lowercase | `My File.txt` → `my file.txt` |
| `upper` | UPPERCASE | `My File.txt` → `MY FILE.TXT` |

### String Operations

| Command | Description | Example |
|---------|-------------|---------|
| `CHANGE: "old" INTO "new"` | Replace substring | `CHANGE: "IMG_" INTO ""` |

## File Operations

### Standard Operations
| Command | Description | Example |
|---------|-------------|---------|
| `mv` | Move/rename files | `smv mv file.txt newname.txt` |
| `cp` | Copy files | `smv cp file.txt backup.txt` |
| `rm` | Remove files | `smv rm . EXT:log` |

### Creation Operations
| Command | Description | Example |
|---------|-------------|---------|
| `-cd` | Create directories | `smv -cd newdir` |
| `-cf` | Create/touch files | `smv -cf newfile.txt` |

## Interactive Modes

| Command | Description |
|---------|-------------|
| `interactive` | Launch REPL mode |
| `tui` | Launch terminal UI |

## Universal Flags

### Core Flags
| Flag | Description |
|------|-------------|
| `-p` | Preview mode (show changes without applying) |
| `-F` | Force mode (skip confirmations) |
| `-r` | Recursive (process subdirectories) |
| `-e` | Everything (include directories) |

### Additional Flags
| Flag | Description |
|------|-------------|
| `-i` | Case-insensitive pattern matching |
| `-v` | Verbose output |
| `-a` | Include hidden files |
| `-n` | No-clobber (don't overwrite existing files) |

## Examples by Use Case

### Project Organization
```bash
# Standardize source files
smv snake ./src EXT:rs -rp

# Organize by file type
smv mv . EXT:md docs/ -p
```

### Cleanup Operations
```bash
# Remove temporary files
smv rm . EXT:tmp EXT:log -rp

# Clean build artifacts
smv rm . NAME:*build* TYPE:dir -rp
```

### Batch Renaming
```bash
# Remove camera prefixes
smv CHANGE: "DSC_" INTO "" ~/Photos -p

# Standardize test files
smv snake . NAME:*test* EXT:js -p
```

### File Type Conversion Prep
```bash
# Prepare files for processing
smv kebab . EXT:md -p
smv lower . EXT:jpg EXT:png -p
```
