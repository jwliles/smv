# SMV - Smart Move

A powerful, Rust-based CNP (Canopy) ecosystem tool that replaces the standard Unix `mv` command with intelligent filename transformation capabilities and full CNP grammar support.

## Features

### Core Functionality
- **POSIX-compatible mv/cp commands** with all standard flags (-r, -f, -n, -L, -P, --preserve)
- **Drop-in replacement** for the standard Unix `mv` command
- **CNP Grammar Support** - Full Canopy ecosystem integration with filters, routes, and tool delegation
- **Interactive REPL Interface** with command history and tab completion
- **Smart File Renaming** with multiple transformation strategies:
  - Convert to snake_case (`document-name.pdf` → `document_name.pdf`)
  - Convert to kebab-case (`document_name.pdf` → `document-name.pdf`)
  - Convert to Title Case (`document_name.pdf` → `Document Name.pdf`)
  - Convert to camelCase (`document_name.pdf` → `documentName.pdf`)
  - Convert to PascalCase (`document_name.pdf` → `DocumentName.pdf`)
  - Convert to lowercase or UPPERCASE
  - Clean up spaces and special characters
  - **Split camelCase/PascalCase** then apply any transformation (`featureWishList.md` → `feature_wish_list.md`)

### CNP Ecosystem Integration
- **Filter Keywords**: `NAME:`, `TYPE:`, `EXT:`, `SIZE>`, `DEPTH<`, `MODIFIED>`, `ACCESSED<`
- **Semantic Groups**: `FOR:notes`, `FOR:media`, `FOR:scripts`, `FOR:projects`, `FOR:configs`
- **Tool Delegation**: `TO:say`, `TO:dff`, `TO:xfd`, `TO:dsc` for specialized operations
- **Output Routing**: `INTO:file.txt`, `FORMAT:json/csv/yaml` for structured output
- **Advanced Filtering**: Complex file discovery with logical grouping

### Advanced Features
- **Directory Organization**:
  - Group files by basename into directories
  - Flatten directory structures by moving all files to the root
  - Clean up empty directories after flattening
- **Preview Mode** - See changes before they're applied
- **Batch Processing** - Apply transformations to multiple files at once
- **Undo Functionality** - Safely revert changes
- **Safety Features** - File backups and conflict detection

## Installation

### Using Cargo

```bash
cargo install smv
```

### From Source

```bash
# Clone the repository
git clone https://github.com/jwliles/smv.git
cd smv

# Build and install
cargo install --path .
```

## Usage

### CNP Grammar (Recommended)

SMV supports the full CNP (Canopy) grammar for advanced file operations with filters, semantic groups, and tool delegation:

#### Basic CNP Syntax
```bash
# Transform with extension filter
smv snake . EXT:md -p                    # Convert markdown files to snake_case

# Filter by file type  
smv kebab . TYPE:file EXT:txt -r         # Only process .txt files, recursively

# Use semantic groups
smv title . FOR:notes -p                 # Transform all note files (md, txt, etc.)
smv clean . FOR:media                    # Clean up media filenames

# Size-based filtering
smv pascal . SIZE>1MB TYPE:file -p       # Large files only

# Date-based filtering  
smv lower . MODIFIED>2024-01-01 -r       # Files modified after Jan 1, 2024

# Prefix removal
smv CHANGE "IMG_" INTO "" . EXT:jpg -p    # Remove "IMG_" prefix from all JPG files
smv CHANGE "DSC" INTO "" . EXT:png        # Remove "DSC" prefix from PNG files

# Split camelCase/PascalCase then transform
smv split snake . EXT:md -p              # Split camelCase files then convert to snake_case
smv split kebab . EXT:js TYPE:file       # Split PascalCase files then convert to kebab-case
```

#### Advanced CNP Features
```bash
# Tool delegation - delegate complex operations to specialized tools
smv snake . EXT:epub TO:say split_and_titlecase  # Use SAY for word segmentation
smv organize . FOR:media TO:dff find_duplicates  # Use DFF to find duplicate media

# Output routing - save results to files
smv clean . FOR:scripts INTO:cleaned_files.txt   # Save file list to text file
smv title . TYPE:file FORMAT:json -p             # Output as JSON format

# Complex filtering with multiple criteria
smv kebab . TYPE:file EXT:md SIZE<1MB NAME:draft -p
```

