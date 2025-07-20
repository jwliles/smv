# SMV Roadmap

## Next Development Sessions (HIGH PRIORITY)

### 1. REPL Mode Implementation =�
**Solves CNP verbosity problem**

- **Goal**: Enable `smv repl` to drop `smv` prefix for familiar commands
- **Benefits**: 
  - `mv` instead of `smv mv` (back to 2-character commands)
  - All SMV power (CNP filters, preview, transformations) without verbosity
  - Session context for efficient multi-file operations
- **Implementation**:
  - Interactive shell with SMV command parsing
  - Command history and autocompletion
  - Session state management
  - Exit/quit commands

### 2. TUI Mode Enhancement <�
**Visual file management with keyboard efficiency**

- **Goal**: Full-featured file explorer with SMV operations
- **Benefits**:
  - Zero typing for file selection
  - Visual previews of transformations
  - Keyboard shortcuts (`d` delete, `c` copy, `m` move, etc.)
  - Safe operations with built-in confirmation
- **Implementation**:
  - File browser with navigation
  - Operation preview pane
  - Keyboard shortcut system
  - Batch operation support

## Core Features (COMPLETED )

### Files-Only Default Behavior
-  Default to files only (directories excluded)
-  `-e, --everything` flag to include directories
-  Comprehensive test coverage
-  Clear documentation

### Enhanced Help System
-  Tabular format with logical sections
-  Alphabetical flag sorting
-  Prominent default behavior explanation
-  Concise, progressive examples

### Flag Reorganization
-  `-F` for force operations
-  `-f` for file creation (so `-cf` creates files)
-  Updated tests and documentation

## Medium Priority Features

### Enhanced Transformation System ✓
- **Case Transformations**: snake, kebab, pascal, camel, title, lower, upper ✓
- **Split Functionality**: `split TRANSFORMATION` for camelCase/PascalCase boundary detection ✓
- **Custom Split Definitions**: User-defined word boundaries in config file for edge cases
- **String Operations**: CHANGE "old" INTO "new" with regex support ✓
- **Batch Operations**: Multiple transformations in sequence ✓
- **Undo Support**: Reverse operations with history ✓

### CNP Filter System
- **File Type Filters**: `TYPE:file`, `TYPE:dir`
- **Extension Filters**: `EXT:txt`, `EXT:md`
- **Size Filters**: `SIZE>1MB`, `SIZE<100KB`
- **Name Patterns**: `NAME:prefix*`, `NAME:*suffix`
- **Date Filters**: `MODIFIED>2024-01-01`

### Advanced File Operations
- **Preview Mode**: `-p` flag for all operations
- **Recursive Operations**: `-r` flag with depth control
- **Interactive Confirmations**: Safe operation defaults
- **Batch Processing**: Multiple files and directories

## Future Enhancements

### Performance & Scalability
- **Parallel Processing**: Multi-threaded operations for large directories
- **Progress Indicators**: Real-time feedback for long operations
- **Memory Optimization**: Efficient handling of large file sets
- **Streaming Operations**: Process files without loading all into memory

### Integration Features
- **Git Integration**: Smart handling of version-controlled files
- **Archive Support**: Operations on compressed files
- **Network Operations**: Remote file handling
- **Plugin System**: Extensible transformation modules

### Developer Experience
- **JSON/YAML Output**: Machine-readable operation results
- **Scripting Support**: Non-interactive batch operations
- **Configuration Files**: User preferences and defaults
- **Shell Completions**: Tab completion for all commands

## Design Principles

### CNP Philosophy Alignment
1. **Minimize Typing**: REPL mode solves verbosity issues
2. **Sensible Defaults**: Files-only by default, clear behavior
3. **Progressive Disclosure**: Simple commands with powerful options
4. **Safety First**: Preview mode, confirmations, undo support

### Code Quality Standards
- **100% Test Coverage**: All features thoroughly tested
- **Documentation**: Clear examples and use cases
- **Performance**: Efficient operations on large file sets
- **Error Handling**: Graceful failure with helpful messages

---

*Last Updated: July 2025*
*Next Review: After REPL/TUI implementation*