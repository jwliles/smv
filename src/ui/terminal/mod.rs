mod app;
mod tui;
pub mod views;
pub mod widgets;

pub use app::App;
pub use tui::Tui;

use crate::ui::UiAction;

/// Events that can occur in the terminal UI
pub enum Event {
    /// Key press event
    Key(crossterm::event::KeyEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// Application tick for animations or background tasks
    Tick,
}

/// Terminal UI application state
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AppMode {
    /// Normal mode - for navigation and basic commands
    Normal,
    /// Visual mode - for selecting multiple files
    Visual,
    /// Command mode - for entering commands
    Command,
    /// Insert mode - for editing text values
    Insert,
    /// Help mode - showing available actions and shortcuts
    Help,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Key handling result for app components
pub enum KeyResult {
    /// Event was handled, with a potential action
    Handled(Option<UiAction>),
    /// Event wasn't handled and should be passed to the next handler
    NotHandled,
}
