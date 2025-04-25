# SMV Enhancement Design Document

## Overview

This document provides detailed technical specifications for the planned enhancements to the Smart Move (smv) utility. It focuses on architecture, component design, and implementation details.

## Architecture Overview

SMV will use a modular, trait-based architecture to support extensibility and maintainability:

```
                   ┌───────────────┐
                   │     Main      │
                   │  Application  │
                   └───────┬───────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │     CLI     │ │ Transformation│ │    File    │
    │  Interface  │ │    Engine    │ │ Operations │
    └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │   Command   │ │Transformation│ │   History   │
    │  Processor  │ │  Registry   │ │   Manager   │
    └─────────────┘ └─────────────┘ └─────────────┘
```

## Core Components

### 1. Transformation System

The transformation system will be redesigned to use a trait-based approach:

```rust
/// Core trait for all transformations
pub trait Transformation {
    /// Apply the transformation to an input string
    fn transform(&self, input: &str) -> String;
    
    /// Get the name of the transformation
    fn name(&self) -> &str;
    
    /// Get a description of what the transformation does
    fn description(&self) -> &str;
}
```

Each transformation will implement this trait:

```rust
pub struct SnakeCaseTransform;

impl Transformation for SnakeCaseTransform {
    fn transform(&self, input: &str) -> String {
        // Implementation for snake_case conversion
    }
    
    fn name(&self) -> &str {
        "snake"
    }
    
    fn description(&self) -> &str {
        "Convert to snake_case format"
    }
}
```

### 2. Transform Pipeline System

The pipeline system will chain transformations:

```rust
pub struct Pipeline {
    /// Sequence of transformations to apply
    transformations: Vec<Box<dyn Transformation>>,
    name: Option<String>,
    description: Option<String>,
}

impl Pipeline {
    /// Apply the pipeline to an input string
    pub fn apply(&self, input: &str) -> String {
        let mut result = input.to_string();
        for transformation in &self.transformations {
            result = transformation.transform(&result);
        }
        result
    }
}
```

#### Pipeline Parser

A parser will convert string pipeline definitions into Pipeline objects:

```rust
pub struct PipelineParser {
    registry: TransformationRegistry,
}

impl PipelineParser {
    /// Parse a pipeline string (e.g., "clean,snake,replace:old:new")
    pub fn parse(&self, pipeline_str: &str) -> Result<Pipeline, PipelineError> {
        // Implementation details
    }
}
```

### 3. Transformation Registry

A registry will manage available transformations:

```rust
pub struct TransformationRegistry {
    transformations: HashMap<String, Box<dyn Fn(Option<&str>) -> Result<Box<dyn Transformation>, TransformError>>>,
}

impl TransformationRegistry {
    /// Register a transformation factory
    pub fn register<F>(&mut self, name: &str, factory: F) 
    where 
        F: Fn(Option<&str>) -> Result<Box<dyn Transformation>, TransformError> + 'static
    {
        self.transformations.insert(name.to_string(), Box::new(factory));
    }
    
    /// Get a transformation by name and parameters
    pub fn get_transformation(&self, name: &str, params: Option<&str>) -> Result<Box<dyn Transformation>, TransformError> {
        // Implementation details
    }
}
```

### 4. Enhanced Character Replacement

New transformation types will be added:

```rust
/// Simple string replacement
pub struct ReplaceTransform {
    old_string: String,
    new_string: String,
}

/// Regular expression based replacement
pub struct RegexReplaceTransform {
    pattern: Regex,
    replacement: String,
}

/// Character set replacement
pub struct CharSetReplaceTransform {
    char_set: CharSet,
    replacement: String,
}

/// Character set definitions
pub enum CharSet {
    Digits,
    Lowercase,
    Uppercase,
    Vowels,
    Consonants,
    Whitespace,
    Punctuation,
    Custom(HashSet<char>),
}
```

### 5. Custom Transformations

Custom transformations will be serializable and storable:

```rust
/// User-defined transformation
pub struct CustomTransform {
    name: String,
    description: String,
    pipeline: Pipeline,
}

/// Custom transformation storage
pub struct TransformStore {
    store_path: PathBuf,
    transforms: HashMap<String, CustomTransform>,
}

impl TransformStore {
    /// Save a custom transformation
    pub fn save(&mut self, transform: CustomTransform) -> Result<(), StoreError> {
        // Implementation details
    }
    
    /// Load a custom transformation
    pub fn load(&self, name: &str) -> Result<CustomTransform, StoreError> {
        // Implementation details
    }
}
```

## CLI Integration

The CLI will be enhanced to support the new features:

```rust
struct Args {
    // Existing fields
    
    /// Pipeline of transformations to apply
    #[arg(short = 'p', long = "pipe", value_name = "PIPELINE")]
    pipeline: Option<String>,
    
    /// Simple string replacement (format: "old:new")
    #[arg(short = 'R', long = "rep", value_name = "REPLACEMENT")]
    replace: Option<String>,
    
    /// Character set replacement (format: "set:replacement")
    #[arg(short = 'C', long = "chars", value_name = "CHAR_SET")]
    replace_chars: Option<String>,
    
    /// Regular expression replacement (format: "pattern:replacement")
    #[arg(short = 'X', long = "regex", value_name = "REGEX")]
    replace_regex: Option<String>,
    
    /// Save a custom transformation
    #[arg(long = "save", value_name = "DEFINITION")]
    save_transform: Option<String>,
    
    /// Apply a custom transformation
    #[arg(short = 't', long = "tx", value_name = "NAME")]
    transform: Option<String>,
}
```

