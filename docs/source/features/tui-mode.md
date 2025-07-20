# TUI Mode (Planned)

Visual file explorer with keyboard-driven operations.

## Overview

TUI (Terminal User Interface) mode provides a visual file management interface with keyboard shortcuts for efficient file operations.

## Benefits

### Visual File Management
- **File browser**: Navigate directories with arrow keys
- **Preview pane**: See file transformations before applying
- **Multi-pane layout**: Source and destination views
- **Visual feedback**: Real-time preview of operations

### Keyboard Efficiency
```
d     Delete selected files
c     Copy files
m     Move files
r     Rename/transform files
t     Transform case (snake, kebab, etc.)
p     Preview mode toggle
/     Search files
```

### Zero Typing for Selection
- **Arrow navigation**: Move through files and directories
- **Space selection**: Select multiple files
- **Pattern selection**: Select by extension or pattern
- **Batch operations**: Apply operations to selected files

## Planned Features

### File Browser
- **Tree navigation**: Hierarchical directory view
- **Dual pane**: Source and destination panels
- **File details**: Size, date, permissions
- **Hidden files**: Toggle visibility with 'h'

### Operation Modes
- **Transform mode**: Apply case transformations visually
- **Move mode**: Visual file moving with destination selection
- **Delete mode**: Safe deletion with confirmation
- **Search mode**: Find files by name or pattern

### Interactive Preview
```
┌─ Source ─────────────────┬─ Preview ─────────────────┐
│ my_file_name.txt        │ my-file-name.txt         │
│ ANOTHER_FILE.md         │ another-file.md          │
│ TestDocument.pdf        │ test-document.pdf        │
└─────────────────────────┴──────────────────────────┘
[p]review [a]pply [c]ancel [ESC]exit
```

### Batch Operations
- **Multi-select**: Select files across directories
- **Pattern select**: Select by extension or name pattern
- **Operation queue**: Queue multiple operations
- **Undo support**: Reverse operations

## Keyboard Shortcuts

### Navigation
```
↑↓        Navigate files
←→        Navigate directories  
Enter     Enter directory / Preview file
Backspace Parent directory
Home/End  First/last file
PgUp/PgDn Page up/down
```

### Selection
```
Space     Toggle file selection
a         Select all
A         Select none
*         Invert selection
/         Search files
```

### Operations
```
d         Delete selected
c         Copy selected  
m         Move selected
r         Rename/transform
t         Transform case menu
p         Toggle preview mode
```

### Transformation Menu
```
s         snake_case
k         kebab-case
p         PascalCase
c         camelCase
T         Title Case
l         lowercase
U         UPPERCASE
```

## Use Cases

### Visual File Organization
1. **Browse** to directory with files to organize
2. **Select** files by pattern or manually
3. **Preview** transformations in real-time
4. **Apply** when satisfied with preview

### Safe File Cleanup
1. **Navigate** to directory with files to clean
2. **Search** for specific patterns (*.tmp, *.log)
3. **Select** files matching criteria
4. **Preview** deletion operation
5. **Confirm** and apply

### Batch Renaming
1. **Select** files to rename
2. **Choose** transformation type
3. **Preview** all changes simultaneously
4. **Apply** batch transformation

## Implementation Status

🚧 **In Development**: TUI mode is a high-priority feature planned for the next development sessions.

### Planned Architecture
- **Ratatui framework**: Modern terminal UI library
- **Event handling**: Keyboard and mouse input
- **State management**: Track selections, operations, and preview
- **Integration**: Reuse SMV's transformation and filtering logic

### Timeline
- **Phase 1**: Basic file browser with navigation
- **Phase 2**: Operation preview and application
- **Phase 3**: Advanced features and keyboard shortcuts