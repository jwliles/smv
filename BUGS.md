# SMV Bug Tracking

## Table of Contents
- [High Priority Issues](#high-priority-issues)
- [Medium Priority Issues](#medium-priority-issues)
- [Low Priority Issues](#low-priority-issues)
- [Fixed Issues](#fixed-issues)

## High Priority Issues

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
  
  Impact: Incorrect transformation output for a specific case type.

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