# smv_case_and_delimiter_combination_architecture

## Overview

  Current SMV implementation uses a single `TransformType` enum with hardcoded variants like `Snake`, `Kebab`, `Title`, etc. This approach makes it difficult to support new combinations (e.g., `title-snake`, `studly-dot`) without adding many enum variants. The new design splits transformations into **two dimensions** and incorporates robust validation, defaults, and extension preservation:

  - **CaseStyle**: Capitalization rules (includes `Lower` for all-lowercase support and `Auto` for detection mode)
  - **DelimiterStyle**: Word separators (Snake, Kebab, Dot, Colon, Flat, Space) We also add:
  - Validation logic to prevent invalid combos (e.g., Camel only with Flat)
  - Default delimiter mapping for each case style
  - Extension preservation handling during transforms
  - Clear CLI mapping strategy (support composite and separate flags)
  - Error handling for invalid combinations or malformed input
  - Migration plan to phase legacy `TransformType`
  - Auto detection for both case styles and delimiters

  ------

## Enums

  ```rust
  pub enum CaseStyle {
      Sentence,
      Start,
      Title,
      Studly,
      StudlyReverse,
      Screaming,
      Camel,      // restricted to Flat delimiter
      Pascal,
      Lower,
      Auto,       // Detect and preserve existing capitalization
  }
  pub enum DelimiterStyle {
      Snake,   // "_"
      Kebab,   // "-"
      Dot,     // "."
      Colon,   // ":"
      Flat,    // "" (no delimiter)
      Space,   // " " (used for Title/Start/Sentence)
  }
  ```

  ------

## Validation Logic

  ```rust
  pub fn is_valid_combination(case: &CaseStyle, delimiter: &DelimiterStyle) -> bool {
      match (case, delimiter) {
          // Camel only works with Flat
          (CaseStyle::Camel, DelimiterStyle::Flat) => true,
          (CaseStyle::Camel, _) => false,
          _ => true,
      }
  }
  ```

  ------

## Default Delimiter Mapping

  ```rust
  impl CaseStyle {
      pub fn default_delimiter(&self) -> DelimiterStyle {
          match self {
              CaseStyle::Camel | CaseStyle::Pascal => DelimiterStyle::Flat,
              CaseStyle::Start | CaseStyle::Sentence | CaseStyle::Title => DelimiterStyle::Space,
              CaseStyle::Lower | CaseStyle::Screaming => DelimiterStyle::Snake,
              CaseStyle::Studly | CaseStyle::StudlyReverse => DelimiterStyle::Flat,
              CaseStyle::Auto => DelimiterStyle::Snake,
          }
      }
  }
  ```

  ------

## Auto Detection Implementation

  ```rust
  impl CaseStyle {
      pub fn detect_from_input(input: &str) -> Self {
          let words = tokenize(input, true);
          if words.is_empty() { return CaseStyle::Lower; }
          
          // Detection logic
          if is_camel_case(&words) { CaseStyle::Camel }
          else if is_pascal_case(&words) { CaseStyle::Pascal }
          else if is_screaming(&words) { CaseStyle::Screaming }
          else if is_title_case(&words) { CaseStyle::Title }
          else if is_sentence_case(&words) { CaseStyle::Sentence }
          else if is_start_case(&words) { CaseStyle::Start }
          else if is_studly_case(&words) { CaseStyle::Studly }
          else { CaseStyle::Lower } // fallback
      }
  }
  
  impl DelimiterStyle {
      pub fn detect_from_input(input: &str) -> Self {
          if input.contains('_') { DelimiterStyle::Snake }
          else if input.contains('-') { DelimiterStyle::Kebab }
          else if input.contains('.') { DelimiterStyle::Dot }
          else if input.contains(':') { DelimiterStyle::Colon }
          else if input.contains(' ') { DelimiterStyle::Space }
          else { DelimiterStyle::Flat }
      }
  }
  ```

  ------

## Pipeline Flow

  1. Split Words
     - Tokenize input string into words using delimiters or case transitions
  2. Apply CaseStyle
     - Transform tokens according to capitalization rules (Sentence, Start, Title, Studly, etc.)
     - Use auto-detection if CaseStyle::Auto is specified
  3. Validate and Join
     - Check combination validity (e.g., Camel requires Flat)
     - Use default delimiter if none specified
     - Combine tokens with chosen delimiter
  4. Preserve Extensions
     - Split filename from extension before pipeline
     - Transform only the basename
     - Recombine with lowercase extension (e.g.,.TXT ->.txt)

  ------

