# SMV - Smart Move

A powerful, Rust-based drop-in replacement for the standard Unix `mv` command with intelligent filename transformation capabilities.

## Features

- **Drop-in replacement** for the standard Unix `mv` command
- **Interactive REPL Interface** with command history and tab completion
- **Smart File Renaming** with multiple transformation strategies:
  - Convert to snake_case (`document-name.pdf` → `document_name.pdf`)
  - Convert to kebab-case (`document_name.pdf` → `document-name.pdf`)
  - Convert to Title Case (`document_name.pdf` → `Document Name.pdf`)
  - Convert to camelCase (`document_name.pdf` → `documentName.pdf`)
  - Convert to PascalCase (`document_name.pdf` → `DocumentName.pdf`)
  - Convert to lowercase or UPPERCASE
  - Clean up spaces and special characters
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

### Basic Usage (like standard mv)

SMV works exactly like the standard `mv` command for basic moving and renaming:

```bash
# Move a file to another location
smv file.txt /path/to/destination/

# Move multiple files to a directory
smv file1.txt file2.txt destination_directory/

# Rename a file
smv old_name.txt new_name.txt

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

## Full Command Options

```
Options:
  -i, --interactive              Launch interactive REPL interface
  -p, --preview                  Preview changes without applying them
  -r, --recursive                Process subdirectories recursively
  -e, --extensions <EXTENSIONS>  Comma-separated list of file extensions to process
  -a, --remove-accents           Remove accents from filenames
      --clean                    Clean up spaces and special characters
      --snake                    Convert filenames to snake_case
      --kebab                    Convert filenames to kebab-case
      --title                    Convert filenames to Title Case
      --camel                    Convert filenames to camelCase
      --pascal                   Convert filenames to PascalCase
      --lower                    Convert filenames to lowercase
      --upper                    Convert filenames to UPPERCASE
      --dry-run                  Same as preview - show changes without applying
      --undo                     Undo the last operation
      --exclude <PATTERNS>       Comma-separated patterns to exclude
      --group                    Group files by basename into directories
      --flatten                  Flatten all files from subdirectories into a single directory and remove empty directories
      --max-history-size <SIZE>  Maximum number of operations in history [default: 50]
  -h, --help                     Print help
  -V, --version                  Print version
```

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
- [ ] **Fix CLI glob pattern handling**: Make CLI mode correctly handle glob patterns like `*.org`
- [ ] **Fix kebab-case transformation**: Make kebab-case correctly convert spaces to hyphens

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
