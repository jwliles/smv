use std::error::Error;
use std::fs;
pub use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::ui::terminal::{AppMode, KeyResult};
use crate::ui::{TransformAction, UiAction};
use skim::prelude::*;

/// File information for display
#[derive(Clone, Debug)]
pub struct FileItem {
    /// File name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Is this a directory?
    pub is_dir: bool,
    /// Is this a symlink?
    pub is_symlink: bool,
    /// Size in bytes
    pub size: u64,
}

/// The file explorer view
pub struct FileExplorer {
    /// Current directory
    current_dir: PathBuf,
    /// Files in the current directory
    pub files: Vec<FileItem>,
    /// List selection state
    pub state: ListState,
    /// Visual selection start
    pub visual_selection_start: Option<usize>,
    /// Current search pattern (if any)
    search_pattern: Option<String>,
    /// Filtered files based on search
    filtered_files: Vec<usize>,
}

impl FileExplorer {
    /// Create a new file explorer
    pub fn new(dir: PathBuf) -> Self {
        let mut explorer = Self {
            current_dir: dir,
            files: Vec::new(),
            state: ListState::default(),
            visual_selection_start: None,
            search_pattern: None,
            filtered_files: Vec::new(),
        };

        // Load initial directory
        let _ = explorer.reload_files();
        explorer.state.select(Some(0));

        explorer
    }

    /// Change directory
    pub fn change_directory(&mut self, dir: PathBuf) -> Result<(), Box<dyn Error>> {
        self.current_dir = dir;
        self.reload_files()?;
        self.state.select(Some(0));
        self.visual_selection_start = None;
        self.search_pattern = None;
        self.filtered_files.clear();
        Ok(())
    }

    /// Reload files in the current directory
    pub fn reload_files(&mut self) -> Result<(), Box<dyn Error>> {
        self.files.clear();

        // Always add a parent directory entry
        let parent = self
            .current_dir
            .parent()
            .unwrap_or(&self.current_dir)
            .to_path_buf();
        self.files.push(FileItem {
            name: "..".to_string(),
            path: parent,
            is_dir: true,
            is_symlink: false,
            size: 0,
        });

        // Read directory entries
        let entries = fs::read_dir(&self.current_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            self.files.push(FileItem {
                name: entry.file_name().to_string_lossy().to_string(),
                path,
                is_dir: metadata.is_dir(),
                is_symlink: metadata.file_type().is_symlink(),
                size: metadata.len(),
            });
        }

        // Sort: directories first, then files alphabetically
        self.files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        // Update filtered files if the search is active
        if let Some(pattern) = self.search_pattern.as_ref() {
            let pattern = pattern.clone();
            self.filter_files(&pattern);
        }

        Ok(())
    }

    /// Get the selected file
    pub fn selected(&self) -> Option<&FileItem> {
        let index = self.state.selected()?;

        if !self.filtered_files.is_empty() {
            // If filtered, map through filtered indexes
            let actual_index = self.filtered_files.get(index)?;
            self.files.get(*actual_index)
        } else {
            // Otherwise use direct index
            self.files.get(index)
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent, mode: &AppMode) -> KeyResult {
        match mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Visual => self.handle_visual_key(key),
            _ => KeyResult::NotHandled,
        }
    }

    /// Start fuzzy search using skim
    pub fn start_fuzzy_search(&mut self) -> Result<(), Box<dyn Error>> {
        // Create the input source from file names
        let file_names: Vec<String> = self.files.iter().map(|f| f.name.clone()).collect();

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(std::io::Cursor::new(file_names.join("\n")));

        // Create skim options
        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .multi(true)
            .build()
            .unwrap();

        // Run skim
        let selected_items = Skim::run_with(&options, Some(items))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| Vec::new());

