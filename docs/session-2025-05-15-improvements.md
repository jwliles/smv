# Development Session Summary - May 15, 2025

## Issues Identified

1. **Directory Handling Limitation**
   - SMV currently treats directories differently from files
   - Transformations don't apply to directory names with the same features
   - Need to make directories first-class objects in the transformation system

2. **CLI vs Interactive Mode Syntax Inconsistency**
   - CLI mode uses flags (e.g., `--snake`)
   - Interactive mode uses verbs (e.g., `snake`)
   - Two potential solutions:
     - Make REPL use flag-style commands (more complex)
     - Add verb-style aliases to CLI mode (simpler)

3. **Glob Pattern Handling Issue** (Previously Known)
   - CLI mode doesn't handle glob patterns like `*.org` correctly
   - Already identified in BUGS.md as a high-priority fix
   - Current workaround: Use interactive mode or explicitly list files

## Next Steps

1. Update BUGS.md to include the newly identified issues
2. Prioritize directory handling for the next development session
3. Consider aligning command syntax between CLI and interactive mode
4. Continue work on the existing glob pattern handling issue

## Technical Notes

- Directory handling changes would require updates to transformation functions to process directory entries
- Command syntax alignment would likely involve either:
  - Modifying the `execute_command` method in `repl.rs`
  - Adding verb-style command support in `main.rs` via the `clap` setup