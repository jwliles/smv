use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::ui::terminal::app::OperationQueue;
use crate::ui::terminal::{AppMode, KeyResult};
use crate::ui::UiAction;

/// View for the operation queue
pub struct QueueView {
    /// List selection state
    state: ListState,
}

impl QueueView {
    /// Create a new queue view
    pub fn new() -> Self {
        let mut queue_view = Self {
            state: ListState::default(),
        };
        queue_view.state.select(Some(0));
        queue_view
    }

    /// Handle key events for the queue view
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        mode: &AppMode,
        queue: &mut OperationQueue,
    ) -> KeyResult {
        if *mode != AppMode::Normal {
            return KeyResult::NotHandled;
        }

        // Queue manipulation keys
        match (key.code, key.modifiers) {
            // Navigation within queue
            (KeyCode::Char('J'), KeyModifiers::SHIFT) => {
                queue.select_next();
                KeyResult::Handled(None)
            }
            (KeyCode::Char('K'), KeyModifiers::SHIFT) => {
                queue.select_prev();
                KeyResult::Handled(None)
            }

            // Queue operations
            (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
                // Delete item from queue
                queue.remove_selected();
                KeyResult::Handled(None)
            }
            (KeyCode::Char('X'), KeyModifiers::SHIFT) => {
                // Execute queue
                KeyResult::Handled(Some(UiAction::ExecuteQueue))
            }
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                // Clear queue
                queue.clear();
                KeyResult::Handled(None)
            }

            _ => KeyResult::NotHandled,
        }
    }
}
