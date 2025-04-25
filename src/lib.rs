//! SMV (Smart Move) library
//!
//! This is the core library for SMV, providing functionality for file operations,
//! transformations, and history management.

pub mod transformers;
pub mod history;
pub mod repl;
pub mod cli;
pub mod core;

// Re-export common types for external use
pub use transformers::{TransformType, transform};
pub use history::{Operation, HistoryManager};
pub use repl::InteractiveSession;

/// Version of the SMV library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");