        // Process selected items
        if !selected_items.is_empty() {
            // For now, select the first matched item
            if let Some(item) = selected_items.first() {
                let text = item.text();
                // Find the corresponding index in files
                for (i, file) in self.files.iter().enumerate() {
                    if file.name == text {
                        self.state.select(Some(i));
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Filter files by pattern
    fn filter_files(&mut self, pattern: &str) {
        self.filtered_files.clear();

        // Simple substring search for now
        let pattern = pattern.to_lowercase();
        for (i, file) in self.files.iter().enumerate() {
            if file.name.to_lowercase().contains(&pattern) {
                self.filtered_files.push(i);
            }
        }

        // Reset selection
        if !self.filtered_files.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next(1);
                KeyResult::Handled(None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev(1);
                KeyResult::Handled(None)
            }
            KeyCode::PageDown => {
                self.select_next(10);
                KeyResult::Handled(None)
            }
            KeyCode::PageUp => {
                self.select_prev(10);
                KeyResult::Handled(None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
                KeyResult::Handled(None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
                KeyResult::Handled(None)
            }

            // Directory navigation
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(item) = self.selected() {
                    if item.is_dir {
                        let _ = self.change_directory(item.path.clone());
                        return KeyResult::Handled(None);
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(parent) = self.current_dir.parent() {
                    let _ = self.change_directory(parent.to_path_buf());
                }
                KeyResult::Handled(None)
            }

            // Search
            KeyCode::Char('/') => {
                // Start search (would be implemented with a search prompt)
                KeyResult::Handled(None)
            }
            KeyCode::Char('f') => {
                // Start a fuzzy search
                let _ = self.start_fuzzy_search();
                KeyResult::Handled(None)
            }

            // Transformation shortcuts
            KeyCode::Char('s') => {
                // Snake case transformation
                if let Some(item) = self.selected() {
                    if !item.is_dir {
                        // Add to queue for snake_case transformation
                        // This will be handled by the parent app
                        return KeyResult::Handled(Some(UiAction::Transform(
                            TransformAction::Snake,
                        )));
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Char('K') => {
                // Kebab case transformation
                if let Some(item) = self.selected() {
                    if !item.is_dir {
                        return KeyResult::Handled(Some(UiAction::Transform(
                            TransformAction::Kebab,
                        )));
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Char('c') => {
                // Clean transformation
                if let Some(item) = self.selected() {
                    if !item.is_dir {
                        return KeyResult::Handled(Some(UiAction::Transform(
                            TransformAction::Clean,
                        )));
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Char('t') => {
                // Title case transformation
                if let Some(item) = self.selected() {
                    if !item.is_dir {
                        return KeyResult::Handled(Some(UiAction::Transform(
                            TransformAction::Title,
                        )));
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Char('o') => {
                // Group files by basename (if current item is a directory)
                if let Some(item) = self.selected() {
                    if item.is_dir {
                        return KeyResult::Handled(Some(UiAction::GroupFiles));
                    }
                }
                KeyResult::Handled(None)
            }
            KeyCode::Char('O') => {
                // Flatten directory (if current item is a directory)
                if let Some(item) = self.selected() {
                    if item.is_dir {
                        return KeyResult::Handled(Some(UiAction::FlattenDirectory));
                    }
                }
                KeyResult::Handled(None)
            }

            // Actions
            KeyCode::Enter => {
                // Select current file/directory
                if let Some(item) = self.selected() {
                    if item.is_dir {
                        let _ = self.change_directory(item.path.clone());
                    } else {
                        // Add file to operation queue
                        return KeyResult::Handled(Some(UiAction::AddToQueue));
                    }
                }
                KeyResult::Handled(None)
            }

            _ => KeyResult::NotHandled,
        }
    }

    /// Handle keys in visual mode
    fn handle_visual_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            // Navigation (same as normal mode)
            KeyCode::Down | KeyCode::Char('j') => {
                // Set the start of selection if not set
                if self.visual_selection_start.is_none() {
                    self.visual_selection_start = self.state.selected();
                }
                self.select_next(1);
                KeyResult::Handled(None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Set the start of selection if not set
                if self.visual_selection_start.is_none() {
                    self.visual_selection_start = self.state.selected();
                }
                self.select_prev(1);
                KeyResult::Handled(None)
            }

            // Visual mode actions
            KeyCode::Char('y') => {
                // Yank (copy) selected files
                self.visual_selection_start = None;
                KeyResult::Handled(Some(UiAction::Continue))
            }
            KeyCode::Char('d') => {
                // Delete selected files
                self.visual_selection_start = None;
                KeyResult::Handled(Some(UiAction::Continue))
            }

            _ => KeyResult::NotHandled,
        }
    }

    /// Get all files in the visual selection
    pub fn visual_selection(&self) -> Vec<&FileItem> {
        let mut result = Vec::new();

        if let (Some(start), Some(current)) = (self.visual_selection_start, self.state.selected()) {
            let (min, max) = if start <= current {
                (start, current)
            } else {
                (current, start)
            };

            for i in min..=max {
                if let Some(file) = self.files.get(i) {
                    result.push(file);
                }
            }
        } else if let Some(current) = self.state.selected() {
            if let Some(file) = self.files.get(current) {
                result.push(file);
            }
        }

        result
    }

    /// Select the next item
    fn select_next(&mut self, count: usize) {
        if self.filtered_files.is_empty() {
            // No filter active
            let len = self.files.len();
            if len == 0 {
                return;
            }

            let i = match self.state.selected() {
                Some(i) => {
                    if i + count < len {
                        i + count
                    } else {
                        len - 1 // Stop at last item instead of wrapping
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        } else {
            // Filter active
            let len = self.filtered_files.len();
            if len == 0 {
                return;
            }

            let i = match self.state.selected() {
                Some(i) => {
                    if i + count < len {
                        i + count
                    } else {
                        len - 1 // Stop at last item instead of wrapping
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    /// Select the previous item
    fn select_prev(&mut self, count: usize) {
        if self.filtered_files.is_empty() {
            // No filter active
            let len = self.files.len();
            if len == 0 {
                return;
            }

            let i = match self.state.selected() {
                Some(i) => {
                    if i >= count {
                        i - count
                    } else {
                        0 // Stop at first item instead of wrapping
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        } else {
            // Filter active
            let len = self.filtered_files.len();
            if len == 0 {
                return;
            }

            let i = match self.state.selected() {
                Some(i) => {
                    if i >= count {
                        i - count
                    } else {
                        0 // Stop at first item instead of wrapping
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    /// Select the first item
    fn select_first(&mut self) {
        if !self.files.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Select the last item
    fn select_last(&mut self) {
        if self.filtered_files.is_empty() {
            if !self.files.is_empty() {
                self.state.select(Some(self.files.len() - 1));
            }
        } else {
            if !self.filtered_files.is_empty() {
                self.state.select(Some(self.filtered_files.len() - 1));
            }
        }
    }
}
