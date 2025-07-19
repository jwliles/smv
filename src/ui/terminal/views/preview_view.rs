use std::path::PathBuf;

use crate::transformers::{self, TransformType};
use crate::ui::terminal::app::FileOperation;

/// Preview of file operations
pub struct PreviewView {
    /// Currently previewed file operations
    operations: Vec<PreviewOperation>,
}

/// A file operation with preview information
pub struct PreviewOperation {
    /// Source path
    pub source: PathBuf,
    /// Destination path
    pub destination: PathBuf,
    /// Source file name for display
    pub source_name: String,
    /// Destination file name for display
    pub destination_name: String,
    /// Would this operation cause conflicts?
    pub has_conflict: bool,
}

impl Default for PreviewView {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewView {
    /// Create a new preview view
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Set operations to preview
    pub fn set_operations(&mut self, operations: &[FileOperation]) {
        self.operations.clear();

        for op in operations {
            let source_name = op
                .source
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let destination_name = op
                .destination
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Check for conflicts (if destination exists and isn't the source)
            let has_conflict = op.destination.exists() && op.source != op.destination;

            self.operations.push(PreviewOperation {
                source: op.source.clone(),
                destination: op.destination.clone(),
                source_name,
                destination_name,
                has_conflict,
            });
        }
    }

    /// Generate preview for a file transformation
    pub fn preview_transform(&self, filename: &str, transform_type: TransformType) -> String {
        match transform_type {
            TransformType::Snake => {
                transformers::transform(filename, &transformers::TransformType::Snake)
            }
            TransformType::Kebab => {
                transformers::transform(filename, &transformers::TransformType::Kebab)
            }
            TransformType::Title => {
                transformers::transform(filename, &transformers::TransformType::Title)
            }
            TransformType::Camel => {
                transformers::transform(filename, &transformers::TransformType::Camel)
            }
            TransformType::Pascal => {
                transformers::transform(filename, &transformers::TransformType::Pascal)
            }
            TransformType::Lower => {
                transformers::transform(filename, &transformers::TransformType::Lower)
            }
            TransformType::Upper => {
                transformers::transform(filename, &transformers::TransformType::Upper)
            }
            TransformType::Clean => {
                transformers::transform(filename, &transformers::TransformType::Clean)
            }
            TransformType::Replace(find, replace) => transformers::transform(
                filename,
                &transformers::TransformType::Replace(find.clone(), replace.clone()),
            ),
            TransformType::ReplaceRegex(pattern, replacement) => transformers::transform(
                filename,
                &transformers::TransformType::ReplaceRegex(pattern.clone(), replacement.clone()),
            ),
            TransformType::RemovePrefix(prefix) => transformers::transform(
                filename,
                &transformers::TransformType::RemovePrefix(prefix.clone()),
            ),
        }
    }
}
