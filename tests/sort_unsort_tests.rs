#[cfg(test)]
mod sort_unsort_tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // Import the functions we want to test
    use smv::sort::group_by_basename;
    use smv::unsort::{flatten_directory, remove_empty_dirs};

    #[test]
    fn test_group_by_basename() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Create test files
        let test_files = vec![
            "document.txt",
            "document.pdf",
            "document.docx",
            "image.jpg",
            "image.png",
            "notes.txt",
        ];

        for file in &test_files {
            let file_path = Path::new(&temp_path).join(file);
            fs::write(&file_path, "test content").expect("Failed to create test file");
        }

        // Run the group_by_basename function with dry_run = false to perform actual changes
        group_by_basename(&temp_path, false).expect("Failed to group files");

        // Check if the directories were created
        let document_dir = Path::new(&temp_path).join("document");
        let image_dir = Path::new(&temp_path).join("image");
        let notes_dir = Path::new(&temp_path).join("notes");

        assert!(document_dir.exists() && document_dir.is_dir(), "document directory was not created");
        assert!(image_dir.exists() && image_dir.is_dir(), "image directory was not created");
        assert!(notes_dir.exists() && notes_dir.is_dir(), "notes directory was not created");

        // Check if files were moved to their respective directories
        assert!(document_dir.join("document.txt").exists(), "document.txt was not moved");
        assert!(document_dir.join("document.pdf").exists(), "document.pdf was not moved");
        assert!(document_dir.join("document.docx").exists(), "document.docx was not moved");
        assert!(image_dir.join("image.jpg").exists(), "image.jpg was not moved");
        assert!(image_dir.join("image.png").exists(), "image.png was not moved");
        assert!(notes_dir.join("notes.txt").exists(), "notes.txt was not moved");
    }

    #[test]
    fn test_flatten_directory() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Create nested directory structure
        let nested_dir1 = Path::new(&temp_path).join("nested1");
        let nested_dir2 = Path::new(&temp_path).join("nested2");
        fs::create_dir(&nested_dir1).expect("Failed to create nested directory 1");
        fs::create_dir(&nested_dir2).expect("Failed to create nested directory 2");

        // Create test files
        let test_files = vec![
            (nested_dir1.join("file1.txt"), "file1 content"),
            (nested_dir1.join("file2.pdf"), "file2 content"),
            (nested_dir2.join("file3.docx"), "file3 content"),
            (nested_dir2.join("file4.jpg"), "file4 content"),
        ];

        for (file_path, content) in &test_files {
            fs::write(file_path, content).expect("Failed to create test file");
        }

        // Run the flatten_directory function with dry_run = false to perform actual changes
        flatten_directory(&temp_path, false).expect("Failed to flatten directory");

        // Check if files were moved to the root directory
        assert!(Path::new(&temp_path).join("file1.txt").exists(), "file1.txt was not moved to root");
        assert!(Path::new(&temp_path).join("file2.pdf").exists(), "file2.pdf was not moved to root");
        assert!(Path::new(&temp_path).join("file3.docx").exists(), "file3.docx was not moved to root");
        assert!(Path::new(&temp_path).join("file4.jpg").exists(), "file4.jpg was not moved to root");

        // Check that the original files no longer exist in their original locations
        assert!(!nested_dir1.join("file1.txt").exists(), "file1.txt still exists in nested directory");
        assert!(!nested_dir1.join("file2.pdf").exists(), "file2.pdf still exists in nested directory");
        assert!(!nested_dir2.join("file3.docx").exists(), "file3.docx still exists in nested directory");
        assert!(!nested_dir2.join("file4.jpg").exists(), "file4.jpg still exists in nested directory");
    }

    #[test]
    fn test_remove_empty_dirs() {
        // Create a temporary directory for our test
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Create nested directory structure with some empty directories
        let dir1 = Path::new(&temp_path).join("dir1");
        let dir2 = Path::new(&temp_path).join("dir2");
        let dir3 = Path::new(&temp_path).join("dir3");
        let nested_dir = Path::new(&temp_path).join("dir1").join("nested");

        fs::create_dir(&dir1).expect("Failed to create dir1");
        fs::create_dir(&dir2).expect("Failed to create dir2");
        fs::create_dir(&dir3).expect("Failed to create dir3");
        fs::create_dir(&nested_dir).expect("Failed to create nested directory");

        // Add a file to dir3 so it's not empty
        fs::write(dir3.join("file.txt"), "content").expect("Failed to create test file");

        // Run the remove_empty_dirs function with dry_run = false to perform actual changes
        remove_empty_dirs(&temp_path, false).expect("Failed to remove empty directories");

        // Check that the empty directories were removed
        assert!(!dir1.exists(), "dir1 still exists");
        assert!(!dir2.exists(), "dir2 still exists");
        assert!(!nested_dir.exists(), "nested directory still exists");

        // Check that the non-empty directory still exists
        assert!(dir3.exists(), "dir3 was removed but shouldn't have been");
    }
}