# SMV - Smart Move

> **⚠️ EARLY DEVELOPMENT**: This project is in early development and not yet ready for production use. Features described in this README are planned but may not be fully implemented yet.

SMV (Smart Move) is an enhanced `mv` command line utility for batch renaming and moving files with transformation capabilities. It provides powerful file renaming operations with preview mode, undo functionality, and an interactive REPL interface.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## 📋 Overview

SMV enhances the standard `mv` command with powerful filename transformation capabilities:

- **Drop-in replacement** for the standard `mv` command
- **Transform filenames** to various formats (snake_case, camelCase, etc.)
- **Preview changes** before applying them
- **Undo operations** with built-in history management
- **Interactive mode** with a user-friendly REPL
- **Batch operations** with glob pattern support

## ⚡ Quick Examples

```bash
# Standard move operation
smv source.txt destination/

# Transform filenames to snake_case
smv --snake "My Document.txt" "Another File.txt"
# Result: my_document.txt, another_file.txt

# Preview kebab-case transformation without applying
smv --kebab --preview *.pdf

# Apply PascalCase recursively to JS files
smv --pascal --recursive src/*.js
```

## 📦 Installation

> ⚠️ This project is not yet published to crates.io. Installation from source is the only option at this time.

### From source

```bash
git clone https://github.com/jwliles/smv.git
cd smv
cargo build --release
```

The compiled binary will be available at `target/release/smv`.

### Requirements

- Rust 1.65.0 or newer
- If using nightly Rust, a modification to clap dependencies is needed (see [Build Issues](docs/BUILD_ISSUES.md))

### Verify installation

```bash
smv --version
```

## 🚀 Features

### Core Features

- **Drop-in replacement** for standard `mv` command
- **Preview mode** to see changes before applying them
- **Undo capability** to revert operations
- **Interactive REPL** with command history
- **Recursive operation** on directories
- **File filtering** by extension or pattern
- **Safety features** including backups and conflict detection

### Filename Transformations

| Transform | Description | Example |
|-----------|-------------|---------|
| `clean` | Remove special characters | `My File (1).txt` → `My File 1.txt` |
| `snake` | Convert to snake_case | `MyFile.txt` → `my_file.txt` |
| `kebab` | Convert to kebab-case | `MyFile.txt` → `my-file.txt` |
| `title` | Convert to Title Case | `my_file.txt` → `My File.txt` |
| `camel` | Convert to camelCase | `my_file.txt` → `myFile.txt` |
| `pascal` | Convert to PascalCase | `my_file.txt` → `MyFile.txt` |
| `lower` | Convert to lowercase | `MyFile.txt` → `myfile.txt` |
| `upper` | Convert to UPPERCASE | `myFile.txt` → `MYFILE.TXT` |

## 📖 Usage

### Basic Usage

```bash
# Standard move operation (like the mv command)
smv source.txt destination.txt
smv file1.txt file2.txt directory/

# Apply a transformation
smv --snake *.txt          # Convert to snake_case
smv --kebab *.png          # Convert to kebab-case
smv --clean "My File*.txt" # Clean up special chars
smv --pascal src/*.rs      # Convert to PascalCase
```

### Command Line Options

```
USAGE:
    smv [OPTIONS] [SOURCE]... [DESTINATION]

ARGS:
    <SOURCE>...      Files or patterns to move/rename
    <DESTINATION>    Destination file or directory

OPTIONS:
    -i, --interactive            Interactive mode - launch REPL interface
    -p, --preview                Preview changes without applying them
    -r, --recursive              Process subdirectories recursively
    -e, --extensions <EXTS>      Filter by file extensions (comma-separated)
    -a, --remove-accents         Remove accents from filenames
        --clean                  Convert to clean format
        --snake                  Convert to snake_case
        --kebab                  Convert to kebab-case
        --title                  Convert to Title Case
        --camel                  Convert to camelCase
        --pascal                 Convert to PascalCase
        --lower                  Convert to lowercase
        --upper                  Convert to UPPERCASE
        --dry-run                Same as preview
        --exclude <PATTERNS>     Exclude patterns (comma-separated)
        --max-history-size <N>   Max operations in history [default: 50]
    -h, --help                   Print help information
    -V, --version                Print version information
```

### 🔍 Example Scenarios

#### Basic Move Operations

```bash
# Move a file to a directory
smv document.txt archive/

# Move multiple files
smv file1.txt file2.txt backup/

# Rename a file
smv old-name.txt new-name.txt
```

