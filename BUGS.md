# SMV Bug Tracking

## Table of Contents
- [High Priority Issues](#high-priority-issues)
- [Medium Priority Issues](#medium-priority-issues)
- [Low Priority Issues](#low-priority-issues)
- [Fixed Issues](#fixed-issues)

## High Priority Issues

### Feature Enhancements

- **Interactive Mode with Fuzzy Search** [#4]

  The current interactive mode uses basic shell-like navigation which could be enhanced with fuzzy search capabilities.
  
  Analysis:
  The project can integrate the Skim crate (Rust port of fzf) to provide an interactive, fuzzy search-based file selection UI, significantly improving the efficiency of file selection.
  
  Implementation:
  ```rust
  // Add to Cargo.toml dependencies:
  // skim = "0.9"
  
  // In repl.rs:
  use skim::{Skim, SkimOptionsBuilder};
  
  impl InteractiveSession {
      // New method for fuzzy file selection
      fn fuzzy_select_files(&self) -> Vec<PathBuf> {
          let entries = fs::read_dir(&self.current_dir)
              .unwrap_or_else(|_| Vec::new())
              .filter_map(|entry| entry.ok())
              .map(|entry| entry.path())
              .collect::<Vec<PathBuf>>();
          
          // Setup skim options
          let options = SkimOptionsBuilder::default()
              .height(Some("50%"))
              .multi(true)
              .build()
              .unwrap();
          
          // Run skim and get selected files
          // Implementation details...
      }
  }
  ```
  
  Impact: Would significantly improve user experience by making file selection faster and more intuitive.

### CLI Mode
- **Glob Pattern Handling** [#1]
  
  CLI mode doesn't properly handle glob patterns like `*.org`. The pattern matching implementation in `main.rs` is rudimentary compared to the proper glob handling in interactive mode.

  Symptoms:
  ```
  $ smv -p --title *.org
  Error: "Failed to read directory : No such file or directory (os error 2)"
  
  $ smv -p --title .
  Error: "No source files specified for transformation"
  ```

  Analysis:
  The issue is in the `path_matches_pattern()` function in `main.rs` which uses a very simple regex-based pattern matching algorithm that doesn't correctly handle shell glob expansion. The interactive mode correctly uses the `glob` crate for pattern matching, but the CLI mode does not.
  
  Proposed fix:
  ```rust
  // Add to imports
  use glob::glob;
  
  // Replace the path_matches_pattern function
  fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
      // If the pattern is a directory, any file in it matches
      if Path::new(pattern).is_dir() {
          return true;
      }
      
      // Use the glob crate for pattern matching
      let abs_pattern = if Path::new(pattern).is_absolute() {
          pattern.to_string()
      } else {
          // Make relative patterns work from current dir
          format!("./{}", pattern)
      };
      
      // Try to match the file against the pattern
      glob(&abs_pattern)
          .map(|entries| {
              entries.filter_map(Result::ok)
                  .any(|matched_path| matched_path == path)
          })
          .unwrap_or(false)
  }
  ```

  Workaround:
  - Use interactive mode (currently the only working solution):
    ```
    $ smv -i
    smv:/path/to/dir> preview title *.org
    ```
     
  Note: No CLI mode commands have been found to work correctly with glob patterns. Neither using recursive flag with extensions filter nor specifying files individually works reliably. Interactive mode is currently the only working solution for handling file patterns.

  Impact: Severely limits usability of the CLI mode, forcing users to use interactive mode for basic operations.

- **Directory Transformation Support** [#5]

  SMV doesn't treat directories the same as files for transformations. Directories should be first-class objects that can be renamed using the same transformation options available for files.

  Symptoms:
  - Directory names aren't transformed when using options like `--snake` or `--kebab`
  - Directories are primarily treated as containers, not objects that can be transformed themselves

  Analysis:
  The transformation functions need to be updated to process directory entries in the same way as file entries. Currently, the code focuses primarily on file operations, with directories serving mostly as containers.

  Proposed fix:
  - Update transformation functions to process both file and directory entries
  - Ensure path-handling logic applies transformations to directory names
  - Extend test cases to verify directory name transformations

  Workaround:
  - Explicitly specify directory names for renaming operations

  Impact: Limits functionality and creates inconsistent user experience when working with both files and directories.

## Medium Priority Issues

### Transformers

- **CLI and Interactive Mode Syntax Inconsistency** [#6]

  There's a significant syntax difference between CLI mode and interactive mode commands, creating a confusing user experience.

  Symptoms:
  - CLI: Uses flags like `--snake`, `--kebab`
  - Interactive: Uses verbs like `snake`, `kebab`

  Analysis:
  The inconsistency stems from different parsing implementations:
  - CLI mode uses the `clap` library with standard flags
  - Interactive mode uses a simpler string parser with command verbs

  Proposed solutions:
  1. Make REPL use flag-style commands (more complex):
     ```
     smv> --snake *.txt  (instead of current: snake *.txt)
     ```

  2. Add verb-style aliases to CLI (easier):
     ```
     smv snake *.txt  (in addition to current: smv --snake *.txt)
     ```

  Workaround:
  - Users must remember different syntax for each mode

  Impact: Increases cognitive load and learning curve for users, reducing usability.

## Low Priority Issues

*No active low priority issues*

## Fixed Issues

### v0.3.0 Fixes

- **✅ Kebab Case Transformation** [#2] 
  
  Fixed spaces not being converted to hyphens. Now "Dir Template.txt" correctly becomes "dir-template.txt".
  
  Resolution: Refactored all transformers to use shared tokenization logic with proper space handling.

- **✅ Space Handling in Transformers** [#3]
  
  Fixed inconsistent space handling across all transformation types.
  
  Resolution: Implemented unified `tokenize()` function with `split_camel_case_word()` helper that properly handles spaces, camelCase, and edge cases like "XMLDocument" → "XML Document".

- **✅ Directory Transformation Support** [#5]
  
  Added support for transforming directory names using the same transformation options as files.
  
  Resolution: Updated `build_file_list()` to include directories and modified path processing logic to handle both files and directories consistently.

- **✅ Transformer Code Duplication** 
  
  Eliminated ~100 lines of duplicated preprocessing logic across transformation functions.
  
  Resolution: Created shared tokenization architecture with separate formatters (`format_snake()`, `format_kebab()`, etc.) that eliminate redundancy while improving maintainability.

- **✅ Transform Mode Inference**
  
  Removed requirement for explicit `--transform` flag when using transformation types.
  
  Resolution: Modified argument parsing to automatically infer transform mode when any transformation flag (`--snake`, `--kebab`, etc.) is used, making the CLI more intuitive.

---

## Issue Classification Guide

Issues are classified based on the following criteria:

**High Priority**
- Breaks core functionality
- No reliable workaround available
- Affects most users
- Blocks common use cases

**Medium Priority**
- Partial functionality breakage
- Has workable workarounds
- Affects specific use cases
- Incorrect but not fatal behavior

**Low Priority**
- Minor issues
- Cosmetic problems
- Rare edge cases
- Enhancement requests