#### CNP Semantic Groups
- `FOR:notes` - Markdown, text, and documentation files  
- `FOR:media` - Images, videos, and audio files
- `FOR:scripts` - Shell, Python, Rust, and other script files
- `FOR:projects` - Source directories and project folders
- `FOR:configs` - Configuration files (yaml, json, toml, etc.)

### Basic Usage (POSIX-compatible mv/cp)

SMV provides full POSIX-compatible file operations with all standard flags:

#### Move Operations
```bash
# Move a file to another location
smv mv file.txt /path/to/destination/

# Move multiple files to a directory
smv mv file1.txt file2.txt destination_directory/

# Rename a file
smv mv old_name.txt new_name.txt

# Move with flags
smv mv source dest -f              # Force overwrite
smv mv source dest -n              # No-clobber (don't overwrite)
smv mv *.txt backup/ -r            # Recursive move
smv mv file dest --interactive-confirm  # Prompt before overwrite
```

#### Copy Operations
```bash
# Copy a file
smv cp file.txt backup/

# Copy multiple files
smv cp file1.txt file2.txt *.md backup/

# Copy directories recursively
smv cp source_dir/ backup_dir/ -r

# Copy with metadata preservation
smv cp important.txt backup/ --preserve

# Copy with symbolic link handling
smv cp -L symlink dest             # Dereference symlinks
smv cp -P symlink dest             # Preserve symlinks
```

#### Standard Flags
- `-r` - Recursive (for directories)
- `-f` - Force (overwrite without confirmation)
- `-n` - No-clobber (never overwrite existing files)
- `-L` - Dereference symbolic links
- `-P` - Do not follow symbolic links
- `--preserve` - Preserve file attributes, ownership, and timestamps
- `--interactive-confirm` - Prompt before overwriting files

```

### New Command Structure

SMV follows the LAR project command philosophy: `<tool> [scope] [targets] [modifiers]`

#### Interactive Guidance System

SMV features an Excel-like command guidance system that shows available options as you type:

```bash
$ smv -snake . pdf
[smv] [-snake] [. pdf] [preview|recursive|force]
 cmd   scope    targets    modifiers (optional)
```

- **F1**: Get context-sensitive help for current position
- **Tab**: Cycle through valid options
- **Backspace**: Navigate to previous position

#### Transformation Mode

Apply transformations using the new sequential structure:

```bash
# Convert filenames to snake_case
smv -snake . txt

# Convert filenames to kebab-case with preview
smv -kebab . pdf preview

# Clean up spaces and special characters recursively
smv -clean /path/to/messy/files/ recursive

# Convert to Title Case for specific extensions
smv -title documents/ md txt

# Multiple file types with preview
smv -pascal . pdf txt docx preview
```

### Directory Organization

Organize or flatten directory structures:

```bash
# Group files by basename into directories
smv -group /path/to/files/

# Preview grouping without making changes
smv -group /path/to/files/ preview

# Flatten all files from subdirectories into the root directory
smv -flatten /path/to/nested/folders/

# Sort files by type with grouping
smv -sort downloads/ type group
```

### Interactive Modes

#### AFN REPL Integration

SMV is designed to work within the AFN REPL environment:

```bash
# Start AFN REPL
$ afn
AFN> smv -snake . pdf preview
AFN> smv -pascal documents/ txt
AFN> exit
$
```

#### Standalone REPL Interface

Launch the interactive REPL interface:

```bash
# Start interactive session
smv -interactive
```

Type `exit` to quit the REPL. All operations are saved to history automatically.

#### Terminal UI Mode

Launch the TUI file explorer with Vim-style navigation:

```bash
# Start TUI mode
smv -tui
```

The TUI mode features:
- File explorer with Vim motions (hjkl, gg, G)
- Visual selection mode for multiple files
- Fuzzy search using Skim integration
- GParted-style operation queue
- Preview of file transformations

<!--Dev Note: We need to explain how the backup mode works in more detail. We can have a separate file that goes all the way through it, or put enough detail in the README most folks will understand. We do need a documentation site somewhere. Maybe a wiki on GitHub or using Sphinx and REst. -->

In the interactive shell:

```
smv> ls
document.pdf document.docx image.jpg notes.txt

