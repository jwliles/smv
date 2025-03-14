use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

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
}

impl HistoryManager {
    pub fn new(max_history_size: usize, backup_directory: &Path) -> Self {
        Self {
            operations: Vec::with_capacity(max_history_size),
            max_history_size,
            backup_directory: backup_directory.to_path_buf(),
        }
    }

    /// Record a new operation
    pub fn record(&mut self, source: PathBuf, destination: PathBuf) -> Result<(), Box<dyn Error>> {
        // Create backup if source file exists
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

        Ok(())
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(operation) = self.operations.pop() {
            // If the destination exists, move it back to source
            if operation.destination.exists() {
                fs::rename(&operation.destination, &operation.source)?;
                println!("Undone: Moved '{}' back to '{}'", 
                    operation.destination.display(), 
                    operation.source.display());
            } 
            // If source doesn't exist but we have a backup, restore it
            else if !operation.source.exists() {
                self.restore_backup(&operation.source)?;
                println!("Undone: Restored '{}' from backup", operation.source.display());
            }
            Ok(())
        } else {
            Err("No operations to undo".into())
        }
    }

    /// Get a list of recorded operations
    pub fn list_operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Create a backup of a file
    fn create_backup(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_directory)?;

        // Create a unique backup name with timestamp
        let filename = file_path.file_name()
            .ok_or("Invalid file path")?
            .to_string_lossy();
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("{}_{}", filename, timestamp);
        let backup_path = self.backup_directory.join(backup_name);

        // Copy the file to backup
        fs::copy(file_path, &backup_path)?;

        Ok(())
    }

    /// Restore a file from backup
    fn restore_backup(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let filename = file_path.file_name()
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
            let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
            let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
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
}
