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
}
