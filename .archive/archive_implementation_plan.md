# Implementation Plan: Transform Pipelines

## Overview

Transform pipelines allow users to chain multiple transformations in sequence to achieve complex file renaming patterns with a single command. This document outlines the implementation details for this feature.

## Architecture Changes

### 1. Create Transformation Trait

Replace the current `TransformType` enum with a trait-based approach:

```rust
// src/transformers.rs
pub trait Transformation {
    /// Apply the transformation to an input string
    fn transform(&self, input: &str) -> String;
    
    /// Get the name of the transformation
    fn name(&self) -> &str;
    
    /// Get a description of what the transformation does
    fn description(&self) -> &str;
}
```

### 2. Implement Transformations for Existing Types

Convert the existing transformation functions to structs that implement the `Transformation` trait:

```rust
pub struct SnakeCaseTransform;

impl Transformation for SnakeCaseTransform {
    fn transform(&self, input: &str) -> String {
        // Existing snake_case implementation
        snake_case(input)
    }
    
    fn name(&self) -> &str {
        "snake"
    }
    
    fn description(&self) -> &str {
        "Convert to snake_case format"
    }
}

// Similar implementations for other transformations
```

### 3. Pipeline Structure

Create a pipeline structure to hold a sequence of transformations:

```rust
// src/pipeline.rs
pub struct Pipeline {
    /// Sequence of transformations to apply
    transformations: Vec<Box<dyn Transformation>>,
    
    /// Optional name for the pipeline
    name: Option<String>,
    
    /// Description of what the pipeline does
    description: Option<String>,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
            name: None,
            description: None,
        }
    }
    
    /// Add a transformation to the pipeline
    pub fn add_transformation(&mut self, transformation: Box<dyn Transformation>) -> &mut Self {
        self.transformations.push(transformation);
        self
    }
    
    /// Apply the pipeline to an input string
    pub fn apply(&self, input: &str) -> String {
        let mut result = input.to_string();
        for transformation in &self.transformations {
            result = transformation.transform(&result);
        }
        result
    }
    
    /// Get a list of transformation steps for display
    pub fn get_steps(&self) -> Vec<&str> {
        self.transformations.iter().map(|t| t.name()).collect()
    }
}
```

### 4. Pipeline Parser

Create a parser for pipeline strings:

```rust
// src/pipeline.rs
pub struct PipelineParser {
    // Registry of available transformations
    registry: TransformationRegistry,
}

impl PipelineParser {
    /// Parse a pipeline string into a Pipeline object
    /// Format: "transform1,transform2:param,transform3"
    pub fn parse(&self, pipeline_str: &str) -> Result<Pipeline, PipelineError> {
        let mut pipeline = Pipeline::new();
        
        for step in pipeline_str.split(',') {
            let parts: Vec<&str> = step.split(':').collect();
            let transform_name = parts[0].trim();
            
            // Look up the transformation in the registry
            let transformation = self.registry.get_transformation(transform_name)?;
            
            // If there are parameters, configure the transformation
            if parts.len() > 1 {
                // Configure transformation with parameters
                // (implementation depends on transformation types)
            }
            
            pipeline.add_transformation(transformation);
        }
        
        Ok(pipeline)
    }
}
```

### 5. Transformation Registry

Create a registry to manage available transformations:

```rust
// src/registry.rs
pub struct TransformationRegistry {
    transformations: HashMap<String, Box<dyn Fn() -> Box<dyn Transformation>>>,
}

impl TransformationRegistry {
    /// Create a new registry with standard transformations
    pub fn new() -> Self {
        let mut registry = Self {
            transformations: HashMap::new(),
        };
        
        // Register standard transformations
        registry.register("clean", || Box::new(CleanTransform));
        registry.register("snake", || Box::new(SnakeCaseTransform));
        registry.register("kebab", || Box::new(KebabCaseTransform));
        // ...
        
        registry
    }
    
    /// Register a new transformation factory
    pub fn register<F>(&mut self, name: &str, factory: F) 
    where 
        F: Fn() -> Box<dyn Transformation> + 'static
    {
        self.transformations.insert(name.to_string(), Box::new(factory));
    }
    
    /// Get a transformation by name
    pub fn get_transformation(&self, name: &str) -> Result<Box<dyn Transformation>, TransformError> {
        match self.transformations.get(name) {
            Some(factory) => Ok(factory()),
            None => Err(TransformError::UnknownTransformation(name.to_string())),
        }
    }
}
```

