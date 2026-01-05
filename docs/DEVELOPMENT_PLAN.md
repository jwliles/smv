---
date_created: 2025-10-02T11-25-26
date_updated: 2025-08-30T02-49-20
timestamp: 1759404326183
title: DEVELOPMENT_PLAN
id: 1acb6367-54e2-4a25-80a6-af4bb3b7660b
hash: 698589b06f8bd4cc7aee99eb1be07fb0c68a9a9809dc379bf1574d587e5a7657
---
# SMV Development Plan

This document outlines the development roadmap for SMV (Smart Move), organizing features and improvements into cohesive phases.

## ✅ Recently Completed

### Case-Delimiter Architecture Refactor (Phase 1)
**Status: Complete** (Branch: `feature/case-delimiter-architecture`)

- Implemented orthogonal case/delimiter design allowing any valid combination
- Added 10 case styles and 6 delimiter styles with validation
- Built transformation pipeline with extension preservation
- Added auto-detection for existing case styles and delimiters  
- Provided 5 key convenience functions for popular combinations
- Maintained 100% backward compatibility with existing TransformType
- Comprehensive test coverage (29/29 tests passing)

This provides the foundation for future CLI enhancements while keeping existing functionality intact.

## Core Interface Improvements

1. **TUI File Explorer with Vim Motions**
   - Create a terminal-based UI similar to Ranger/Yazi with file preview
   - Implement Vim-style navigation (hjkl, dd, visual mode selection)
   - Support batch selection across multiple directories
   - Add status line showing current mode and available commands

2. **Operation Queue System**
   - GParted-style operation queue showing planned transformations
   - Allow reordering, editing, and removal of queued operations
   - Pre-execution validation showing conflicts and potential issues
   - Support for saving queues as reusable templates

3. **Fuzzy Search Integration with Skim**
   - Integrate Skim for fuzzy file filtering and selection
   - Real-time filtering as users type
   - Preview transformations on matched files
   - Multi-select capability for batch operations

## Bug Fixes

1. **Kebab Case Transformation**
   - Fix spaces not converting to hyphens in kebab-case transformation
   - Update implementation to use pattern matching similar to other transformers
   - Add tests to verify space handling in all transformers

2. **CLI Glob Pattern Handling**
   - Replace rudimentary pattern matching with proper glob crate usage
   - Make CLI mode correctly handle glob patterns like `*.org`
   - Unify pattern handling between CLI and interactive modes

## Scripting & Automation

1. **YAML Script Format**
   - Define YAML structure following standard Linux CLI sequence:
     ```yaml
     operations:
       - source: "pattern/or/path"
         options:
           recursive: true
           snake_case: true
         destination: "target/path"
     ```
   - Support templates and variables `{year}`, `{month}`, etc.
   - Allow multiple operations in a single script

2. **Script Verification Tool**
   - Command to validate script syntax and logic
   - Simulate execution and report potential issues
   - Provide helpful error messages with fix suggestions

3. **Headless Mode for Automation**
   - Support for reading operations from files/stdin
   - Run without visual interface for cron jobs and scripts
   - Structured JSON output for integration with other tools

## User Experience Enhancements

1. **Command Correction**
   - Git-style command correction for typos
   - Suggest alternatives for unknown commands/options
   - Allow auto-correction with confirmation

2. **Smart PWD Handling**
   - Use current directory as default for missing source/destination
   - Clear indicators when operations will be performed in-place

3. **Visual Preview Enhancements**
   - Split view showing before/after transformations
   - Syntax highlighting for changed portions of filenames
   - Conflict warnings with visual indicators

## Implementation Organization

### Phase 1: Core Improvements
- Fix kebab-case transformer bug
- Integrate Skim for fuzzy search in interactive mode
- Design and implement basic TUI framework

### Phase 2: Queue and Operation Management 
- Implement operation queue system
- Add Vim-style motions for queue management
- Create visual preview system for transformation results

### Phase 3: Scripting and Automation
- Design and implement YAML script format
- Create script verification tool
- Develop headless mode for automation

### Phase 4: UX Refinements
- Add command correction system
- Enhance visual feedback and preview capabilities
- Implement smart defaults and PWD handling

## Technology Considerations

- **TUI Framework**: Use tui-rs or cursive for the terminal interface
- **Fuzzy Search**: Leverage existing skim installation
- **YAML Processing**: Use serde_yaml for parsing and validation
- **Shell Integration**: Consider readline/rustyline for command input