## Example Usage

  ```rust
  transform("TheQuickBrownFox", CaseStyle::Title, DelimiterStyle::Snake);
  // -> "The_Quick_Brown_Fox"
  
  transform("theQuickBrownFox", CaseStyle::StudlyReverse, DelimiterStyle::Dot);
  // -> "ThE.QuIcK.BrOwN.FoX"
  
  // Auto detection examples
  transform("myFileName", CaseStyle::Auto, DelimiterStyle::Kebab);
  // -> "my-file-name" (detected camel, changed to kebab)
  
  transform("snake_case_file", CaseStyle::Pascal, DelimiterStyle::Auto);
  // -> "SnakeCaseFile" (detected snake, changed to pascal/flat)
  ```

  ------

## CLI Mapping

  - Composite flags: `--title-snake`, `--studly-dot`
  - Separate flags: `--case=title --delimiter=snake`
  - Hybrid for backward compatibility: `--snake` maps to (Lower, Snake)
  - Auto mode flags: `--auto-kebab`, `--pascal-auto`, `--auto-auto`

### CLI Auto Mode Usage

  ```bash
  smv --auto-kebab file.txt    # Keep case, change to kebab
  smv --pascal-auto file.txt   # Change to pascal, keep delimiter  
  smv --auto-auto file.txt     # Detect both (useful for normalization)
  ```

  ------

## Migration Plan

  1. Add new Case+Delimiter enums alongside legacy `TransformType`
  2. Provide mapping from legacy variants to new system
  3. Refactor transformation logic to use pipeline and validation
  4. Implement extension preservation in pipeline
  5. Remove legacy variants after transition period

### Migration Example

  ```rust
  // Legacy usage
  transform("file.txt", &TransformType::Snake);
  
  // New usage  
  transform("file.txt", CaseStyle::Lower, DelimiterStyle::Snake);
  
  // During migration period, both work through compatibility layer
  ```

  ------

## Error Handling

  ```rust
  pub enum TransformError {
      InvalidCombination(CaseStyle, DelimiterStyle),
      InvalidRegex(String),
      EmptyInput,
      DetectionFailed(String),
  }
  ```

  ------

## Benefits

  - Comprehensive case/delimiter combos with validation and defaults
  - Easier addition of new styles and delimiters
  - Clear CLI mapping and backward compatibility
  - Extension-safe transformations with robust error handling
  - Supports Auto mode for preserving existing case while changing delimiters
  - Smart detection capabilities for both case styles and delimiters
  - Flexible usage patterns (auto-kebab, pascal-auto, auto-auto)

## Implementation Status

### ✅ Phase 1 Complete (Branch: feature/case-delimiter-architecture)

**Core Architecture Implemented:**
- ✅ `CaseStyle` enum with 10 variants (Sentence, Start, Title, Studly, StudlyReverse, Screaming, Camel, Pascal, Lower, Auto)
- ✅ `DelimiterStyle` enum with 6 variants (Snake, Kebab, Dot, Colon, Flat, Space)  
- ✅ `TransformError` enum with comprehensive error handling
- ✅ Validation logic for invalid combinations (e.g., Camel + Snake)

**Core Pipeline:**
- ✅ `transform_with_pipeline()` - main transformation function
- ✅ Extension preservation with edge case handling (.hidden files, trailing dots)
- ✅ Auto-detection for both case styles and delimiter styles
- ✅ Default delimiter mapping for each case style

**Key Combinations Available:**
- ✅ `transform_title_snake()` - Title case with underscores  
- ✅ `transform_lower_kebab()` - Lowercase with hyphens
- ✅ `transform_pascal_dot()` - PascalCase with dots
- ✅ `transform_screaming_snake()` - UPPERCASE with underscores
- ✅ `transform_studly_dot()` - aLtErNaTiNg case with dots

**Backward Compatibility:**
- ✅ `legacy_transform_to_pipeline()` - bridges old TransformType to new system
- ✅ All existing functionality preserved and working
- ✅ Comprehensive test suite (29/29 tests passing)

**Next Phases:**
- [ ] Phase 2: CLI Integration with composite flags (`--title-snake`, `--lower-kebab`)
- [ ] Phase 3: Auto-detection CLI flags (`--auto-kebab`, `--pascal-auto`)
