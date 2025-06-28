#[cfg(test)]
mod file_operations_tests {
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use smv::transformers::{transform, TransformType};

    #[test]
    fn test_transform_with_extensions() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Create test files with different extensions
        let test_files = vec![
            "document.txt",
            "image.jpg", 
            "notes.md",
            "script.py",
            "data.csv",
            "no_extension",
        ];

        for file in &test_files {
            let file_path = temp_path.join(file);
            fs::write(&file_path, "test content").expect("Failed to create test file");
        }

        // Test that transform preserves file extensions correctly
        for file in &test_files {
            let transformed = transform(file, &TransformType::Snake);
            
            // Check that extension is preserved (if it exists)
            if let Some(original_ext) = Path::new(file).extension() {
                let transformed_ext = Path::new(&transformed).extension();
                assert_eq!(original_ext, transformed_ext.unwrap(), 
                    "Extension not preserved for file: {}", file);
            }
        }
    }

    #[test]
    fn test_transform_edge_cases() {
        let test_cases = vec![
            // (input, transform_type, expected_output)
            ("", TransformType::Snake, ""), // Empty string
            (".", TransformType::Snake, ""), // Just dot - gets filtered out as empty token
            (".hidden", TransformType::Snake, "hidden"), // Hidden file - no extension
            ("file.", TransformType::Snake, "file"), // Ends with dot - no extension  
            ("ALLCAPS.TXT", TransformType::Snake, "allcaps.txt"), // All caps with extension
            ("mixed.Case.File.txt", TransformType::Snake, "mixed_case_file.txt"), // Multiple dots with extension
            ("file with spaces.txt", TransformType::Snake, "file_with_spaces.txt"), // Spaces with extension
            ("file-with-dashes.txt", TransformType::Snake, "file_with_dashes.txt"), // Dashes with extension
            ("file_with_underscores.txt", TransformType::Snake, "file_with_underscores.txt"), // Already snake case
        ];

        for (input, transform_type, expected) in test_cases {
            let result = transform(input, &transform_type);
            assert_eq!(result, expected, "Transform failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_transform_unicode_handling() {
        let test_cases = vec![
            ("café.txt", TransformType::Snake, "cafe.txt"), // Accented characters
            ("naïve.txt", TransformType::Snake, "naive.txt"), // Diaeresis
            ("résumé.txt", TransformType::Snake, "resume.txt"), // Multiple accents
            ("日本語.txt", TransformType::Snake, "ri_ben_yu.txt"), // Japanese characters
            ("файл.txt", TransformType::Snake, "fail.txt"), // Cyrillic
        ];

        for (input, transform_type, expected) in test_cases {
            let result = transform(input, &transform_type);
            assert_eq!(result, expected, "Unicode transform failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_filename_conflict_detection() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Create original file
        let original_file = temp_path.join("Original File.txt");
        fs::write(&original_file, "original content").expect("Failed to create original file");

        // Create file that would conflict with transformation
        let conflicting_file = temp_path.join("original_file.txt");
        fs::write(&conflicting_file, "conflicting content").expect("Failed to create conflicting file");

        // Test that we can detect the conflict exists
        let transformed_name = transform("Original File.txt", &TransformType::Snake);
        assert_eq!(transformed_name, "original_file.txt");
        
        // Both files should exist, indicating a potential conflict
        assert!(original_file.exists());
        assert!(conflicting_file.exists());
    }

    #[test]
    fn test_extension_filtering_logic() {
        let filenames = vec![
            "document.txt",
            "image.jpg",
            "notes.md", 
            "script.py",
            "no_extension",
            ".hidden",
            "archive.tar.gz", // Multiple extensions
        ];

        // Test extension extraction logic
        for filename in &filenames {
            let path = Path::new(filename);
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                
                // Basic validation that extensions are extracted correctly
                match filename {
                    f if f.contains(".txt") => assert_eq!(ext_str, "txt"),
                    f if f.contains(".jpg") => assert_eq!(ext_str, "jpg"),
                    f if f.contains(".md") => assert_eq!(ext_str, "md"),
                    f if f.contains(".py") => assert_eq!(ext_str, "py"),
                    f if f.contains(".gz") => assert_eq!(ext_str, "gz"), // Gets last extension
                    _ => {} // No assertion for edge cases
                }
            }
        }
    }
}