smv> preview snake *.pdf
document.pdf → document.pdf (no change needed)

smv> rename notes.txt
...
Select transformation:
  1. Clean up spaces and special characters
  ...
  4. Convert to Title Case
...
Preview of changes:
  "notes.txt" → "Notes.txt"
Apply these changes? [y/N] y
Renamed: "notes.txt" → "Notes.txt"

smv> undo
Operation undone successfully.

smv> help
Commands:
  preview <transform> <files>  - Show transformation without applying
  apply <transform> <files>    - Apply transformation
  rename <files> --options     - Interactive renaming wizard
  undo                        - Revert last operation
  cd <directory>              - Change directory
  ls [pattern]                - List files
  help                        - Show this help
  quit                        - Exit program
```

## Command Reference

### CNP Grammar Syntax

```
smv <COMMAND> <PATH> [FILTERS] [ROUTES] [FLAGS]
```

**Commands**: `snake`, `kebab`, `pascal`, `camel`, `title`, `lower`, `upper`, `clean`, `split TRANSFORMATION`, `CHANGE "old" INTO "new"`, `CHANGE "prefix" INTO ""` (prefix removal), `REGEX "pattern" INTO "replacement"`

**Filters**:
- `NAME:value` - Match filenames containing value  
- `TYPE:file|folder|symlink` - Filter by file type
- `EXT:extension` - Filter by file extension
- `SIZE>1MB` / `SIZE<500KB` - Filter by file size
- `DEPTH>2` / `DEPTH<1` - Filter by directory depth  
- `MODIFIED>2024-01-01` / `MODIFIED<2023-12-31` - Filter by modification date
- `ACCESSED>2024-01-01` / `ACCESSED<2023-12-31` - Filter by access date
- `FOR:notes|media|scripts|projects|configs` - Semantic file groups

**Routes**:
- `TO:tool` - Delegate operation to another CNP tool (say, dff, xfd, dsc)
- `INTO:filename` - Save output to file
- `FORMAT:json|csv|yaml|text` - Format output

**Flags**: `-r` (recursive), `-p` (preview), `-f` (force), `-i` (interactive), `-T` (TUI), `-u` (undo)

### Legacy Command Options

```
Options:
  -i, --interactive              Launch interactive REPL interface
  -p, --preview                  Preview changes without applying them
  -r, --recursive                Process subdirectories recursively
  -f, --force                    Skip confirmations
  -T, --tui                      Launch terminal UI mode
  -u, --undo                     Undo the last operation
      --exclude <PATTERNS>       Comma-separated patterns to exclude
      --max-history-size <SIZE>  Maximum number of operations in history [default: 50]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Transform Commands
- `snake` - Convert to snake_case
- `kebab` - Convert to kebab-case  
- `pascal` - Convert to PascalCase
- `camel` - Convert to camelCase
- `title` - Convert to Title Case
- `lower` - Convert to lowercase
- `upper` - Convert to UPPERCASE
- `clean` - Clean up spaces and special characters
- `split TRANSFORMATION` - Split camelCase/PascalCase then apply transformation

## Transformations

