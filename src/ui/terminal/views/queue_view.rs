use ratatui::widgets::ListState;

/// View for the operation queue
pub struct QueueView {
    /// List selection state (kept for potential future stateful rendering)
    pub state: ListState,
}

impl Default for QueueView {
    fn default() -> Self {
        Self::new()
    }
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
}
