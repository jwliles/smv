pub mod input;
pub mod terminal;
#[cfg(test)]
mod tests;
mod theme;

pub use theme::Theme;

use std::error::Error;
use std::path::PathBuf;

/// The main UI trait that all UI implementations must implement
pub trait UserInterface {
    /// Run the UI until it exits
    fn run(&mut self) -> Result<(), Box<dyn Error>>;

    /// Open a specific directory in the UI
    fn open_directory(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>>;
}

/// Result of a UI operation that may require further action
pub enum UiAction {
    /// Continue normal operation
    Continue,
    /// Exit the application
    Exit,
    /// Perform file operations from the queue
    ExecuteQueue,
    /// Show help information
    ShowHelp,
    /// Add file to operation queue
    AddToQueue,
    /// Transform the selected file
    Transform(TransformAction),
    /// Group files by basename
    GroupFiles,
    /// Flatten directory structure
    FlattenDirectory,
}

/// Transform action for UI operations
#[derive(Clone, Debug)]
pub enum TransformAction {
    Snake,
    Kebab,
    Clean,
    Title,
    Camel,
    Pascal,
    Lower,
    Upper,
}

impl TransformAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransformAction::Snake => "snake_case",
            TransformAction::Kebab => "kebab-case",
            TransformAction::Clean => "clean",
            TransformAction::Title => "Title Case",
            TransformAction::Camel => "camelCase",
            TransformAction::Pascal => "PascalCase",
            TransformAction::Lower => "lowercase",
            TransformAction::Upper => "UPPERCASE",
        }
    }
}
