# SMV TUI Queue-Based Action System Design Document

**Version**: 1.0  
**Date**: 2025-07-12  
**Author**: Design Discussion  
**Status**: Draft  

## 1. Overview

This document outlines the design for a queue-based action system in SMV's Terminal User Interface (TUI). The system enables users to build complex, multi-directory file reorganization workflows by queuing operations for batch execution, similar to GParted's operation queue.

## 2. Core Concept

Users can browse the file tree, mark files and directories for various operations, preview the complete transformation plan, and execute all operations sequentially. This transforms complex file reorganization from an error-prone manual process into a planned, reviewable workflow.

## 3. User Workflow

### 3.1 Basic Flow
1. **Browse** file tree using TUI
2. **Mark files** for operations (move, rename, delete)
3. **Review queue** showing all planned operations
4. **Reorder operations** if needed for dependency resolution
5. **Preview final state** before execution
6. **Execute** operations sequentially (FIFO by default)

### 3.2 Example Scenario
```
Downloads/old-projects/     → Queue: Move to ~/Archive/Projects/
Documents/screenshots/      → Queue: Rename pattern *.png → screen-{date}-{num}.png  
Desktop/temp-files/         → Queue: Delete after review
Photos/duplicates/          → Queue: Move to ~/Trash/
```

## 4. Visual Design

### 4.1 Virtual Text Overlays
Operations are displayed directly in the file explorer using virtual text overlays, eliminating the need for separate preview panes.

#### 4.1.1 Visual Language
```
📁 Projects/
├── ~~old-project.zip~~ ❌ [DELETE]
├── current-work/ → ~/Active/current-work/ ➡️ [MOVE]  
├── backup.tar.gz → backup-2024-07-12.tar.gz 📝 [RENAME]
└── temp/ ❌ [DELETE DIR + 15 files]
```

#### 4.1.2 Operation Encoding
- **Deletions**: Strikethrough + red tint + ❌
- **Moves**: Arrow with destination path + ➡️
- **Renames**: `old-name → new-name` + 📝
- **Bulk operations**: Show count like `+ 15 files`

### 4.2 Delta View for Long Paths
Only show path changes from the divergence point to save space:
```
Projects/old-project.zip → Archive/old-project.zip
Projects/backup/ → ∆Archive/backup/
Photos/IMG_001.jpg → ∆Screenshots/screenshot-001.jpg
```

### 4.3 Adaptive UI Density
```
Normal:  ~~old-file.txt~~ ❌ [DELETE]
Crowded: old-file.txt ❌
Dense:   old-file.txt ❌ (popup on hover shows full operation)
```

## 5. Navigation and Controls

### 5.1 Vim-Style Navigation
```
Ctrl+W h/j/k/l  - Switch between explorer/queue panes
Tab/Shift+Tab   - Cycle through panes  
g g / G         - Jump to top/bottom of queue
/ ?             - Search within queue operations
d d             - Delete operation from queue
y y / p         - Yank/paste operations (reorder)
```

### 5.2 Queue Manipulation
- **Live reordering**: Move operations up/down with `k`/`j`
- **Batch selection**: Select multiple operations for group moves
- **Undo/redo**: Full history of queue modifications
- **Collapsible groups**: Tree-style view for complex multi-file operations

## 6. Operation Validation

### 6.1 Live Filesystem Validation
The system performs real-time validation using DSC for:
- **Path existence**: Destination directories must exist
- **Permission checking**: Write permissions for target locations
- **Name conflicts**: Detect filename collisions
- **Circular dependencies**: Prevent impossible filesystem states

### 6.2 Impossible State Prevention
Rather than complex dependency resolution, the system prevents impossible states:
- Cannot delete directory that has queued moves into it
- Cannot move file to location queued for deletion  
- Cannot rename to name that conflicts with queued operations

### 6.3 Contextual Error Messages
```
❌ Cannot delete "Projects/"
   Directory is queued to receive:
   • old-backup.zip → Projects/Archive/old-backup.zip
   • temp-files/ → Projects/temp-files/
   
   Remove these move operations first.
```

