//! Integration tests for transformations
//!
//! These tests verify that filename transformations work correctly.

use smv::transformers::{TransformType, transform};

#[test]
fn test_basic_transformations() {
    // Test snake_case
    assert_eq!(transform("HelloWorld", TransformType::Snake), "hello_world");
    assert_eq!(transform("My-File.txt", TransformType::Snake), "my_file.txt");
    
    // Test kebab-case
    assert_eq!(transform("HelloWorld", TransformType::Kebab), "hello-world");
    assert_eq!(transform("My_File.txt", TransformType::Kebab), "my-file.txt");
    
    // Test PascalCase
    assert_eq!(transform("hello_world", TransformType::Pascal), "HelloWorld");
    assert_eq!(transform("my-file.txt", TransformType::Pascal), "MyFileTxt");
    
    // Test camelCase
    assert_eq!(transform("hello_world", TransformType::Camel), "helloWorld");
    assert_eq!(transform("My-File.txt", TransformType::Camel), "myFileTxt");
    
    // Test clean
    assert_eq!(transform("  My File (1) !!  ", TransformType::Clean), "My File 1");
}

#[test]
fn test_title_case() {
    assert_eq!(transform("hello_world", TransformType::Title), "Hello World");
    assert_eq!(transform("my-file.txt", TransformType::Title), "My File Txt");
    assert_eq!(transform("already Title Case", TransformType::Title), "Already Title Case");
}

#[test]
fn test_uppercase_lowercase() {
    assert_eq!(transform("Hello World", TransformType::Upper), "HELLO WORLD");
    assert_eq!(transform("Hello World", TransformType::Lower), "hello world");
}