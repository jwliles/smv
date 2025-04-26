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

## Medium Priority Issues

### Transformers
- **Kebab Case Transformation** [#2]
  
  Does not convert spaces to hyphens (e.g., "Dir Template.txt" becomes "dir template.txt" instead of "dir-template.txt")
  
  Analysis:
  The issue is in the `kebab_case()` function in `src/transformers.rs`. Unlike the `title_case()` function that correctly processes spaces as word separators using `WORD_SEPARATORS_RE`, the kebab case implementation only replaces underscores with hyphens and doesn't handle spaces.
  
  Proposed fix:
  ```rust
  fn kebab_case(name: &str) -> String {
      // First split by word separators (spaces, underscores, hyphens)
      let words = WORD_SEPARATORS_RE
          .split(name)
          .filter(|s| !s.is_empty())
          .collect::<Vec<&str>>();
      
      // Join with hyphens and convert to lowercase
      words.join("-").to_lowercase()
  }
  ```
  
  Impact: Incorrect transformation output for a specific case type, undermining user expectations for kebab-case formatting.

## Low Priority Issues

- **Space Handling in Transformers** [#3]
  
  Need to review all case transformers for similar issues with space handling
  
  Impact: Potential inconsistencies in transformation outputs.

## Fixed Issues

*None recorded yet*

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