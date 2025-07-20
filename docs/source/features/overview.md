# Features Overview

SMV provides powerful file management capabilities through intelligent transformations, CNP filtering, and safe operation modes.

## Core Features

### Intelligent File Transformations
- **Case transformations**: snake_case, kebab-case, PascalCase, camelCase, Title Case
- **String operations**: Find and replace with CHANGE command
- **Extension preservation**: Automatic handling of file extensions
- **Unicode support**: Proper handling of international characters

### CNP Filter System
- **Extension filtering**: `EXT:txt`, `EXT:md`
- **Type filtering**: `TYPE:file`, `TYPE:dir`
- **Size filtering**: `SIZE>1MB`, `SIZE<100KB`
- **Name pattern matching**: `NAME:test*`, `NAME:*backup`
- **Complex combinations**: Multiple filters with AND logic

### Safety and Preview
- **Preview mode**: `-p` flag shows what would happen without applying changes
- **Force mode**: `-F` flag for confirmed operations
- **Recursive operations**: `-r` flag for subdirectory processing
- **No-clobber protection**: `-n` flag prevents overwriting existing files

### File Operations
- **Enhanced move/copy**: Standard operations with SMV's filtering power
- **Intelligent deletion**: Remove files by pattern, size, or type
- **File creation**: Create files and directories with `-cf` and `-cd`
- **Batch operations**: Process multiple files efficiently

## Advanced Features

### Files-Only Default
- **Sensible defaults**: Operates on files only by default
- **Directory inclusion**: Use `-e` flag to include directories
- **Clear behavior**: No ambiguity about what will be affected

### Interactive Modes
- **REPL mode**: Interactive shell for multiple operations (planned)
- **TUI mode**: Terminal user interface for visual file management (planned)
- **Command history**: Remember previous operations

### Integration Features
- **CNP ecosystem**: Part of the larger CNP toolset
- **Shell integration**: Works seamlessly with existing workflows
- **Scriptable**: Perfect for automation and batch processing

## Safety Features

### Built-in Protections
- **Preview-first workflow**: Always preview before applying
- **Confirmation prompts**: Interactive confirmations for destructive operations
- **Backup awareness**: Warns about overwriting existing files
- **Error handling**: Graceful failure with helpful error messages

### Best Practices Support
- **Incremental operations**: Start small, expand scope gradually
- **Filter-first approach**: Target specific files before operating
- **Verbose feedback**: Clear output about what's happening

## Performance Features

### Efficient Processing
- **Smart traversal**: Efficient directory walking
- **Pattern optimization**: Fast pattern matching
- **Memory efficient**: Handles large directories without excessive memory use
- **Parallel processing**: Multi-threaded operations where beneficial

### Scalability
- **Large directories**: Handles thousands of files efficiently
- **Network filesystems**: Works with remote and networked storage
- **Low resource usage**: Minimal system impact during operations

## Future Features

### Planned Enhancements
- **REPL mode**: Interactive command environment
- **TUI mode**: Visual file browser with keyboard shortcuts
- **Undo system**: Reverse operations with operation history
- **Plugin system**: Extensible transformation modules
- **Git integration**: Smart handling of version-controlled files