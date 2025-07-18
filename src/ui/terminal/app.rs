use std::error::Error;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

use crate::transformers::transform;
use crate::ui::terminal::views::{FileExplorer, FileItem, PreviewView, QueueView};
use crate::ui::terminal::{AppMode, Event, KeyResult, Tui};
use crate::ui::{Theme, TransformAction, UiAction, UserInterface};
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
        eprintln!(
            "DEBUG: Loaded {} files in {}",
            explorer.files.len(),
            current_dir.display()
        );

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
                // Toggle help mode
                self.mode = AppMode::Help;
                self.status_message = String::from("Help mode - press ESC or ? to exit");
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
            AppMode::Help => self.handle_help_mode_key(key)?,
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
        // Handle escape to return to normal mode
        if matches!(key.code, KeyCode::Esc) {
            self.mode = AppMode::Normal;
            self.status_message = String::from("Normal mode");
            return Ok(());
        }

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

    /// Handle keys in help mode
    fn handle_help_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                // Exit help mode
                self.mode = AppMode::Normal;
                self.status_message = String::from("Normal mode");
            }
            _ => {
                // Ignore other keys in help mode
            }
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
                // Handle both single file (normal mode) and multiple files (visual mode)
                let files_to_add: Vec<_> = self
                    .explorer
                    .visual_selection()
                    .into_iter()
                    .cloned()
                    .collect();
                let mut added_count = 0;

                for file in files_to_add {
                    if !file.is_dir {
                        let operation = FileOperation {
                            source: file.path.clone(),
                            destination: file.path.clone(), // Will be updated based on operation
                            operation_type: OperationType::Move,
                        };
                        self.queue.add(operation);
                        added_count += 1;
                    }
                }

                if added_count > 0 {
                    self.status_message = format!("Added {} file(s) to queue", added_count);
                } else {
                    self.status_message = String::from("No files to add (directories are ignored)");
                }
            }
            UiAction::Transform(transform_action) => {
                // Handle both single file (normal mode) and multiple files (visual mode)
                let files_to_transform: Vec<_> = self
                    .explorer
                    .visual_selection()
                    .into_iter()
                    .cloned()
                    .collect();
                let mut added_count = 0;

                for file in files_to_transform {
                    if !file.is_dir {
                        self.add_transform_to_queue(&file, transform_action)?;
                        added_count += 1;
                    }
                }

                if added_count > 0 {
                    self.status_message = format!(
                        "Added {} file(s) to queue for {} transformation",
                        added_count,
                        transform_action.as_str()
                    );
                } else {
                    self.status_message =
                        String::from("No files to transform (directories are ignored)");
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
    fn add_transform_to_queue(
        &mut self,
        file: &FileItem,
        transform_action: TransformAction,
    ) -> anyhow::Result<()> {
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
        let filename = file
            .path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy();
        let new_filename = transform(&filename, &transform_type);

        // Create new path with transformed filename
        let new_path = file
            .path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid parent directory"))?
            .join(&new_filename);

        let operation = FileOperation {
            source: file.path.clone(),
            destination: new_path,
            operation_type: OperationType::Transform(transform_type),
        };

        self.queue.add(operation);
        self.status_message = format!(
            "Added {} transformation for {}",
            transform_action.as_str(),
            file.name
        );

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
        self.status_message = format!(
            "Executed: {} success, {} errors",
            success_count, error_count
        );

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
        let selected_index = self.explorer.state.selected();
        let visual_start = if matches!(self.mode, AppMode::Visual) {
            self.explorer.visual_selection_start
        } else {
            None
        };
        let files_data: Vec<(String, bool, usize)> = self
            .explorer
            .files
            .iter()
            .enumerate()
            .map(|(idx, file)| (file.name.clone(), file.is_dir, idx))
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

            // File explorer with real data and visual selection support
            let explorer_content: Vec<ListItem> = files_data.iter()
                .map(|(name, is_dir, idx)| {
                    let icon = if *is_dir { "📁" } else { "📄" };
                    let mut line = format!("{} {}", icon, name);

                    // Add visual selection indicator
                    if let (Some(start), Some(current)) = (visual_start, selected_index) {
                        let (min, max) = if start <= current { (start, current) } else { (current, start) };
                        if *idx >= min && *idx <= max {
                            line = format!("► {}", line);  // Visual selection marker
                        }
                    }

                    ListItem::new(line)
                })
                .collect();

            let explorer = List::new(explorer_content)
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD));

            frame.render_stateful_widget(explorer, main_chunks[0], &mut self.explorer.state);

            // Queue view with detailed operations
            let queue_content = if queue_len > 0 {
                let mut items = vec![ListItem::new(format!("📝 {} operations pending:", queue_len))];

                // Show up to 8 operations in detail
                for (_i, op) in self.queue.operations().iter().take(8).enumerate() {
                    let op_icon = match &op.operation_type {
                        OperationType::Move => "📁",
                        OperationType::Transform(t) => match t {
                            crate::transformers::TransformType::Snake => "🐍",
                            crate::transformers::TransformType::Kebab => "🍢",
                            crate::transformers::TransformType::Clean => "🧹",
                            crate::transformers::TransformType::Title => "📚",
                            _ => "✏️",
                        }
                    };

                    let source_name = op.source.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_else(|| "<unknown>".into());
                    let dest_name = op.destination.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_else(|| "<unknown>".into());

                    let op_text = if source_name == dest_name {
                        format!("{} {}", op_icon, source_name)
                    } else {
                        format!("{} {} → {}", op_icon, source_name, dest_name)
                    };

                    items.push(ListItem::new(op_text));
                }

                if queue_len > 8 {
                    items.push(ListItem::new(format!("... and {} more", queue_len - 8)));
                }

                items.push(ListItem::new(""));
                items.push(ListItem::new("Press 'x' to execute all"));
                items.push(ListItem::new("Press 'q' to clear queue"));

                items
            } else {
                vec![
                    ListItem::new("No operations queued"),
                    ListItem::new(""),
                    ListItem::new("Select files and press:"),
                    ListItem::new("• s = snake_case"),
                    ListItem::new("• c = clean spaces"),
                    ListItem::new("• t = Title Case"),
                    ListItem::new("• K = kebab-case"),
                    ListItem::new("• o = group files"),
                    ListItem::new("• O = flatten dirs"),
                ]
            };

            let queue = List::new(queue_content)
                .block(Block::default().borders(Borders::ALL).title("Operations Queue"))
                .style(Style::default().fg(Color::White));
            frame.render_widget(queue, main_chunks[1]);

            // Status bar with navigation and action help
            let nav_help = match self.mode {
                AppMode::Normal => "j/k: Navigate | Enter: Dir/Add to Queue | h: Back | l: Enter Dir | Actions: s=Snake c=Clean t=Title K=Kebab | v: Visual | x: Execute | q: Clear Queue | ?: Help | Ctrl+Q: Quit",
                AppMode::Visual => "j/k: Extend selection | Enter: Apply to Selection | Esc: Normal mode | Available actions: s c t K o O | ?: Help",
                AppMode::Help => "Press ESC, ?, or q to exit help mode",
                _ => "j/k: Navigate | Enter: select | h: back | l: forward | ?: Help",
            };
            let status_text = format!("Mode: {} | {} | {}", mode, status_message, nav_help);
            let status = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });
            frame.render_widget(status, chunks[2]);

            // Render help overlay if in help mode
            if matches!(self.mode, AppMode::Help) {
                use ratatui::{
                    layout::Alignment,
                    widgets::{Clear, Paragraph},
                };

                // Create a centered help popup
                let help_area = ratatui::layout::Rect {
                    x: size.width / 6,
                    y: size.height / 8,
                    width: size.width * 2 / 3,
                    height: size.height * 3 / 4,
                };

                // Clear the area first
                frame.render_widget(Clear, help_area);

                let help_text = "
🔧 SMV Terminal UI - Help & Actions

📁 NAVIGATION:
  j, ↓    - Move down in file list
  k, ↑    - Move up in file list
  h, ←    - Go back to parent directory
  l, →    - Enter selected directory
  Enter   - Enter directory OR add file to queue
  gg      - Go to first item
  G       - Go to last item

🎯 FILE TRANSFORMATION ACTIONS:
  s       - Convert to snake_case (my_file.txt)
  c       - Clean up spaces & special chars
  t       - Convert to Title Case (My File.txt)
  K       - Convert to kebab-case (my-file.txt)

📂 DIRECTORY OPERATIONS:
  o       - Group files by basename into directories
  O       - Flatten directory (move all files to root)

👁️ MODES:
  v       - Enter Visual mode (select multiple files)
  :       - Enter Command mode
  Esc     - Return to Normal mode

⚡ QUEUE OPERATIONS:
  x       - Execute all queued operations
  q       - Clear the operation queue

🔍 OTHER:
  f       - Fuzzy search (if available)
  /       - Start search

🚪 EXIT:
  Ctrl+Q  - Quit application
  ?       - Toggle this help screen

Press ESC, ?, or q to close this help.
";

                let help_popup = Paragraph::new(help_text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(" Help - SMV Actions & Navigation ")
                        .title_alignment(Alignment::Center))
                    .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });

                frame.render_widget(help_popup, help_area);
            }
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
        self.render()
            .map_err(|e| format!("Initial render failed: {}", e))?;

        // Main event loop
        while !self.should_exit {
            // Handle events first to avoid blocking on render
            match self.tui.next_event() {
                Ok(Event::Key(key)) => {
                    self.handle_key_event(key)
                        .map_err(|e| format!("Key event handling failed: {}", e))?;
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
