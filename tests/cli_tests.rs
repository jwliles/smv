use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
        .arg("-f")
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