## File Format Specifications

### Custom Transformation Format

Custom transformations will be stored in JSON format:

```json
{
  "name": "my-transform",
  "description": "My custom transformation",
  "pipeline": "clean,snake,replace:space:_,upper"
}
```

## Configuration Storage

User configuration will be stored in standard locations:

- Linux: `~/.config/smv/`
- macOS: `~/Library/Application Support/smv/`
- Windows: `%APPDATA%\smv\`

Directory structure:
```
smv/
├── config.toml       # Main configuration
├── backups/          # File operation backups
├── transforms/       # Custom transformations
│   ├── transform1.json
│   └── transform2.json
└── templates/        # Transformation templates
```

## Implementation Strategy

### Phase 1: Transformation Trait Implementation

1. Create the `Transformation` trait in `src/transformers/mod.rs`
   ```rust
   pub trait Transformation {
       fn transform(&self, input: &str) -> String;
       fn name(&self) -> &str;
       fn description(&self) -> &str;
   }
   ```

2. Move existing transformation functions to `src/transformers/basic.rs`
   - Refactor existing code to maintain backward compatibility
   - Create structs for each transformation type

3. Implement the trait for each transformation type
   ```rust
   pub struct SnakeCaseTransform;
   
   impl Transformation for SnakeCaseTransform {
       fn transform(&self, input: &str) -> String {
           snake_case(input)
       }
       
       fn name(&self) -> &str {
           "snake"
       }
       
       fn description(&self) -> &str {
           "Convert to snake_case format"
       }
   }
   ```

4. Add unit tests for each implementation
   - Test individual transformations
   - Verify trait behavior

### Phase 2: Pipeline System Implementation

1. Create pipeline structures in `src/transformers/pipeline.rs`
   ```rust
   pub struct Pipeline {
       transformations: Vec<Box<dyn Transformation>>,
       name: Option<String>,
       description: Option<String>,
   }
   
   impl Pipeline {
       pub fn add_transformation(&mut self, transformation: Box<dyn Transformation>) -> &mut Self {
           self.transformations.push(transformation);
           self
       }
       
       pub fn apply(&self, input: &str) -> String {
           let mut result = input.to_string();
           for transformation in &self.transformations {
               result = transformation.transform(&result);
           }
           result
       }
   }
   ```

2. Implement pipeline parsing
   - Create parser for comma-separated pipeline strings
   - Handle parameters with colon syntax
   - Add proper error handling for malformed pipelines

3. Add pipeline visualization for preview mode
   - Show step-by-step transformation results 
   - Format output for readability

4. Write comprehensive tests for pipeline functionality
   - Test complex pipeline combinations
   - Verify error handling for edge cases

### Phase 3: Registry Implementation

1. Create the transformation registry in `src/transformers/registry.rs`
   ```rust
   pub struct TransformationRegistry {
       transformations: HashMap<String, Box<dyn Fn(Option<&str>) -> Result<Box<dyn Transformation>, TransformError>>>,
   }
   ```

2. Register standard transformations
   - Add factory functions for each transformation
   - Include parameter handling for configurable transformations

3. Add extension mechanisms for custom transformations
   - Create serialization format for storing transformations
   - Implement loading/saving functionality

4. Test registration and retrieval functionality
   - Verify transformations can be looked up by name
   - Test parameter passing

### Phase 4: CLI Integration

1. Update CLI argument parsing in `src/cli/mod.rs`
   - Add new command options
   - Support shorter command forms

2. Add command handlers for new features
   - Implement pipeline execution logic
   - Add custom transformation support

3. Implement output formatting for pipeline results
   - Format preview output
   - Show clear error messages

4. Add interactive mode support for pipelines
   - Update REPL commands
   - Add pipeline visualization in interactive mode

## Acceptance Criteria

The pipeline feature will be considered complete when:

1. Users can specify a comma-separated list of transformations
2. Pipeline execution applies transformations in the specified order
3. Pipeline preview shows intermediate results for each step
4. Error handling provides clear messages for invalid pipelines
5. Pipeline syntax is documented in help text and man page
6. Unit tests provide good coverage of pipeline functionality
7. Performance is acceptable for typical usage patterns

## Error Handling Strategy

A comprehensive error handling system will be implemented:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Unknown transformation: {0}")]
    UnknownTransformation(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Syntax error in pipeline: {0}")]
    PipelineSyntaxError(String),
    
    #[error("Transformation execution error: {0}")]
    ExecutionError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Store error: {0}")]
    StoreError(String),
}
```

## Testing Strategy

1. **Unit Tests**: Test individual transformations and components
2. **Integration Tests**: Test pipeline execution end-to-end
3. **Property Tests**: Verify transformation invariants
4. **CLI Tests**: Test command-line interface with actual invocations

## Performance Considerations

- Lazy loading of transformations to minimize startup time
- Efficient parsing of pipeline strings
- Reuse of regex patterns where possible
- Memory-efficient handling of large file sets

## Security Considerations

- Validate all user input before processing
- Handle file paths safely to prevent path traversal
- Sanitize transformation parameters
- Ensure configuration files have appropriate permissions