## CLI Integration

Update the CLI arguments to support pipelines:

```rust
// src/main.rs
struct Args {
    // Existing fields...
    
    /// Pipeline of transformations to apply
    #[arg(long, value_name = "PIPELINE")]
    pipeline: Option<String>,
}
```

Modify the main application flow to handle pipelines:

```rust
fn main() -> Result<(), Box<dyn Error>> {
    // Parse arguments...
    
    if let Some(pipeline_str) = &args.pipeline {
        // Initialize registry and parser
        let registry = TransformationRegistry::new();
        let parser = PipelineParser { registry };
        
        // Parse the pipeline
        let pipeline = parser.parse(pipeline_str)?;
        
        // Apply pipeline to files
        apply_pipeline_to_files(&pipeline, &args)?;
    } else if is_transformation_requested(&args) {
        // Existing transformation logic...
    } else if !args.source.is_empty() {
        // Existing mv logic...
    }
    
    Ok(())
}
```

## Interactive Mode Integration

Update the REPL to support pipelines:

```rust
// src/repl.rs
impl InteractiveSession {
    // Add a new command for pipelines
    fn cmd_pipeline(&mut self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if args.len() < 2 {
            return Err("Usage: pipeline <pipeline_string> <file_pattern>".into());
        }
        
        let pipeline_str = args[0];
        let file_patterns = &args[1..];
        
        // Initialize registry and parser
        let registry = TransformationRegistry::new();
        let parser = PipelineParser { registry };
        
        // Parse the pipeline
        let pipeline = parser.parse(pipeline_str)?;
        
        // Show steps
        println!("Pipeline steps: {}", pipeline.get_steps().join(" → "));
        
        // Apply pipeline to files
        self.apply_pipeline_to_files(&pipeline, file_patterns)
    }
}
```

## Pipeline Visualization

Implement a visualization for pipeline previews:

```rust
// src/pipeline.rs
pub struct PipelineVisualizer;

impl PipelineVisualizer {
    /// Generate a preview of pipeline steps
    pub fn preview(&self, pipeline: &Pipeline, input: &str) -> String {
        let mut result = format!("Original: {}\n", input);
        let mut current = input.to_string();
        
        for (i, transformation) in pipeline.transformations.iter().enumerate() {
            current = transformation.transform(&current);
            result.push_str(&format!("Step {}: {} → {}\n", 
                i+1, 
                transformation.name(), 
                current
            ));
        }
        
        result.push_str(&format!("Final: {}", current));
        result
    }
}
```

## Testing Plan

1. Unit tests for each transformation implementation
2. Tests for the pipeline parser with various input formats
3. Tests for pipeline execution with different transformation combinations
4. Integration tests for CLI pipeline usage
5. Test cases for error handling and invalid inputs

## Implementation Steps

1. Refactor the transformation system to use the trait-based approach
2. Implement the basic pipeline structure and execution logic
3. Create the pipeline parser and registry
4. Update the CLI to support pipeline strings
5. Add pipeline support to the interactive mode
6. Implement visualization for pipeline steps
7. Add comprehensive tests for all components
8. Update documentation with pipeline examples

## Example Usage

```bash
# Apply a pipeline of transformations
smv --pipeline "clean,snake,replace:space:_" *.txt

# In interactive mode
pipeline "clean,kebab" *.jpg

# Preview a pipeline
smv --pipeline "clean,pascal" --preview *.md
```

## Acceptance Criteria

1. Users can specify a comma-separated list of transformations
2. Pipeline execution applies transformations in the specified order
3. Pipeline preview shows intermediate results for each step
4. Error handling provides clear messages for invalid pipelines
5. Pipeline syntax is documented in help and README
6. Unit tests provide good coverage of pipeline functionality
7. Performance is acceptable for typical usage patterns

## Estimated Effort

- Core implementation: 3-4 days
- Testing: 2 days
- Documentation: 1 day
- Total: 6-7 days