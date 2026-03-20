use std::collections::HashMap;
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

impl Default for OperationQueue {
    fn default() -> Self {
        Self::new()
    }
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
    /// Command mode input buffer
    command_buffer: String,
    /// Insert mode rename buffer (holds the name being edited)
    rename_buffer: Option<String>,
    /// Search mode input buffer (None = not searching)
    search_buffer: Option<String>,
    /// History manager for undo support
    history: crate::history::HistoryManager,
}

impl App {
    /// Create a new application
    pub fn new() -> anyhow::Result<Self> {
        let tui = Tui::new()?;
        Tui::init_panic_hook();

        let current_dir = std::env::current_dir()?;
        let mut explorer = FileExplorer::new(current_dir.clone());

        eprintln!(
            "DEBUG: Loaded {} files in {}",
            explorer.files.len(),
            current_dir.display()
        );

        if explorer.files.is_empty() {
            eprintln!("DEBUG: No files found, adding placeholder");
            explorer.files.push(FileItem {
                name: "No files found".to_string(),
                path: current_dir.clone(),
                is_dir: false,
                is_symlink: false,
                size: 0,
            });
        }

        let backup_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/smv");
        let history = crate::history::HistoryManager::new(100, &backup_dir);

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
            status_message: String::from("Press ? for help. j/k: navigate, Ctrl+Q: quit"),
            command_buffer: String::new(),
            rename_buffer: None,
            search_buffer: None,
            history,
        })
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Search mode intercepts all keys before mode dispatch
        if self.search_buffer.is_some() {
            return self.handle_search_key(key);
        }

        // Global key handlers (work in any mode)
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_exit = true;
                return Ok(());
            }
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                self.undo_last_operation();
                return Ok(());
            }
            (KeyCode::Char('?'), KeyModifiers::NONE)
                if !matches!(self.mode, AppMode::Command | AppMode::Insert) =>
            {
                self.mode = AppMode::Help;
                self.status_message = String::from("Help mode — press ESC or ? to exit");
                return Ok(());
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.mode = AppMode::Normal;
                self.command_buffer.clear();
                self.rename_buffer = None;
                self.status_message = String::from("Normal mode");
                return Ok(());
            }
            _ => {}
        }

        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key)?,
            AppMode::Visual => self.handle_visual_mode_key(key)?,
            AppMode::Command => self.handle_command_mode_key(key)?,
            AppMode::Insert => self.handle_insert_mode_key(key)?,
            AppMode::Help => self.handle_help_mode_key(key)?,
        }

        Ok(())
    }

    /// Handle search-mode keys (active when search_buffer is Some)
    fn handle_search_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                if let Some(buf) = &mut self.search_buffer {
                    buf.push(c);
                    let pattern = buf.clone();
                    self.explorer.start_search(&pattern);
                    let shown = self.explorer.display_count();
                    let total = self.explorer.files.len();
                    self.status_message = format!("/{pattern} [{shown}/{total} files]");
                }
            }
            KeyCode::Backspace => {
                if let Some(buf) = &mut self.search_buffer {
                    buf.pop();
                    if buf.is_empty() {
                        self.explorer.clear_search();
                        self.status_message = String::from("/");
                    } else {
                        let pattern = buf.clone();
                        self.explorer.start_search(&pattern);
                        let shown = self.explorer.display_count();
                        let total = self.explorer.files.len();
                        self.status_message = format!("/{pattern} [{shown}/{total} files]");
                    }
                }
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.search_buffer = None;
                if !self.explorer.is_search_active() {
                    self.status_message = String::from("Normal mode");
                } else {
                    let shown = self.explorer.display_count();
                    let total = self.explorer.files.len();
                    let pattern = self.explorer.search_pattern().unwrap_or("").to_string();
                    self.status_message = format!("Filter: /{pattern} [{shown}/{total}] — ESC to clear");
                    if key.code == KeyCode::Esc {
                        self.explorer.clear_search();
                        self.search_buffer = None;
                        self.status_message = String::from("Search cleared");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Undo the last recorded operation
    fn undo_last_operation(&mut self) {
        match self.history.undo() {
            Ok(_) => {
                let _ = self.explorer.reload_files();
                self.status_message = String::from("Undone last operation");
            }
            Err(e) => {
                self.status_message = format!("Nothing to undo: {e}");
            }
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // Queue navigation keys take priority (J/K = prev/next, D = delete selected)
        match key.code {
            KeyCode::Char('J') => {
                self.queue.select_next();
                let idx = self.queue.selected_index();
                let total = self.queue.operations().len();
                if total > 0 {
                    self.status_message = format!("Queue: item {}/{}", idx + 1, total);
                }
                return Ok(());
            }
            KeyCode::Char('K') => {
                self.queue.select_prev();
                let idx = self.queue.selected_index();
                let total = self.queue.operations().len();
                if total > 0 {
                    self.status_message = format!("Queue: item {}/{}", idx + 1, total);
                }
                return Ok(());
            }
            KeyCode::Char('D') => {
                if !self.queue.is_empty() {
                    self.queue.remove_selected();
                    self.preview.set_operations(self.queue.operations());
                    self.status_message = format!(
                        "Removed from queue. {} operations remain",
                        self.queue.operations().len()
                    );
                }
                return Ok(());
            }
            _ => {}
        }

        // Try explorer key handler
        match self.explorer.handle_key(key, &self.mode) {
            KeyResult::Handled(action) => {
                if let Some(action) = action {
                    self.handle_ui_action(action)?;
                }
                return Ok(());
            }
            KeyResult::NotHandled => {}
        }

        // Application-level keys
        match (key.code, key.modifiers) {
            (KeyCode::Char('v'), KeyModifiers::NONE) => {
                self.mode = AppMode::Visual;
                self.status_message = String::from("Visual mode — j/k to extend, s/c/t for transforms, Enter to apply");
            }
            (KeyCode::Char(':'), KeyModifiers::NONE) => {
                self.mode = AppMode::Command;
                self.command_buffer.clear();
                self.status_message = String::from(":");
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.search_buffer = Some(String::new());
                self.status_message = String::from("/");
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                if let Some(file) = self.explorer.selected().cloned() {
                    if !file.is_dir {
                        self.rename_buffer = Some(file.name.clone());
                        self.mode = AppMode::Insert;
                        self.status_message =
                            format!("Rename (Enter to confirm, Esc to cancel): {}", file.name);
                    }
                }
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                self.handle_ui_action(UiAction::ExecuteQueue)?;
            }
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.queue.clear();
                self.preview.set_operations(self.queue.operations());
                self.status_message = String::from("Queue cleared");
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keys in visual mode
    fn handle_visual_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
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

    /// Handle keys in command mode (text input)
    fn handle_command_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                self.status_message = format!(":{}", self.command_buffer);
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                if self.command_buffer.is_empty() {
                    self.mode = AppMode::Normal;
                    self.status_message = String::from("Normal mode");
                } else {
                    self.status_message = format!(":{}", self.command_buffer);
                }
            }
            KeyCode::Enter => {
                let cmd = self.command_buffer.trim().to_string();
                self.command_buffer.clear();
                self.mode = AppMode::Normal;
                self.dispatch_command(&cmd)?;
            }
            KeyCode::Esc => {
                self.command_buffer.clear();
                self.mode = AppMode::Normal;
                self.status_message = String::from("Normal mode");
            }
            _ => {}
        }
        Ok(())
    }

    /// Dispatch a parsed command string
    fn dispatch_command(&mut self, cmd: &str) -> anyhow::Result<()> {
        let action: Option<UiAction> = match cmd {
            "snake" => Some(UiAction::Transform(TransformAction::Snake)),
            "kebab" => Some(UiAction::Transform(TransformAction::Kebab)),
            "clean" => Some(UiAction::Transform(TransformAction::Clean)),
            "title" => Some(UiAction::Transform(TransformAction::Title)),
            "camel" => Some(UiAction::Transform(TransformAction::Camel)),
            "pascal" => Some(UiAction::Transform(TransformAction::Pascal)),
            "lower" => Some(UiAction::Transform(TransformAction::Lower)),
            "upper" => Some(UiAction::Transform(TransformAction::Upper)),
            "sentence" => Some(UiAction::Transform(TransformAction::Sentence)),
            "start" => Some(UiAction::Transform(TransformAction::Start)),
            "studly" => Some(UiAction::Transform(TransformAction::Studly)),
            "split snake" => Some(UiAction::Transform(TransformAction::SplitSnake)),
            "split kebab" => Some(UiAction::Transform(TransformAction::SplitKebab)),
            "split title" => Some(UiAction::Transform(TransformAction::SplitTitle)),
            "split camel" => Some(UiAction::Transform(TransformAction::SplitCamel)),
            "split pascal" => Some(UiAction::Transform(TransformAction::SplitPascal)),
            "split lower" => Some(UiAction::Transform(TransformAction::SplitLower)),
            "split upper" => Some(UiAction::Transform(TransformAction::SplitUpper)),
            "split sentence" => Some(UiAction::Transform(TransformAction::SplitSentence)),
            "split start" => Some(UiAction::Transform(TransformAction::SplitStart)),
            "split studly" => Some(UiAction::Transform(TransformAction::SplitStudly)),
            "exec" | "execute" | "x" => Some(UiAction::ExecuteQueue),
            "clear" => {
                self.queue.clear();
                self.preview.set_operations(self.queue.operations());
                self.status_message = String::from("Queue cleared");
                None
            }
            "q" | "quit" => {
                self.should_exit = true;
                None
            }
            _ => {
                // Try replace: replace "old" "new" or replace old new
                if let Some(rest) = cmd.strip_prefix("replace ") {
                    if let Some((find, rep)) = parse_two_args(rest) {
                        Some(UiAction::Transform(TransformAction::Replace(find, rep)))
                    } else {
                        self.status_message = format!("Usage: replace <find> <replacement>");
                        None
                    }
                } else if let Some(rest) = cmd.strip_prefix("regex ") {
                    if let Some((pattern, rep)) = parse_two_args(rest) {
                        Some(UiAction::Transform(TransformAction::ReplaceRegex(pattern, rep)))
                    } else {
                        self.status_message = format!("Usage: regex <pattern> <replacement>");
                        None
                    }
                } else if let Some(path) = cmd.strip_prefix("cd ") {
                    let p = PathBuf::from(path.trim());
                    if p.is_dir() {
                        self.current_dir = p.clone();
                        let _ = self.explorer.change_directory(p);
                        self.status_message = format!("Changed to {}", self.current_dir.display());
                    } else {
                        self.status_message = format!("Not a directory: {}", p.display());
                    }
                    None
                } else if cmd.is_empty() {
                    None
                } else {
                    self.status_message = format!("Unknown command: {cmd}");
                    None
                }
            }
        };

        if let Some(action) = action {
            self.handle_ui_action(action)?;
        }
        Ok(())
    }

    /// Handle keys in insert mode (rename)
    fn handle_insert_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                if let Some(buf) = &mut self.rename_buffer {
                    buf.push(c);
                    self.status_message = format!("Rename: {buf}");
                }
            }
            KeyCode::Backspace => {
                if let Some(buf) = &mut self.rename_buffer {
                    buf.pop();
                    self.status_message = format!("Rename: {buf}");
                }
            }
            KeyCode::Enter => {
                if let Some(new_name) = self.rename_buffer.take() {
                    if !new_name.is_empty() {
                        if let Err(e) = self.perform_rename(&new_name) {
                            self.status_message = format!("Rename failed: {e}");
                        }
                    }
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                self.rename_buffer = None;
                self.mode = AppMode::Normal;
                self.status_message = String::from("Rename cancelled");
            }
            _ => {}
        }
        Ok(())
    }

    /// Rename the currently selected file
    fn perform_rename(&mut self, new_name: &str) -> anyhow::Result<()> {
        if let Some(file) = self.explorer.selected().cloned() {
            if !file.is_dir {
                let parent = file
                    .path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
                let new_path = parent.join(new_name);
                std::fs::rename(&file.path, &new_path)?;
                let _ = self.history.record(file.path, new_path);
                let _ = self.explorer.reload_files();
                self.status_message = format!("Renamed to {new_name}");
            }
        }
        Ok(())
    }

    /// Handle keys in help mode
    fn handle_help_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
                self.status_message = String::from("Normal mode");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle a UI action
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
                            destination: file.path.clone(),
                            operation_type: OperationType::Move,
                        };
                        self.queue.add(operation);
                        added_count += 1;
                    }
                }

                self.preview.set_operations(self.queue.operations());
                if added_count > 0 {
                    self.status_message = format!("Added {added_count} file(s) to queue");
                } else {
                    self.status_message =
                        String::from("No files to add (directories are ignored)");
                }
            }
            UiAction::Transform(transform_action) => {
                let files_to_transform: Vec<_> = self
                    .explorer
                    .visual_selection()
                    .into_iter()
                    .cloned()
                    .collect();
                let mut added_count = 0;

                for file in files_to_transform {
                    if !file.is_dir {
                        self.add_transform_to_queue(&file, &transform_action)?;
                        added_count += 1;
                    }
                }

                self.preview.set_operations(self.queue.operations());
                if added_count > 0 {
                    self.status_message = format!(
                        "Queued {} file(s) for {} transform",
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
        transform_action: &TransformAction,
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
            TransformAction::Sentence => crate::transformers::TransformType::Sentence,
            TransformAction::Start => crate::transformers::TransformType::Start,
            TransformAction::Studly => crate::transformers::TransformType::Studly,
            TransformAction::SplitSnake => crate::transformers::TransformType::SplitSnake,
            TransformAction::SplitKebab => crate::transformers::TransformType::SplitKebab,
            TransformAction::SplitTitle => crate::transformers::TransformType::SplitTitle,
            TransformAction::SplitCamel => crate::transformers::TransformType::SplitCamel,
            TransformAction::SplitPascal => crate::transformers::TransformType::SplitPascal,
            TransformAction::SplitLower => crate::transformers::TransformType::SplitLower,
            TransformAction::SplitUpper => crate::transformers::TransformType::SplitUpper,
            TransformAction::SplitSentence => crate::transformers::TransformType::SplitSentence,
            TransformAction::SplitStart => crate::transformers::TransformType::SplitStart,
            TransformAction::SplitStudly => crate::transformers::TransformType::SplitStudly,
            TransformAction::Replace(f, r) => {
                crate::transformers::TransformType::Replace(f.clone(), r.clone())
            }
            TransformAction::ReplaceRegex(p, r) => {
                crate::transformers::TransformType::ReplaceRegex(p.clone(), r.clone())
            }
        };

        let filename = file
            .path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy();
        let new_filename = transform(&filename, &transform_type);

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
                    let _ =
                        self.history
                            .record(operation.source.clone(), operation.destination.clone());
                    success_count += 1;
                }
                Err(_e) => {
                    error_count += 1;
                }
            }
        }

        self.queue.clear();
        self.preview.set_operations(self.queue.operations());
        self.status_message = format!("Executed: {success_count} success, {error_count} errors");

        let _ = self.explorer.reload_files();
        Ok(())
    }

    /// Group files by basename in the selected directory
    fn group_files_in_directory(&mut self, dir_path: &PathBuf) -> anyhow::Result<()> {
        match sort::group_by_basename(&dir_path.to_string_lossy(), false) {
            Ok(_) => {
                self.status_message = format!("Grouped files in {}", dir_path.display());
                let _ = self.explorer.reload_files();
            }
            Err(e) => {
                self.status_message = format!("Error grouping files: {e}");
            }
        }
        Ok(())
    }

    /// Flatten the selected directory structure
    fn flatten_directory(&mut self, dir_path: &PathBuf) -> anyhow::Result<()> {
        match unsort::flatten_directory(&dir_path.to_string_lossy(), false) {
            Ok(_) => {
                let _ = unsort::remove_empty_dirs(&dir_path.to_string_lossy(), false);
                self.status_message = format!("Flattened directory {}", dir_path.display());
                let _ = self.explorer.reload_files();
            }
            Err(e) => {
                self.status_message = format!("Error flattening directory: {e}");
            }
        }
        Ok(())
    }

    /// Main render function
    fn render(&mut self) -> anyhow::Result<()> {
        // Snapshot data needed in the closure
        let current_dir = self.current_dir.display().to_string();
        let mode = format!("{:?}", self.mode);
        let queue_len = self.queue.operations().len();
        let queue_selected = self.queue.selected_index();

        // Build a set of queued source paths for overlay lookup
        let queue_map: HashMap<PathBuf, (PathBuf, &OperationType)> = self
            .queue
            .operations()
            .iter()
            .map(|op| (op.source.clone(), (op.destination.clone(), &op.operation_type)))
            .collect();

        let selected_index = self.explorer.state.selected();
        let visual_start = if matches!(self.mode, AppMode::Visual) {
            self.explorer.visual_selection_start
        } else {
            None
        };

        // Build display file list (respects filter)
        let display_files: Vec<(String, PathBuf, bool, usize)> = {
            let df = self.explorer.display_files();
            df.iter()
                .enumerate()
                .map(|(display_idx, file)| {
                    (file.name.clone(), file.path.clone(), file.is_dir, display_idx)
                })
                .collect()
        };

        let is_search_active = self.explorer.is_search_active();
        let total_files = self.explorer.files.len();
        let shown_files = display_files.len();

        let search_buf = self.search_buffer.clone();
        let command_buf = self.command_buffer.clone();
        let rename_buf = self.rename_buffer.clone();

        // Build status text based on mode
        let status_text = {
            let msg = &self.status_message;
            match self.mode {
                AppMode::Command => format!(":{command_buf}"),
                AppMode::Insert => format!(
                    "RENAME: {}",
                    rename_buf.as_deref().unwrap_or("")
                ),
                _ => {
                    if let Some(ref sbuf) = search_buf {
                        if is_search_active {
                            format!("/{sbuf} [{shown_files}/{total_files}]")
                        } else {
                            format!("/{sbuf}")
                        }
                    } else {
                        msg.clone()
                    }
                }
            }
        };

        let mode_clone = self.mode;
        let should_show_help = matches!(self.mode, AppMode::Help);

        self.tui.draw(|frame| {
            use ratatui::{
                layout::{Constraint, Direction, Layout},
                style::{Color, Modifier, Style},
                text::{Line, Span},
                widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
            };

            let size = frame.size();

            // Main layout: header | content | status
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);

            // Header
            let header_title = if is_search_active {
                format!("SMV — {current_dir}  [filter: {shown_files}/{total_files}]")
            } else {
                format!("SMV — {current_dir}")
            };
            let header = Paragraph::new(header_title)
                .block(Block::default().borders(Borders::ALL).title("Smart Move"))
                .style(Style::default().fg(Color::Cyan));
            frame.render_widget(header, chunks[0]);

            // Content area: file explorer | right panel
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(65),
                    Constraint::Percentage(35),
                ])
                .split(chunks[1]);

            // Right panel: queue (top 60%) | preview (bottom 40%)
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ])
                .split(main_chunks[1]);

            // ── File explorer ──
            let explorer_content: Vec<ListItem> = display_files
                .iter()
                .map(|(name, path, is_dir, display_idx)| {
                    let icon = if *is_dir { "📁" } else { "📄" };

                    // Visual selection range
                    let in_visual = if let (Some(start), Some(current)) =
                        (visual_start, selected_index)
                    {
                        let (min, max) =
                            if start <= current { (start, current) } else { (current, start) };
                        *display_idx >= min && *display_idx <= max
                    } else {
                        false
                    };

                    // Queue overlay annotation
                    let queue_annotation: Option<(String, Color)> =
                        if let Some((dest, _op_type)) = queue_map.get(path) {
                            let dest_name = dest
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if dest_name == *name {
                                Some((" [queued]".to_string(), Color::DarkGray))
                            } else {
                                Some((format!(" → {dest_name}"), Color::Yellow))
                            }
                        } else {
                            None
                        };

                    let base_style = if in_visual {
                        Style::default().fg(Color::Black).bg(Color::Blue)
                    } else if *is_dir {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if in_visual { "► " } else { "  " };

                    let mut spans = vec![
                        Span::styled(format!("{prefix}{icon} "), base_style),
                        Span::styled(name.clone(), base_style),
                    ];

                    if let Some((annotation, color)) = queue_annotation {
                        spans.push(Span::styled(annotation, Style::default().fg(color)));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect();

            let explorer_title = if is_search_active {
                format!(" Files [{shown_files}/{total_files}] ")
            } else {
                " Files ".to_string()
            };

            let explorer_widget = List::new(explorer_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(explorer_title),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_stateful_widget(
                explorer_widget,
                main_chunks[0],
                &mut self.explorer.state,
            );

            // ── Queue panel ──
            let queue_content = if queue_len > 0 {
                let mut items =
                    vec![ListItem::new(format!("📝 {queue_len} operation(s) pending:"))];

                for (i, op) in self.queue.operations().iter().enumerate() {
                    let op_icon = match &op.operation_type {
                        OperationType::Move => "📁",
                        OperationType::Transform(t) => match t {
                            crate::transformers::TransformType::Snake => "🐍",
                            crate::transformers::TransformType::Kebab => "🍢",
                            crate::transformers::TransformType::Clean => "🧹",
                            crate::transformers::TransformType::Title => "📚",
                            crate::transformers::TransformType::Sentence
                            | crate::transformers::TransformType::Start
                            | crate::transformers::TransformType::Studly => "✏️",
                            _ => "✏️",
                        },
                    };

                    let src = op
                        .source
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_else(|| "<unknown>".into());
                    let dst = op
                        .destination
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_else(|| "<unknown>".into());

                    let text = if src == dst {
                        format!("{op_icon} {src}")
                    } else {
                        format!("{op_icon} {src} → {dst}")
                    };

                    let style = if i == queue_selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    items.push(ListItem::new(text).style(style));
                }

                items.push(ListItem::new(""));
                items.push(ListItem::new(
                    Line::from(vec![
                        Span::styled("x", Style::default().fg(Color::Green)),
                        Span::raw(" execute  "),
                        Span::styled("q", Style::default().fg(Color::Yellow)),
                        Span::raw(" clear  "),
                        Span::styled("J/K", Style::default().fg(Color::Cyan)),
                        Span::raw(" nav  "),
                        Span::styled("D", Style::default().fg(Color::Red)),
                        Span::raw(" delete"),
                    ])
                ));
                items
            } else {
                vec![
                    ListItem::new(
                        Span::styled("No operations queued", Style::default().fg(Color::DarkGray))
                    ),
                    ListItem::new(""),
                    ListItem::new("Select files, then:"),
                    ListItem::new(
                        Line::from(vec![
                            Span::styled(" s", Style::default().fg(Color::Green)),
                            Span::raw("=snake  "),
                            Span::styled("c", Style::default().fg(Color::Green)),
                            Span::raw("=clean  "),
                            Span::styled("t", Style::default().fg(Color::Green)),
                            Span::raw("=title"),
                        ])
                    ),
                    ListItem::new(
                        Line::from(vec![
                            Span::styled(" r", Style::default().fg(Color::Green)),
                            Span::raw("=rename  "),
                            Span::styled(":", Style::default().fg(Color::Cyan)),
                            Span::raw("=command mode"),
                        ])
                    ),
                ]
            };

            let queue_widget = List::new(queue_content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Queue ({queue_len}) ")),
            );
            frame.render_widget(queue_widget, right_chunks[0]);

            // ── Preview panel ──
            self.preview.render(frame, right_chunks[1]);

            // ── Status bar ──
            let nav_hint = match mode_clone {
                AppMode::Normal => {
                    if is_search_active {
                        "/ search active — ESC to clear | j/k nav | s/c/t transforms | ?: help"
                    } else {
                        "j/k nav | /: search | r: rename | :: cmd | v: visual | x: exec | ?: help | Ctrl+Q: quit"
                    }
                }
                AppMode::Visual => "j/k extend | Enter/s/c/t apply | Esc: normal",
                AppMode::Command => "Type command, Enter to execute, Esc to cancel",
                AppMode::Insert => "Type new name, Enter to confirm, Esc to cancel",
                AppMode::Help => "ESC/? to close help",
            };
            let full_status = format!("[{mode}] {status_text} | {nav_hint}");
            let status = Paragraph::new(full_status)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });
            frame.render_widget(status, chunks[2]);

            // ── Help overlay ──
            if should_show_help {
                use ratatui::{
                    layout::Alignment,
                    widgets::{Clear, Paragraph},
                };

                let help_area = ratatui::layout::Rect {
                    x: size.width / 6,
                    y: size.height / 8,
                    width: size.width * 2 / 3,
                    height: size.height * 3 / 4,
                };
                frame.render_widget(Clear, help_area);

                let help_text = "\
SMV Terminal UI — Help

NAVIGATION:
  j/k  ↓↑   Navigate file list
  h/l  ←→   Parent / enter directory
  g/G        First / last item
  /          Start search filter (live)
  f          Fuzzy search (skim)

TRANSFORMS (queue the selected file):
  s          snake_case
  c          clean (normalize spaces/chars)
  t          Title Case
  Enter      Add selected file to queue

RENAME:
  r          Rename selected file (insert mode)

MODES:
  v          Visual mode (multi-select)
  :          Command mode (type command + Enter)
  Esc        Return to normal mode

COMMAND MODE EXAMPLES:
  :snake     :kebab   :title   :camel   :pascal
  :sentence  :start   :studly  :lower   :upper
  :split snake  :split kebab  ... (all case variants)
  :replace old new   :regex pattern repl
  :cd <path>  :exec   :clear   :q

QUEUE:
  x          Execute all queued operations
  q          Clear queue
  J/K        Navigate queue selection
  D          Delete selected queue item
  Ctrl+Z     Undo last executed operation

Press ESC, ?, or q to close.
";
                let popup = Paragraph::new(help_text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Help ")
                            .title_alignment(Alignment::Center),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });
                frame.render_widget(popup, help_area);
            }
        })?;
        Ok(())
    }

    /// Unused stub kept for interface compatibility
    fn render_app(&self, _frame: &mut Frame) -> anyhow::Result<()> {
        Ok(())
    }
}

