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
    Sentence,
    Start,
    Studly,
    SplitSnake,
    SplitKebab,
    SplitTitle,
    SplitCamel,
    SplitPascal,
    SplitLower,
    SplitUpper,
    SplitSentence,
    SplitStart,
    SplitStudly,
    Replace(String, String),
    ReplaceRegex(String, String),
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
            TransformAction::Sentence => "Sentence case",
            TransformAction::Start => "Start Case",
            TransformAction::Studly => "StUdLyCaPs",
            TransformAction::SplitSnake => "split→snake_case",
            TransformAction::SplitKebab => "split→kebab-case",
            TransformAction::SplitTitle => "split→Title Case",
            TransformAction::SplitCamel => "split→camelCase",
            TransformAction::SplitPascal => "split→PascalCase",
            TransformAction::SplitLower => "split→lowercase",
            TransformAction::SplitUpper => "split→UPPERCASE",
            TransformAction::SplitSentence => "split→Sentence case",
            TransformAction::SplitStart => "split→Start Case",
            TransformAction::SplitStudly => "split→StUdLyCaPs",
            TransformAction::Replace(_, _) => "replace",
            TransformAction::ReplaceRegex(_, _) => "regex replace",
        }
    }
}