## 7. Operation Precedence

### 7.1 Positive Operations (Container → Contents)
```
1. CREATE directories     (mkdir ~/Archive/Projects/)
2. RENAME directories     (mv Projects/ → Archive/)
3. MOVE directories       (mv old-stuff/ → Archive/old-stuff/)  
4. CREATE files          (touch new-file.txt)
5. RENAME files          (mv report.txt → final-report.txt)
6. MOVE files            (mv file.txt → Archive/file.txt)
```

### 7.2 Negative Operations (Contents → Container)  
```
7. DELETE files          (rm Archive/old-file.txt)
8. DELETE directories    (rmdir Archive/)  # Only when empty
```

### 7.3 Auto-Reordering
- System can auto-sort operations by precedence before execution
- Show reordered preview with highlighted changes
- Allow manual override for advanced users

## 8. Directory Creation Enhancement

### 8.1 Problem Statement
Current `mkdir -p` flag conflicts with SMV's universal `-p` preview convention.

### 8.2 Solution: Default Parent Creation
```bash
smv mkdir /deep/nested/path         # Creates all parents (default)
smv mkdir /deep/nested/path --np    # Fails if parents don't exist  
smv mkdir /deep/nested/path -p      # Preview what would be created
```

### 8.3 Rationale
- **Default behavior matches intent**: Users want directories created
- **Preserves SMV consistency**: `-p` remains universal preview
- **Rare case gets the flag**: `--np` for uncommon strict checking
- **Reduces typing**: Most common case requires no flags

## 9. Technical Architecture

### 9.1 State Management
- **Virtual Filesystem State**: Maintain predicted filesystem state as queue executes
- **Real-time Validation**: Recalculate validity as operations are reordered
- **Transaction Semantics**: Treat queue as single logical transaction

### 9.2 DSC Integration
- **Instant filesystem scanning** for path validation
- **Permission checking** before queue execution
- **Existence validation** for all operations
- **Name conflict detection** across operations

### 9.3 Queue Data Structure
```rust
struct OperationQueue {
    operations: Vec<QueuedOperation>,
    virtual_state: FilesystemState,
    validation_cache: HashMap<OperationId, ValidationResult>,
}

enum QueuedOperation {
    Move { from: PathBuf, to: PathBuf, id: OperationId },
    Delete { path: PathBuf, id: OperationId },
    Rename { from: PathBuf, to: PathBuf, id: OperationId },
    CreateDir { path: PathBuf, id: OperationId },
}
```

## 10. Safety Features

### 10.1 Preview Everything
- **Complete queue preview** before execution
- **Virtual filesystem state** prediction
- **Undo capability** for completed operations where possible

### 10.2 Atomic Execution
- **Stop on first error** with partial rollback option
- **Log all operations** for audit trail
- **Backup critical operations** where feasible

## 11. Future Enhancements

### 11.1 Advanced Features
- **Operation templates**: Save common reorganization patterns
- **Conditional operations**: Execute based on file properties
- **Parallel execution**: Safe operations in parallel where possible
- **External tool integration**: Queue operations for other tools

### 11.2 UI Improvements  
- **Visual diff view**: Before/after filesystem state
- **Progress indicators**: Real-time execution progress
- **Operation statistics**: Files moved, space saved, etc.

## 12. Implementation Priority

### 12.1 Phase 1: Core Queue System
- Basic operation queuing (move, rename, delete)
- Virtual text overlays in file explorer
- Simple validation and error messages

### 12.2 Phase 2: Advanced Navigation
- Vim-style navigation and controls
- Queue reordering and manipulation
- Live validation with DSC integration

### 12.3 Phase 3: Polish and Safety
- Precedence-based auto-reordering
- Comprehensive error handling
- Undo/redo functionality

---

**End of Document**

This design creates a powerful, intuitive system for complex file operations while maintaining SMV's principles of safety, transparency, and user control.