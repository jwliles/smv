//! Filename transformation functionality
//!
//! This module provides various transformations that can be applied to filenames.

mod basic;
// These will be implemented later:
// mod pipeline;
// mod custom;
// mod character;

// Re-export types from submodules
pub use basic::{TransformType, transform};

/// Common trait for all transformations
pub trait Transformation {
    /// Apply the transformation to an input string
    fn transform(&self, input: &str) -> String;
    
    /// Get the name of the transformation
    fn name(&self) -> &str;
    
    /// Get a description of what the transformation does
    fn description(&self) -> &str;
}