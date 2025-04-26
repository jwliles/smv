use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

/// Action to perform when a key is pressed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move cursor up
    Up,
    /// Move cursor down
    Down,
    /// Move cursor left
    Left,
    /// Move cursor right
    Right,
    /// Enter a directory or select a file
    Enter,
    /// Go to parent directory
    Parent,
    /// Start search
    Search,
    /// Start fuzzy search
    FuzzySearch,
    /// Enter visual selection mode
    Visual,
    /// Enter normal mode
    Normal,
    /// Enter command mode
    Command,
    /// Execute a command
    Execute,
    /// Execute queue
    ExecuteQueue,
    /// Quit the application
    Quit,
    /// Show help
    Help,
    /// Copy/yank
    Yank,
    /// Delete
    Delete,
    /// Move to first item
    First,
    /// Move to last item
    Last,
    /// Refresh view
    Refresh,
}

/// Key mapping configuration
pub struct KeyMap {
    /// Normal mode key mappings
    normal_mode: HashMap<KeyEvent, Action>,
    /// Visual mode key mappings
    visual_mode: HashMap<KeyEvent, Action>,
    /// Command mode key mappings
    command_mode: HashMap<KeyEvent, Action>,
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut normal_mode = HashMap::new();
        let mut visual_mode = HashMap::new();
        let mut command_mode = HashMap::new();
        
        // Normal mode mappings
        normal_mode.insert(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE), Action::Down);
        normal_mode.insert(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), Action::Down);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE), Action::Up);
        normal_mode.insert(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), Action::Up);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), Action::Left);
        normal_mode.insert(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), Action::Left);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE), Action::Right);
        normal_mode.insert(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), Action::Right);
        normal_mode.insert(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), Action::Enter);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE), Action::Search);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE), Action::FuzzySearch);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE), Action::Visual);
        normal_mode.insert(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE), Action::Command);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT), Action::ExecuteQueue);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL), Action::Quit);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE), Action::Help);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE), Action::Yank);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE), Action::Delete);
        normal_mode.insert(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE), Action::First);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE), Action::First);
        normal_mode.insert(KeyEvent::new(KeyCode::End, KeyModifiers::NONE), Action::Last);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE), Action::Last);
        normal_mode.insert(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE), Action::Refresh);
        
        // Visual mode mappings
        visual_mode.insert(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE), Action::Down);
        visual_mode.insert(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), Action::Down);
        visual_mode.insert(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE), Action::Up);
        visual_mode.insert(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), Action::Up);
        visual_mode.insert(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), Action::Normal);
        visual_mode.insert(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE), Action::Yank);
        visual_mode.insert(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE), Action::Delete);
        
        // Command mode mappings
        command_mode.insert(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), Action::Execute);
        command_mode.insert(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), Action::Normal);
        
        Self {
            normal_mode,
            visual_mode,
            command_mode,
        }
    }
}

impl KeyMap {
    /// Get the action for a key event in normal mode
    pub fn get_normal_action(&self, key: KeyEvent) -> Option<&Action> {
        self.normal_mode.get(&key)
    }
    
    /// Get the action for a key event in visual mode
    pub fn get_visual_action(&self, key: KeyEvent) -> Option<&Action> {
        self.visual_mode.get(&key)
    }
    
    /// Get the action for a key event in command mode
    pub fn get_command_action(&self, key: KeyEvent) -> Option<&Action> {
        self.command_mode.get(&key)
    }
}