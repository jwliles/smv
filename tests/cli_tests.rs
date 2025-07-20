use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn smv_cmd() -> Command {
    Command::cargo_bin("smv").unwrap()
}

#[test]
fn test_help_flag() {
    smv_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CNP Smart Move"));
}

#[test]
fn test_version_flag() {
    smv_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("smv"));
}

#[test]
fn test_prefix_removal_with_change_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("IMG_1234.jpg"), "").unwrap();
    fs::write(temp_path.join("IMG_5678.png"), "").unwrap();
    fs::write(temp_path.join("regular_file.txt"), "").unwrap();

    // Test preview mode
    smv_cmd()
        .arg("CHANGE")
        .arg("IMG_")
        .arg("INTO")
        .arg("")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("IMG_1234.jpg"))
        .stdout(predicate::str::contains("IMG_5678.png"));
}

#[test]
fn test_substring_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    fs::write(temp_path.join("old_file_name.txt"), "").unwrap();
    fs::write(temp_path.join("another_old_file.md"), "").unwrap();

    // Test preview mode for substring replacement
    smv_cmd()
        .arg("CHANGE")
        .arg("old")
        .arg("INTO")
        .arg("new")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("replace(old → new)"));
}

#[test]
fn test_snake_case_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file
    fs::write(temp_path.join("My-File Name.txt"), "").unwrap();

    // Test snake case transformation
    smv_cmd()
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("snake"));
}

#[test]
fn test_recursive_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create nested directory structure
    let sub_dir = temp_path.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("IMG_nested.jpg"), "").unwrap();

    // Test recursive processing
    smv_cmd()
        .arg("CHANGE")
        .arg("IMG_")
        .arg("INTO")
        .arg("")
        .arg(temp_path.to_str().unwrap())
        .arg("-rp")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recursive: Yes"));
}

#[test]
fn test_invalid_command() {
    smv_cmd()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_missing_arguments_for_change() {
    smv_cmd()
        .arg("CHANGE")
        .arg("prefix")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Expected 'INTO' keyword"));
}

#[test]
fn test_force_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file
    fs::write(temp_path.join("test_file.txt"), "").unwrap();

    smv_cmd()
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-F")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transform Mode"));
}

#[test]
fn test_interactive_flag() {
    smv_cmd()
        .arg("-I")
        .timeout(std::time::Duration::from_secs(1))
        .assert()
        .success();
}

#[test]
fn test_tui_flag() {
    // TUI mode should fail in non-interactive environment
    smv_cmd()
        .arg("-T")
        .timeout(std::time::Duration::from_secs(1))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to enable raw mode"));
}

#[test]
fn test_default_files_only() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files and directories
    fs::write(temp_path.join("test_file.txt"), "").unwrap();
    fs::write(temp_path.join("another_file.md"), "").unwrap();
    fs::create_dir_all(temp_path.join("test_directory")).unwrap();
    fs::create_dir_all(temp_path.join("another_directory")).unwrap();

    // Test default behavior (files only, no flag needed)
    smv_cmd()
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("snake"));
}

#[test]
fn test_everything_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files and directories
    fs::write(temp_path.join("test_file.txt"), "").unwrap();
    fs::write(temp_path.join("another_file.md"), "").unwrap();
    fs::create_dir_all(temp_path.join("test_directory")).unwrap();
    fs::create_dir_all(temp_path.join("another_directory")).unwrap();

    // Test everything flag (files and directories)
    smv_cmd()
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .arg("-e")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("snake"));
}

#[test]
fn test_everything_flag_help() {
    smv_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--everything"))
        .stdout(predicate::str::contains(
            "Process everything (files and directories)",
        ));
}

#[test]
fn test_lower_case_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with uppercase names
    fs::write(temp_path.join("UPPERCASE_FILE.TXT"), "").unwrap();
    fs::write(temp_path.join("MixedCase.MD"), "").unwrap();

    // Test lower case transformation
    smv_cmd()
        .arg("lower")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("lower"));
}

#[test]
fn test_upper_case_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with lowercase names
    fs::write(temp_path.join("lowercase_file.txt"), "").unwrap();
    fs::write(temp_path.join("mixedcase.md"), "").unwrap();

    // Test upper case transformation
    smv_cmd()
        .arg("upper")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("upper"));
}

#[test]
fn test_case_transformation_default_vs_everything() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files and directories
    fs::write(temp_path.join("UPPERCASE_FILE.TXT"), "").unwrap();
    fs::create_dir_all(temp_path.join("UPPERCASE_DIR")).unwrap();

    // Test default behavior (files only)
    smv_cmd()
        .arg("lower")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("lower"));

    // Test everything flag (files and directories)
    smv_cmd()
        .arg("lower")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .arg("-e")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("lower"));
}

#[test]
fn test_single_file_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with a specific name
    let test_file = temp_path.join("test_file_name.txt");
    fs::write(&test_file, "test content").unwrap();

    // Test single file transformation with preview
    smv_cmd()
        .arg("kebab")
        .arg("test_file_name.txt")
        .arg("-p")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("Target: test_file_name.txt"))
        .stdout(predicate::str::contains(
            "test_file_name.txt -> test-file-name.txt",
        ));

    // Test actual transformation (without preview)
    smv_cmd()
        .arg("kebab")
        .arg("test_file_name.txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transform Mode"))
        .stdout(predicate::str::contains("✓ Renamed"));

    // Verify the file was renamed correctly
    assert!(!Path::new(&temp_path.join("test_file_name.txt")).exists());
    assert!(Path::new(&temp_path.join("test-file-name.txt")).exists());
}

