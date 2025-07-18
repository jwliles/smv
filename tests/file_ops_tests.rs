use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_basic_mv_operation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file
    let source_file = temp_path.join("test_source.txt");
    let dest_file = temp_path.join("test_dest.txt");
    fs::write(&source_file, "test content").unwrap();

    // Test mv command
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("mv")
        .arg(source_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .assert()
        .success();

    // Verify the operation
    assert!(
        !source_file.exists(),
        "Source file should not exist after mv"
    );
    assert!(dest_file.exists(), "Destination file should exist after mv");
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "test content");
}

#[test]
fn test_basic_cp_operation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file
    let source_file = temp_path.join("test_source.txt");
    let dest_file = temp_path.join("test_dest.txt");
    fs::write(&source_file, "test content").unwrap();

    // Test cp command
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("cp")
        .arg(source_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .assert()
        .success();

    // Verify the operation
    assert!(
        source_file.exists(),
        "Source file should still exist after cp"
    );
    assert!(dest_file.exists(), "Destination file should exist after cp");
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "test content");
}

#[test]
fn test_recursive_cp_operation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test directory structure
    let source_dir = temp_path.join("source_dir");
    let dest_dir = temp_path.join("dest_dir");
    fs::create_dir(&source_dir).unwrap();

    let nested_file = source_dir.join("nested.txt");
    fs::write(&nested_file, "nested content").unwrap();

    // Test recursive cp command
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("cp")
        .arg(source_dir.to_str().unwrap())
        .arg(dest_dir.to_str().unwrap())
        .arg("-r")
        .assert()
        .success();

    // Verify the operation
    assert!(
        source_dir.exists(),
        "Source directory should still exist after cp"
    );
    assert!(
        dest_dir.exists(),
        "Destination directory should exist after cp"
    );

    let copied_file = dest_dir.join("nested.txt");
    assert!(copied_file.exists(), "Nested file should be copied");
    assert_eq!(fs::read_to_string(&copied_file).unwrap(), "nested content");
}

#[test]
fn test_force_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let source_file = temp_path.join("source.txt");
    let dest_file = temp_path.join("dest.txt");
    fs::write(&source_file, "source content").unwrap();
    fs::write(&dest_file, "dest content").unwrap();

    // Test mv with force flag
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("mv")
        .arg(source_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .arg("-f")
        .assert()
        .success();

    // Verify the operation
    assert!(
        !source_file.exists(),
        "Source file should not exist after mv"
    );
    assert!(dest_file.exists(), "Destination file should exist after mv");
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "source content");
}

#[test]
fn test_no_clobber_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let source_file = temp_path.join("source.txt");
    let dest_file = temp_path.join("dest.txt");
    fs::write(&source_file, "source content").unwrap();
    fs::write(&dest_file, "dest content").unwrap();

    // Test mv with no-clobber flag
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("mv")
        .arg(source_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .arg("-n")
        .assert()
        .success();

    // Verify the operation - destination should be unchanged
    assert!(
        source_file.exists(),
        "Source file should still exist with -n flag"
    );
    assert!(dest_file.exists(), "Destination file should exist");
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "dest content");
}

#[test]
fn test_wildcard_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files with different extensions
    let txt_file = temp_path.join("test.txt");
    let log_file = temp_path.join("test.log");
    let dest_dir = temp_path.join("dest");

    fs::write(&txt_file, "txt content").unwrap();
    fs::write(&log_file, "log content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    // Test cp with wildcard pattern
    let pattern = format!("{}/*.txt", temp_path.to_str().unwrap());
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("cp")
        .arg(&pattern)
        .arg(dest_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify only .txt files were copied
    let copied_txt = dest_dir.join("test.txt");
    let copied_log = dest_dir.join("test.log");

    assert!(copied_txt.exists(), "TXT file should be copied");
    assert!(!copied_log.exists(), "LOG file should not be copied");
}

#[test]
fn test_multiple_sources() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let file1 = temp_path.join("file1.txt");
    let file2 = temp_path.join("file2.txt");
    let dest_dir = temp_path.join("dest");

    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    // Test cp with multiple sources
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("cp")
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .arg(dest_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify both files were copied
    let copied1 = dest_dir.join("file1.txt");
    let copied2 = dest_dir.join("file2.txt");

    assert!(copied1.exists(), "File1 should be copied");
    assert!(copied2.exists(), "File2 should be copied");
    assert_eq!(fs::read_to_string(&copied1).unwrap(), "content1");
    assert_eq!(fs::read_to_string(&copied2).unwrap(), "content2");
}

#[test]
fn test_preserve_metadata_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file
    let source_file = temp_path.join("source.txt");
    let dest_file = temp_path.join("dest.txt");
    fs::write(&source_file, "test content").unwrap();

    // Test cp with preserve metadata flag
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("cp")
        .arg(source_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .arg("--preserve")
        .assert()
        .success();

    // Verify the operation
    assert!(dest_file.exists(), "Destination file should exist");
    assert_eq!(fs::read_to_string(&dest_file).unwrap(), "test content");

    // Note: Testing actual metadata preservation is complex and platform-dependent
    // This test mainly verifies the flag is accepted
}

#[test]
fn test_error_handling_missing_source() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let missing_file = temp_path.join("missing.txt");
    let dest_file = temp_path.join("dest.txt");

    // Test mv with missing source file
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("mv")
        .arg(missing_file.to_str().unwrap())
        .arg(dest_file.to_str().unwrap())
        .assert()
        .failure(); // Should fail
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("smv").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Smart Move"));
}

use predicates::prelude::*;