impl UserInterface for App {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.render()
            .map_err(|e| format!("Initial render failed: {e}"))?;

        while !self.should_exit {
            match self.tui.next_event() {
                Ok(Event::Key(key)) => {
                    self.handle_key_event(key)
                        .map_err(|e| format!("Key event handling failed: {e}"))?;
                }
                Ok(Event::Resize(_, _)) => {}
                Ok(Event::Tick) => {}
                Err(e) => {
                    eprintln!("Event error: {e}");
                }
            }

            self.render().map_err(|e| format!("Render failed: {e}"))?;
        }

        self.tui.exit()?;
        Ok(())
    }

    fn open_directory(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        self.current_dir = path.clone();
        self.explorer.change_directory(path)?;
        Ok(())
    }
}

/// Parse two arguments from a string, supporting quoted strings
fn parse_two_args(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    if s.starts_with('"') {
        let inner = &s[1..];
        if let Some(end) = inner.find('"') {
            let first = inner[..end].to_string();
            let rest = inner[end + 1..].trim();
            let second = if rest.starts_with('"') {
                let rest = &rest[1..];
                rest.find('"').map(|e| rest[..e].to_string())
            } else {
                Some(rest.to_string())
            };
            return second.filter(|r| !r.is_empty()).map(|r| (first, r));
        }
    }
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}