#### Filename Transformations

```bash
# Clean up messy filenames
smv --clean "My File (1)!!.txt" "Test & Demo.pdf"
# Result: "My File 1.txt", "Test Demo.pdf"

# Convert to snake_case with preview
smv --snake --preview "UserData.json" "ConfigFile.yaml"
# Shows: "user_data.json", "config_file.yaml" (without renaming)

# Convert all .js files to PascalCase recursively
smv --pascal --recursive src/*.js
# All .js files in src/ and subdirectories will use PascalCase

# Only process specific extensions
smv --title --extensions jpg,png images/
# Only .jpg and .png files will be converted to Title Case
```

#### Advanced Filtering

```bash
# Rename all .txt files except those with "backup" in the name
smv --snake --exclude "backup" *.txt

# Process only image files recursively
smv --clean --extensions jpg,png,gif --recursive media/
```

### 💻 Interactive Mode

Launch the interactive REPL with:

```bash
smv -i
```

This opens a command prompt where you can:

```
smv:/current/directory> help

Commands:
  preview <transform> <files>  - Show transformation without applying
  apply <transform> <files>    - Apply transformation
  rename <files> --options     - Interactive renaming wizard
  undo                         - Revert last operation
  cd <directory>               - Change directory
  ls [pattern]                 - List files
  help                         - Show this help
  quit                         - Exit program

Transformations:
  clean, snake, kebab, title, camel, pascal, lower, upper
```

#### Interactive Examples

```
# Preview a transformation
smv:/home/user> preview snake *.txt
Preview: "File Name.txt" → "file_name.txt"

# Apply a transformation
smv:/home/user> apply pascal *.js
Renamed: "user-model.js" → "UserModel.js"

# Start interactive renaming wizard
smv:/home/user> rename *.jpg
```

## 🛡️ Safety Features

### Backups

SMV automatically creates backups in `~/.config/smv/backups/`, enabling the undo functionality to work even after the program has been closed and reopened.

### Conflict Detection

SMV will not overwrite existing files unless explicitly instructed to do so, preventing accidental data loss.

## 🔮 Current Status & Roadmap

### Currently Implemented Features

- Basic `mv`-like functionality for moving files
- Filename transformations (snake_case, kebab-case, etc.)
- Preview mode to see changes before applying
- Interactive mode with REPL interface
- History tracking for undo support

### In Active Development (Next Release)

- **Transform Pipelines**: Chain multiple transformations
  ```bash
  smv -p "clean,snake,rep:space:_" file.txt
  ```
- **Enhanced Character Replacement**: Advanced pattern matching
  ```bash
  smv -R "old:new" file.txt
  smv -C "digits:X" file*.txt
  ```
- **Custom Transformations**: Save your own transformations
  ```bash
  smv --save "my-format:clean,snake,upper"
  smv -t my-format file.txt
  ```

### Development Roadmap

Our current development timeline:

1. **Phase 1: Core Foundation** (Completed)
   - Basic transformation functionality
   - Standard mv replacement capabilities 
   - History and undo support
   - Interactive REPL interface
   - Preview mode

2. **Phase 2: Advanced Features** (In Progress)
   - Transform pipelines to chain operations
   - Enhanced character replacement with patterns
   - Custom user-defined transformations
   - Short command aliases for reduced verbosity

3. **Phase 3: User Experience Improvements** (Planned)
   - Templating system for common rename patterns
   - Conditional transformations with rules
   - Enhanced REPL with tab completion
   - Plugin system for extensibility

4. **Phase 4: Integrations** (Future)
   - Configuration file support
   - Directory synchronization
   - Version control system awareness
   - API for third-party tools

5. **Phase 5: Stabilization** (Future)
   - Comprehensive testing
   - Performance optimization
   - Security review
   - Documentation finalization
   - v1.0.0 release

### Feature Prioritization

We prioritize features based on:
1. Foundational architecture needs
2. User request frequency
3. Implementation complexity
4. Potential impact on workflow efficiency

### Post-1.0 Considerations

- GUI interface development
- Remote file system integration
- Cloud storage service connectors
- Machine learning-based suggestions
- Community template repository

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📚 Documentation

For more detailed information, see:

- [Documentation Guide](docs/DOCUMENTATION.md) - Overview of project docs
- [Roadmap](docs/ROADMAP.md) - Development roadmap and planned features
- [Build Issues](docs/BUILD_ISSUES.md) - Solutions for common build problems
- [Changelog](CHANGELOG.md) - Version history and changes