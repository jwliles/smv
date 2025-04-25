// Example implementation of the Transformation trait system

use std::collections::HashMap;
use std::fmt;

/// Error type for transformation operations
#[derive(Debug)]
pub enum TransformError {
    UnknownTransformation(String),
    InvalidParameter(String),
    ExecutionError(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::UnknownTransformation(name) => write!(f, "Unknown transformation: {}", name),
            TransformError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            TransformError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for TransformError {}

/// Trait for all transformations
pub trait Transformation {
    /// Apply the transformation to an input string
    fn transform(&self, input: &str) -> String;
    
    /// Get the name of the transformation
    fn name(&self) -> &str;
    
    /// Get a description of what the transformation does
    fn description(&self) -> &str;
}

/// Basic clean transformation that removes special characters and normalizes spaces
pub struct CleanTransform;

impl Transformation for CleanTransform {
    fn transform(&self, input: &str) -> String {
        // Simple implementation for example purposes
        let trimmed = input.trim();
        let no_special = trimmed.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '.' || *c == '-')
            .collect::<String>();
        
        // Normalize multiple spaces to single space
        let mut result = String::with_capacity(no_special.len());
        let mut last_was_space = false;
        
        for c in no_special.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }
        
        result
    }
    
    fn name(&self) -> &str {
        "clean"
    }
    
    fn description(&self) -> &str {
        "Remove special characters and normalize spaces"
    }
}

/// Snake case transformation
pub struct SnakeCaseTransform;

impl Transformation for SnakeCaseTransform {
    fn transform(&self, input: &str) -> String {
        // Simple implementation for example purposes
        let mut result = String::with_capacity(input.len());
        let mut last_was_uppercase = false;
        let mut last_was_lowercase = false;
        
        for (i, c) in input.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && last_was_lowercase {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                last_was_uppercase = true;
                last_was_lowercase = false;
            } else if c.is_lowercase() {
                result.push(c);
                last_was_uppercase = false;
                last_was_lowercase = true;
            } else if c.is_whitespace() || c == '-' || c == '.' {
                result.push('_');
                last_was_uppercase = false;
                last_was_lowercase = false;
            } else if c.is_alphanumeric() {
                result.push(c);
                last_was_uppercase = false;
                last_was_lowercase = false;
            }
        }
        
        result
    }
    
    fn name(&self) -> &str {
        "snake"
    }
    
    fn description(&self) -> &str {
        "Convert to snake_case format"
    }
}

/// Parameterized replacement transformation
pub struct ReplaceTransform {
    old_string: String,
    new_string: String,
}

impl ReplaceTransform {
    pub fn new(old_string: &str, new_string: &str) -> Self {
        Self {
            old_string: old_string.to_string(),
            new_string: new_string.to_string(),
        }
    }
    
    /// Parse a parameter string in the format "old:new"
    pub fn from_parameter(param: &str) -> Result<Self, TransformError> {
        let parts: Vec<&str> = param.split(':').collect();
        
        if parts.len() != 2 {
            return Err(TransformError::InvalidParameter(
                "Replace parameter must be in the format 'old:new'".to_string()
            ));
        }
        
        Ok(Self::new(parts[0], parts[1]))
    }
}

impl Transformation for ReplaceTransform {
    fn transform(&self, input: &str) -> String {
        input.replace(&self.old_string, &self.new_string)
    }
    
    fn name(&self) -> &str {
        "replace"
    }
    
    fn description(&self) -> &str {
        "Replace a substring with another string"
    }
}

/// A pipeline of transformations
pub struct Pipeline {
    transformations: Vec<Box<dyn Transformation>>,
    name: Option<String>,
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
    
    /// Set a name for the pipeline
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set a description for the pipeline
    pub fn set_description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_string());
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
    
    /// Get the transformation steps
    pub fn get_steps(&self) -> Vec<&str> {
        self.transformations.iter().map(|t| t.name()).collect()
    }
    
    /// Get a preview of the pipeline execution
    pub fn preview(&self, input: &str) -> String {
        let mut result = format!("Original: {}\n", input);
        let mut current = input.to_string();
        
        for (i, transformation) in self.transformations.iter().enumerate() {
            current = transformation.transform(&current);
            result.push_str(&format!("Step {}: {} → {}\n", 
                i+1, 
                transformation.name(), 
                current
            ));
        }
        
        result
    }
}

/// Registry of available transformations
pub struct TransformationRegistry {
    factories: HashMap<String, Box<dyn Fn(&str) -> Result<Box<dyn Transformation>, TransformError>>>,
}

impl TransformationRegistry {
    /// Create a new registry with standard transformations
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        
        // Register standard transformations
        registry.register("clean", |_| Ok(Box::new(CleanTransform)));
        registry.register("snake", |_| Ok(Box::new(SnakeCaseTransform)));
        
        // Register parameterized transformations
        registry.register("replace", |param| {
            if param.is_empty() {
                return Err(TransformError::InvalidParameter(
                    "Replace requires parameters".to_string()
                ));
            }
            let transform = ReplaceTransform::from_parameter(param)?;
            Ok(Box::new(transform))
        });
        
        registry
    }
    
    /// Register a new transformation factory
    pub fn register<F>(&mut self, name: &str, factory: F) 
    where 
        F: Fn(&str) -> Result<Box<dyn Transformation>, TransformError> + 'static
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }
    
    /// Get a transformation by name and parameters
    pub fn get_transformation(&self, name: &str, params: &str) -> Result<Box<dyn Transformation>, TransformError> {
        match self.factories.get(name) {
            Some(factory) => factory(params),
            None => Err(TransformError::UnknownTransformation(name.to_string())),
        }
    }
}

/// Parser for pipeline strings
pub struct PipelineParser {
    registry: TransformationRegistry,
}

impl PipelineParser {
    /// Create a new parser with the standard registry
    pub fn new() -> Self {
        Self {
            registry: TransformationRegistry::new(),
        }
    }
    
    /// Parse a pipeline string into a Pipeline object
    /// Format: "transform1,transform2:param,transform3"
    pub fn parse(&self, pipeline_str: &str) -> Result<Pipeline, TransformError> {
        let mut pipeline = Pipeline::new();
        
        for step in pipeline_str.split(',') {
            let parts: Vec<&str> = step.split(':').collect();
            let transform_name = parts[0].trim();
            
            // Get parameters if any
            let params = if parts.len() > 1 {
                parts[1..].join(":")
            } else {
                String::new()
            };
            
            // Get the transformation
            let transformation = self.registry.get_transformation(transform_name, &params)?;
            pipeline.add_transformation(transformation);
        }
        
        Ok(pipeline)
    }
}

// Example usage
fn main() {
    // Create a pipeline parser
    let parser = PipelineParser::new();
    
    // Parse a pipeline string
    let pipeline_str = "clean,snake,replace:space:_";
    let pipeline = parser.parse(pipeline_str).unwrap();
    
    // Apply the pipeline to a filename
    let input = "My File (1).txt";
    let output = pipeline.apply(input);
    
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!("\nPipeline steps:");
    println!("{}", pipeline.preview(input));
}