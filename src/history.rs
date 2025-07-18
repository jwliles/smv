use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a single file operation that can be undone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub timestamp: DateTime<Local>,
}

impl Operation {
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            timestamp: Local::now(),
        }
    }
}

/// History manager for tracking file operations
#[derive(Debug)]
pub struct HistoryManager {
    operations: Vec<Operation>,
    max_history_size: usize,
    backup_directory: PathBuf,
    history_file: PathBuf,
}

impl HistoryManager {
    pub fn new(max_history_size: usize, backup_directory: &Path) -> Self {
        let history_file = backup_directory.join("history.json");
        let mut manager = Self {
            operations: Vec::with_capacity(max_history_size),
            max_history_size,
            backup_directory: backup_directory.to_path_buf(),
            history_file,
        };
        // Load existing history from file
        let _ = manager.load_history();
        manager
    }

    /// Record a new operation
    pub fn record(&mut self, source: PathBuf, destination: PathBuf) -> Result<(), Box<dyn Error>> {
        // Create backup if a source file exists
        if source.exists() {
            self.create_backup(&source)?;
        }

        // Add operation to history
        let operation = Operation::new(source, destination);
        self.operations.push(operation);

        // Trim history if needed
        if self.operations.len() > self.max_history_size {
            self.operations.remove(0);
        }

        // Save history to file
        self.save_history()?;

        Ok(())
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(operation) = self.operations.pop() {
            // Check if this was a file creation operation (source is empty)
            if operation.source.as_os_str().is_empty() {
                // This was a file creation - delete the created file
                if operation.destination.exists() {
                    fs::remove_file(&operation.destination)?;
                    println!(
                        "Undone: Deleted created file '{}'",
                        operation.destination.display()
                    );
                } else {
                    println!(
                        "File '{}' was already deleted or doesn't exist",
                        operation.destination.display()
                    );
                }
            }
            // If the destination exists, move it back to source
            else if operation.destination.exists() {
                fs::rename(&operation.destination, &operation.source)?;
                println!(
                    "Undone: Moved '{}' back to '{}'",
                    operation.destination.display(),
                    operation.source.display()
                );
            }
            // If source doesn't exist but we have a backup, restore it
            else if !operation.source.exists() {
                self.restore_backup(&operation.source)?;
                println!(
                    "Undone: Restored '{}' from backup",
                    operation.source.display()
                );
            }
            // Save updated history to file
            self.save_history()?;
            Ok(())
        } else {
            Err("No operations to undo".into())
        }
    }

    /// Get a list of recorded operations
    #[allow(dead_code)]
    pub fn list_operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Create a backup of a file
    fn create_backup(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_directory)?;

        // Create a unique backup name with timestamp
        let filename = file_path
            .file_name()
            .ok_or("Invalid file path")?
            .to_string_lossy();
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("{filename}_{timestamp}");
        let backup_path = self.backup_directory.join(backup_name);

        // Copy the file to backup
        fs::copy(file_path, &backup_path)?;

        Ok(())
    }

    /// Restore a file from backup
    fn restore_backup(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let filename = file_path
            .file_name()
            .ok_or("Invalid file path")?
            .to_string_lossy();

        // Find the most recent backup for this file
        let mut backups: Vec<PathBuf> = fs::read_dir(&self.backup_directory)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let name = path.file_name()?.to_string_lossy();

                if name.starts_with(&*filename) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Sort by modification time (most recent first)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now());
            let b_time = fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now());
            b_time.cmp(&a_time)
        });

        // Restore the most recent backup if found
        if let Some(backup_path) = backups.first() {
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(backup_path, file_path)?;
            Ok(())
        } else {
            Err("No backup found for this file".into())
        }
    }

    /// Save history to disk
    fn save_history(&self) -> Result<(), Box<dyn Error>> {
        // Ensure the directory exists
        if let Some(parent) = self.history_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize operations to JSON
        let json = serde_json::to_string_pretty(&self.operations)?;
        fs::write(&self.history_file, json)?;

        Ok(())
    }

    /// Load history from disk
    fn load_history(&mut self) -> Result<(), Box<dyn Error>> {
        if self.history_file.exists() {
            let json = fs::read_to_string(&self.history_file)?;
            let operations: Vec<Operation> = serde_json::from_str(&json)?;

            // Only keep operations up to max_history_size
            let start_index = if operations.len() > self.max_history_size {
                operations.len() - self.max_history_size
            } else {
                0
            };

            self.operations = operations[start_index..].to_vec();
        }
        Ok(())
    }
}
