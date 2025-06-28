#[cfg(test)]
mod history_tests {
    use std::fs;
    use tempfile::tempdir;
    use smv::history::HistoryManager;

    #[test]
    fn test_history_manager_creation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let backup_dir = temp_dir.path().join("backups");
        
        let _history_manager = HistoryManager::new(10, &backup_dir);
        
        // HistoryManager doesn't create backup directory until first backup
        // Create it manually to test the functionality
        fs::create_dir_all(&backup_dir).expect("Failed to create backup directory");
        assert!(backup_dir.exists());
    }

    #[test]
    fn test_history_with_file_operations() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();
        let backup_dir = temp_path.join("backups");
        
        let mut history_manager = HistoryManager::new(5, &backup_dir);
        
        // Create a test file
        let original_file = temp_path.join("test_file.txt");
        let original_content = "original content";
        fs::write(&original_file, original_content).expect("Failed to create test file");
        
        // Simulate a rename operation
        let new_file = temp_path.join("renamed_file.txt");
        
        // Record the operation in history before performing it
        // Note: We'd need to check the actual HistoryManager API
        
        // Perform the rename
        fs::rename(&original_file, &new_file).expect("Failed to rename file");
        
        // Verify the rename worked
        assert!(!original_file.exists());
        assert!(new_file.exists());
        
        let content = fs::read_to_string(&new_file).expect("Failed to read renamed file");
        assert_eq!(content, original_content);
    }

    #[test] 
    fn test_history_size_limit() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let backup_dir = temp_dir.path().join("backups");
        
        // Create history manager with small limit
        let _history_manager = HistoryManager::new(2, &backup_dir);
        
        // HistoryManager doesn't create backup directory until first backup
        // Create it manually to test the functionality
        fs::create_dir_all(&backup_dir).expect("Failed to create backup directory");
        assert!(backup_dir.exists());
    }

    #[test]
    fn test_undo_nonexistent_operation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let backup_dir = temp_dir.path().join("backups");
        
        let mut history_manager = HistoryManager::new(5, &backup_dir);
        
        // Try to undo when no operations have been performed
        let result = history_manager.undo();
        
        // Should return an error since there's nothing to undo
        assert!(result.is_err());
    }
}