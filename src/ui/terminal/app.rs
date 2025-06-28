use std::error::Error;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

use crate::ui::terminal::views::{FileExplorer, FileItem, PreviewView, QueueView};
use crate::ui::terminal::{AppMode, Event, KeyResult, Tui};
use crate::ui::{Theme, UiAction, UserInterface, TransformAction};
use crate::transformers::transform;
use crate::{sort, unsort};

/// Queue for file operations to be performed
pub struct OperationQueue {
    operations: Vec<FileOperation>,
    selected_index: usize,
}

impl OperationQueue {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            selected_index: 0,
        }
    }

    pub fn add(&mut self, operation: FileOperation) {
        self.operations.push(operation);
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn operations(&self) -> &[FileOperation] {
        &self.operations
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn select_next(&mut self) {
        if !self.operations.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.operations.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.operations.is_empty() {
            self.selected_index = self
                .selected_index
                .checked_sub(1)
                .unwrap_or(self.operations.len() - 1);
        }
    }

    pub fn remove_selected(&mut self) {
        if !self.operations.is_empty() {
            self.operations.remove(self.selected_index);
            if self.selected_index >= self.operations.len() && !self.operations.is_empty() {
                self.selected_index = self.operations.len() - 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.operations.clear();
        self.selected_index = 0;
    }
}

/// Represents a file operation in the queue
#[derive(Clone, Debug)]
pub struct FileOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub operation_type: OperationType,
}

/// Type of file operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationType {
    Move,
    Transform(crate::transformers::TransformType),
}

/// The main terminal application
pub struct App {
    /// Terminal interface
    tui: Tui,
    /// Current mode
    mode: AppMode,
    /// Current working directory
    current_dir: PathBuf,
    /// File explorer view
    explorer: FileExplorer,
    /// Operation queue
    queue: OperationQueue,
    /// Queue view
    queue_view: QueueView,
    /// Preview view
    preview: PreviewView,
    /// Global theme
    theme: Theme,
    /// Whether the application should exit
    should_exit: bool,
    /// Status message
    status_message: String,
}

impl App {
    /// Create a new application
    pub fn new() -> anyhow::Result<Self> {
        // Initialize terminal UI
        let tui = Tui::new()?;

        // Set up panic hook
        Tui::init_panic_hook();

        // Get current directory
        let current_dir = std::env::current_dir()?;

        // Create file explorer
        let mut explorer = FileExplorer::new(current_dir.clone());
        
        // Debug: Log file count
        eprintln!("DEBUG: Loaded {} files in {}", explorer.files.len(), current_dir.display());
        
        // Ensure we have at least some content to display
        if explorer.files.is_empty() {
            eprintln!("DEBUG: No files found, adding placeholder");
            // This shouldn't happen since reload_files adds ".." but just in case
            use crate::ui::terminal::views::FileItem;
            explorer.files.push(FileItem {
                name: "No files found".to_string(),
                path: current_dir.clone(),
                is_dir: false,
                is_symlink: false,
                size: 0,
            });
        }

        Ok(Self {
            tui,
            mode: AppMode::Normal,
            current_dir: current_dir.clone(),
            explorer,
            queue: OperationQueue::new(),
            queue_view: QueueView::new(),
            preview: PreviewView::new(),
            theme: Theme::default(),
            should_exit: false,
            status_message: String::from("Press ? for help. j/k to navigate, Ctrl+Q to quit"),
        })
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Global key handlers (work in any mode)
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_exit = true;
                return Ok(());
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                // Show help
                self.status_message = String::from("Help mode - press ESC to exit");
                return Ok(());
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                // Always go back to normal mode on ESC
                self.mode = AppMode::Normal;
                self.status_message = String::from("Normal mode");
                return Ok(());
            }
            _ => {}
        }

        // Mode-specific key handlers
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key)?,
            AppMode::Visual => self.handle_visual_mode_key(key)?,
            AppMode::Command => self.handle_command_mode_key(key)?,
            AppMode::Insert => self.handle_insert_mode_key(key)?,
        }

        Ok(())
    }

    /// Handle keys in normal mode
    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // First try to handle keys in the explorer view
        match self.explorer.handle_key(key, &self.mode) {
            KeyResult::Handled(action) => {
                if let Some(action) = action {
                    self.handle_ui_action(action)?;
                }
                return Ok(());
            }
            KeyResult::NotHandled => {}
        }

        // Then try to handle keys in the queue view
        match self.queue_view.handle_key(key, &self.mode, &mut self.queue) {
            KeyResult::Handled(action) => {
                if let Some(action) = action {
                    self.handle_ui_action(action)?;
                }
                return Ok(());
            }
            KeyResult::NotHandled => {}
        }

        // Finally, handle application-level keys
        match (key.code, key.modifiers) {
            (KeyCode::Char('v'), KeyModifiers::NONE) => {
                self.mode = AppMode::Visual;
                self.status_message = String::from("Visual mode");
            }
            (KeyCode::Char(':'), KeyModifiers::NONE) => {
                self.mode = AppMode::Command;
                self.status_message = String::from(":");
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                // Execute queue
                self.handle_ui_action(UiAction::ExecuteQueue)?;
            }
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                // Clear queue
                self.queue.clear();
                self.status_message = String::from("Queue cleared");
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keys in visual mode
    fn handle_visual_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Handle visual mode selection
        match self.explorer.handle_key(key, &self.mode) {
            KeyResult::Handled(action) => {
                if let Some(action) = action {
                    self.handle_ui_action(action)?;
                }
                return Ok(());
            }
            KeyResult::NotHandled => {}
        }

        Ok(())
    }

    /// Handle keys in command mode
    fn handle_command_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Command input handling
        match key.code {
            KeyCode::Enter => {
                // Process command (to be implemented)
                self.mode = AppMode::Normal;
                self.status_message = String::from("Command executed");
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keys in insert mode
    fn handle_insert_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Text editing for rename operations
        match key.code {
            KeyCode::Enter => {
                // Finish text input
                self.mode = AppMode::Normal;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle UI action
    fn handle_ui_action(&mut self, action: UiAction) -> anyhow::Result<()> {
        match action {
            UiAction::Exit => {
                self.should_exit = true;
            }
            UiAction::ExecuteQueue => {
                self.execute_queue()?;
            }
            UiAction::ShowHelp => {
                self.status_message = String::from("Help view (not implemented)");
            }
            UiAction::AddToQueue => {
                if let Some(file) = self.explorer.selected() {
                    if !file.is_dir {
                        let operation = FileOperation {
                            source: file.path.clone(),
                            destination: file.path.clone(), // Will be updated based on operation
                            operation_type: OperationType::Move,
                        };
                        self.queue.add(operation);
                        self.status_message = format!("Added {} to queue", file.name);
                    }
                }
            }
            UiAction::Transform(transform_action) => {
                if let Some(file) = self.explorer.selected().cloned() {
                    if !file.is_dir {
                        self.add_transform_to_queue(&file, transform_action)?;
                    }
                }
            }
            UiAction::GroupFiles => {
                if let Some(dir) = self.explorer.selected().cloned() {
                    if dir.is_dir {
                        self.group_files_in_directory(&dir.path)?;
                    }
                }
            }
            UiAction::FlattenDirectory => {
                if let Some(dir) = self.explorer.selected().cloned() {
                    if dir.is_dir {
                        self.flatten_directory(&dir.path)?;
                    }
                }
            }
            UiAction::Continue => {}
        }

        Ok(())
    }

    /// Add a transformation operation to the queue
    fn add_transform_to_queue(&mut self, file: &FileItem, transform_action: TransformAction) -> anyhow::Result<()> {
        let transform_type = match transform_action {
            TransformAction::Snake => crate::transformers::TransformType::Snake,
            TransformAction::Kebab => crate::transformers::TransformType::Kebab,
            TransformAction::Clean => crate::transformers::TransformType::Clean,
            TransformAction::Title => crate::transformers::TransformType::Title,
            TransformAction::Camel => crate::transformers::TransformType::Camel,
            TransformAction::Pascal => crate::transformers::TransformType::Pascal,
            TransformAction::Lower => crate::transformers::TransformType::Lower,
            TransformAction::Upper => crate::transformers::TransformType::Upper,
        };

        // Get the filename and apply transformation
        let filename = file.path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy();
        let new_filename = transform(&filename, &transform_type);
        
        // Create new path with transformed filename
        let new_path = file.path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid parent directory"))?
            .join(&new_filename);

        let operation = FileOperation {
            source: file.path.clone(),
            destination: new_path,
            operation_type: OperationType::Transform(transform_type),
        };

        self.queue.add(operation);
        self.status_message = format!("Added {} transformation for {}", transform_action.as_str(), file.name);
        
        Ok(())
    }

    /// Execute all operations in the queue
    fn execute_queue(&mut self) -> anyhow::Result<()> {
        if self.queue.is_empty() {
            self.status_message = String::from("Queue is empty");
            return Ok(());
        }

        let operations = self.queue.operations().to_vec();
        let mut success_count = 0;
        let mut error_count = 0;

        for operation in operations {
            match std::fs::rename(&operation.source, &operation.destination) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(_e) => {
                    error_count += 1;
                }
            }
        }

        self.queue.clear();
        self.status_message = format!("Executed: {} success, {} errors", success_count, error_count);
        
        // Reload the file explorer to show changes
        let _ = self.explorer.reload_files();
        
        Ok(())
    }

    /// Group files by basename in the selected directory
    fn group_files_in_directory(&mut self, dir_path: &PathBuf) -> anyhow::Result<()> {
        match sort::group_by_basename(&dir_path.to_string_lossy(), false) {
            Ok(_) => {
                self.status_message = format!("Grouped files in {}", dir_path.display());
                // Reload the file explorer to show changes
                let _ = self.explorer.reload_files();
            }
            Err(e) => {
                self.status_message = format!("Error grouping files: {}", e);
            }
        }
        Ok(())
    }

    /// Flatten the selected directory structure
    fn flatten_directory(&mut self, dir_path: &PathBuf) -> anyhow::Result<()> {
        match unsort::flatten_directory(&dir_path.to_string_lossy(), false) {
            Ok(_) => {
                // Also remove empty directories
                let _ = unsort::remove_empty_dirs(&dir_path.to_string_lossy(), false);
                self.status_message = format!("Flattened directory {}", dir_path.display());
                // Reload the file explorer to show changes
                let _ = self.explorer.reload_files();
            }
            Err(e) => {
                self.status_message = format!("Error flattening directory: {}", e);
            }
        }
        Ok(())
    }

    /// Main render function
    fn render(&mut self) -> anyhow::Result<()> {
        // Prepare data outside the closure to avoid borrow checker issues
        let current_dir = self.current_dir.display().to_string();
        let status_message = self.status_message.clone();
        let mode = format!("{:?}", self.mode);
        let queue_len = self.queue.operations().len();
        let files_data: Vec<(String, bool)> = self.explorer.files.iter()
            .map(|file| (file.name.clone(), file.is_dir))
            .collect();

        self.tui.draw(|frame| {
            use ratatui::{
                layout::{Constraint, Direction, Layout},
                style::{Color, Modifier, Style},
                widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
            };

            let size = frame.size();

            // Create main layout: vertical split
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(0),     // Main content
                    Constraint::Length(3),  // Status bar
                ])
                .split(size);

            // Header
            let header = Paragraph::new(format!("SMV Terminal UI - {}", current_dir))
                .block(Block::default().borders(Borders::ALL).title("Smart Move"))
                .style(Style::default().fg(Color::Cyan));
            frame.render_widget(header, chunks[0]);

            // Main content area: horizontal split
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),  // File explorer
                    Constraint::Percentage(30),  // Queue
                ])
                .split(chunks[1]);

            // File explorer with real data
            let explorer_content: Vec<ListItem> = files_data.iter()
                .map(|(name, is_dir)| {
                    let icon = if *is_dir { "📁" } else { "📄" };
                    ListItem::new(format!("{} {}", icon, name))
                })
                .collect();
            
            let explorer = List::new(explorer_content)
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Blue));
            
            frame.render_widget(explorer, main_chunks[0]);

            // Queue view
            let queue_content = if queue_len > 0 {
                vec![ListItem::new(format!("Operations: {}", queue_len))]
            } else {
                vec![ListItem::new("No operations queued")]
            };
            
            let queue = List::new(queue_content)
                .block(Block::default().borders(Borders::ALL).title("Queue"))
                .style(Style::default().fg(Color::White));
            frame.render_widget(queue, main_chunks[1]);

            // Status bar
            let status_text = format!("Mode: {} | {} | Ctrl+Q: Quit, ?: Help", mode, status_message);
            let status = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });
            frame.render_widget(status, chunks[2]);
        })?;
        Ok(())
    }

    /// Render the application UI
    fn render_app(&self, _frame: &mut Frame) -> anyhow::Result<()> {
        // Layout will be implemented here
        // For now, just a simple split layout:
        // +-------------------+------------------+
        // |                   |                  |
        // |   File Explorer   |   Queue View     |
        // |                   |                  |
        // +-------------------+------------------+
        // |           Preview View               |
        // +--------------------------------------+
        // |           Status Bar                 |
        // +--------------------------------------+

        // Render main interface using ratatui layout

        Ok(())
    }
}

impl UserInterface for App {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Force initial render
        self.render().map_err(|e| format!("Initial render failed: {}", e))?;
        
        // Main event loop
        while !self.should_exit {
            // Handle events first to avoid blocking on render
            match self.tui.next_event() {
                Ok(Event::Key(key)) => {
                    self.handle_key_event(key).map_err(|e| format!("Key event handling failed: {}", e))?;
                }
                Ok(Event::Resize(_, _)) => {
                    // Terminal was resized, redraw on next iteration
                }
                Ok(Event::Tick) => {
                    // Regular tick event for animations
                }
                Err(e) => {
                    eprintln!("Event error: {}", e);
                    // Continue rather than exit on event errors
                }
            }

            // Draw UI after handling events
            self.render().map_err(|e| format!("Render failed: {}", e))?;
        }

        // Clean up
        self.tui.exit()?;

        Ok(())
    }

    fn open_directory(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        self.current_dir = path.clone();
        self.explorer.change_directory(path)?;
        Ok(())
    }
}