#[test]
fn test_single_file_transformation_no_change() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file already in snake_case
    let test_file = temp_path.join("already_snake_case.txt");
    fs::write(&test_file, "test content").unwrap();

    // Test transformation that shouldn't change anything
    smv_cmd()
        .arg("snake")
        .arg("already_snake_case.txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No change needed"));

    // Verify the file still exists with the same name
    assert!(test_file.exists());
}

// ===== Split functionality tests =====

#[test]
fn test_split_snake_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with camelCase name
    fs::write(temp_path.join("featureWishList.md"), "").unwrap();

    // Test split snake transformation
    smv_cmd()
        .arg("split")
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-snake"))
        .stdout(predicate::str::contains("featureWishList.md"));
}

#[test]
fn test_split_kebab_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with PascalCase name
    fs::write(temp_path.join("FeatureWishList.txt"), "").unwrap();

    // Test split kebab transformation
    smv_cmd()
        .arg("split")
        .arg("kebab")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-kebab"))
        .stdout(predicate::str::contains("FeatureWishList.txt"));
}

#[test]
fn test_split_title_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with camelCase name
    fs::write(temp_path.join("myFeatureList.md"), "").unwrap();

    // Test split title transformation
    smv_cmd()
        .arg("split")
        .arg("title")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-title"))
        .stdout(predicate::str::contains("myFeatureList.md"));
}

#[test]
fn test_split_camel_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with PascalCase name
    fs::write(temp_path.join("UserSettings.json"), "").unwrap();

    // Test split camel transformation
    smv_cmd()
        .arg("split")
        .arg("camel")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-camel"))
        .stdout(predicate::str::contains("UserSettings.json"));
}

#[test]
fn test_split_pascal_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with camelCase name
    fs::write(temp_path.join("userSettings.js"), "").unwrap();

    // Test split pascal transformation
    smv_cmd()
        .arg("split")
        .arg("pascal")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-pascal"))
        .stdout(predicate::str::contains("userSettings.js"));
}

#[test]
fn test_split_lower_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with mixed case
    fs::write(temp_path.join("XMLDocument.xml"), "").unwrap();

    // Test split lower transformation
    smv_cmd()
        .arg("split")
        .arg("lower")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-lower"))
        .stdout(predicate::str::contains("XMLDocument.xml"));
}

#[test]
fn test_split_upper_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with camelCase
    fs::write(temp_path.join("dataProcessor.cpp"), "").unwrap();

    // Test split upper transformation
    smv_cmd()
        .arg("split")
        .arg("upper")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-upper"))
        .stdout(predicate::str::contains("dataProcessor.cpp"));
}

#[test]
fn test_split_sentence_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with PascalCase
    fs::write(temp_path.join("HelloWorld.py"), "").unwrap();

    // Test split sentence transformation
    smv_cmd()
        .arg("split")
        .arg("sentence")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-sentence"))
        .stdout(predicate::str::contains("HelloWorld.py"));
}

#[test]
fn test_split_start_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with camelCase
    fs::write(temp_path.join("todoList.md"), "").unwrap();

    // Test split start transformation
    smv_cmd()
        .arg("split")
        .arg("start")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-start"))
        .stdout(predicate::str::contains("todoList.md"));
}

#[test]
fn test_split_studly_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file with PascalCase
    fs::write(temp_path.join("HelloWorld.rb"), "").unwrap();

    // Test split studly transformation
    smv_cmd()
        .arg("split")
        .arg("studly")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-studly"))
        .stdout(predicate::str::contains("HelloWorld.rb"));
}

#[test]
fn test_split_with_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with camelCase name
    let test_file = temp_path.join("apiEndpoint.ts");
    fs::write(&test_file, "test content").unwrap();

    // Test split snake transformation on single file
    smv_cmd()
        .arg("split")
        .arg("snake")
        .arg("apiEndpoint.ts")
        .arg("-p")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("Target: apiEndpoint.ts"));

    // Test actual transformation
    smv_cmd()
        .arg("split")
        .arg("snake")
        .arg("apiEndpoint.ts")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transform Mode"));

    // Verify the file was renamed correctly
    assert!(!Path::new(&temp_path.join("apiEndpoint.ts")).exists());
    assert!(Path::new(&temp_path.join("api_endpoint.ts")).exists());
}

#[test]
fn test_split_no_boundaries() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test file without camelCase boundaries
    fs::write(temp_path.join("lowercase.txt"), "").unwrap();

    // Test split transformation on file without boundaries (should fall back to regular transformation)
    smv_cmd()
        .arg("split")
        .arg("snake")
        .arg(temp_path.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview Mode"))
        .stdout(predicate::str::contains("split-snake"));
}

#[test]
fn test_split_invalid_command() {
    smv_cmd()
        .arg("split")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_split_missing_transformation() {
    smv_cmd()
        .arg("split")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_single_file_transformation_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Test transformation on nonexistent file (falls back to directory mode)
    smv_cmd()
        .arg("snake")
        .arg("nonexistent.txt")
        .current_dir(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No files or directories found"));
}