| Transform | Description | Example |
|-----------|-------------|---------|
| `clean` | Clean up spaces and special characters | `My File (1).txt` → `My File 1.txt` |
| `snake` | Convert to snake_case | `My-File.txt` → `my_file.txt` |
| `kebab` | Convert to kebab-case | `My_File.txt` → `my-file.txt` |
| `title` | Convert to Title Case | `my_file.txt` → `My File.txt` |
| `camel` | Convert to camelCase | `my_file.txt` → `myFile.txt` |
| `pascal` | Convert to PascalCase | `my_file.txt` → `MyFile.txt` |
| `lower` | Convert to lowercase | `MyFile.txt` → `myfile.txt` |
| `upper` | Convert to UPPERCASE | `myFile.txt` → `MYFILE.TXT` |
| `split snake` | Split camelCase/PascalCase then convert to snake_case | `featureWishList.md` → `feature_wish_list.md` |
| `split kebab` | Split camelCase/PascalCase then convert to kebab-case | `UserSettings.json` → `user-settings.json` |
| `split title` | Split camelCase/PascalCase then convert to Title Case | `apiEndpoint.ts` → `ApiEndpoint.ts` |
| `CHANGE "prefix" INTO ""` | Remove prefix from filename | `IMG_1234.jpg` → `1234.jpg` |

## Safety Features

### Backups

SMV automatically creates backups of modified files in `~/.config/smv/backups/`. This allows the undo functionality to work even after the program has been closed and reopened.

### Undo Functionality

The undo command reverts the most recent operation. 

In interactive mode, you can use:
```
smv> undo
```

In command-line mode, you can use:
```bash
smv --undo
```

### Conflict Detection

SMV will not overwrite existing files unless explicitly instructed to do so, preventing accidental data loss.

## Requirements

- Rust (Minimum supported version: 1.85.0)
- GNU/Linux or other free operating system
- Standard system libraries

**Note**: SMV is developed exclusively for free operating systems. It is not officially tested or supported on proprietary platforms.

## Development

See [DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md) for the current roadmap and development priorities.

## ROADMAP

### Completed Features ✓

- **CNP Grammar Support**: Full Canopy ecosystem integration with filters, routes, and tool delegation
- **Advanced Filtering**: NAME:, TYPE:, EXT:, SIZE>, DEPTH<, MODIFIED>, ACCESSED< filters
- **Semantic Groups**: FOR:notes, FOR:media, FOR:scripts, FOR:projects, FOR:configs
- **Tool Delegation**: TO:tool routing for specialized operations
- **Output Routing**: INTO:file and FORMAT:json/csv/yaml support
- **DSC Integration**: Ultra-fast file discovery using DSC instead of broken glob patterns
- **New Command Structure**: Implemented LAR project command philosophy
- **Interactive Guidance System**: Excel-like command preview and help (designed)
- **Sequential Command Parsing**: Position-based argument validation
- **AFN REPL Integration**: Designed for use within AFN environment
- **Simplified Syntax**: No more `--` clutter after scope declaration

### High Priority Tasks

- [ ] **Interactive Guidance Implementation**: Build the Excel-like command preview system
- [ ] **F1 Help System**: Context-sensitive help for each command position
- [ ] **Tab Completion**: Cycle through valid options at each position
- [ ] **AFN Library Integration**: Extract shared command guidance into AFN library
- [ ] **Complete Tool Delegation**: Finalize integration with SAY, DFF, XFD tools
- [ ] **WHERE Filter Groups**: Implement logical grouping of multiple filters

### Future Enhancements

- [ ] **Multi-step transformation pipelines**: Chain multiple transforms together
  ```bash
  # Example: Clean, convert to snake_case, then replace spaces with underscores
  smv -pipeline "clean,snake,replace:space:_" file.txt
  ```

- [ ] **Regular expression-based transformations**: Advanced pattern replacement
  ```bash
  # Example: Replace "old" with "new" in filenames
  smv -replace "old:new" file.txt
  # Example: Replace all digits with 'X'
  smv -char-replace "digits:X" file*.txt
  ```

- [ ] **Custom user-defined transformations**: Save your own transform combinations
  ```bash
  # Example: Save a custom transformation
  smv -save "my-format:clean,snake,upper"
  # Use the custom transformation
  smv -custom my-format file.txt
  ```

- [ ] **Configuration file system**: Persistent settings and defaults
- [ ] **Directory synchronization features**: Keep renamed files in sync
- [ ] **Plugin system for add-ons**: Extend functionality
- [ ] **Enhanced REPL with syntax highlighting**: Improved interactive mode
- [ ] **Color-coded preview output**: Visual differentiation of changes

## License

MIT License
