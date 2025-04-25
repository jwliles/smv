//! Core functionality for the SMV application
//!
//! This module contains core logic for file operations, path handling,
//! and other central functionality.

use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use regex::Regex;
use walkdir::WalkDir;
use colored::*;
use dirs::home_dir;

use crate::transformers::{TransformType, transform};
use crate::history::HistoryManager;

/// Statistics for file operations
#[derive(Debug, Default)]
pub struct Stats {
    pub processed: u32,
    pub renamed: u32,
    pub errors: u32,
    pub skipped: u32,
}

/// Process exclude patterns into Regex objects
pub fn process_exclude_patterns(patterns: Option<&str>) -> Result<Vec<Regex>, Box<dyn Error>> {
    match patterns {
        Some(patterns) => {
            let mut result = Vec::new();
            for p in patterns.split(',') {
                let p = p.trim();
                if !p.is_empty() {
                    match Regex::new(p) {
                        Ok(re) => result.push(re),
                        Err(e) => {
                            eprintln!("{}: {}", "Invalid regex pattern".red(), e);
                        }
                    }
                }
            }
            Ok(result)
        },
        None => Ok(Vec::new()),
    }
}

/// Process a single glob pattern for transformation
pub fn process_pattern(
    pattern: &str,
    transform_type: TransformType,
    preview_only: bool,
    recursive: bool,
    exclude_patterns: &[Regex],
    extensions: &Option<Vec<String>>,
    stats: &mut Stats,
) -> Result<(), Box<dyn Error>> {
    // Get the directory part of the pattern
    let path = Path::new(pattern);
    let base_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // Process files based on whether we're doing recursive traversal or not
    if recursive {
        // Use WalkDir for recursive traversal
        for entry in WalkDir::new(&base_dir) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    // Skip directories
                    if path.is_dir() {
                        continue;
                    }
                    
                    // Check if path matches the pattern
                    if !path_matches_pattern(path, pattern) {
                        continue;
                    }
                    
                    // Process the file
                    process_file(
                        path, 
                        transform_type, 
                        preview_only, 
                        exclude_patterns, 
                        extensions, 
                        stats
                    )?;
                },
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    stats.errors += 1;
                }
            }
        }
    } else {
        // Just process the current directory
        match fs::read_dir(&base_dir) {
            Ok(dir_entries) => {
                for entry in dir_entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            
                            // Skip directories
                            if path.is_dir() {
                                continue;
                            }
                            
                            // Check if path matches the pattern
                            if !path_matches_pattern(&path, pattern) {
                                continue;
                            }
                            
                            // Process the file
                            process_file(
                                &path, 
                                transform_type, 
                                preview_only, 
                                exclude_patterns, 
                                extensions, 
                                stats
                            )?;
                        },
                        Err(e) => {
                            eprintln!("Error reading directory entry: {}", e);
                            stats.errors += 1;
                        }
                    }
                }
            },
            Err(e) => {
                return Err(format!("Failed to read directory {}: {}", base_dir.display(), e).into());
            }
        }
    }

    Ok(())
}

/// Check if a path matches a glob pattern
pub fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
    // If the pattern is a directory, any file in it matches
    if Path::new(pattern).is_dir() {
        return true;
    }
    
    // Otherwise use simple string matching for now
    // This could be improved with proper glob matching
    let path_str = path.to_string_lossy();
    if pattern.contains('*') || pattern.contains('?') {
        // Very simple wildcard matching
        let pattern_regex = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        
        Regex::new(&format!("^{}$", pattern_regex))
            .map(|re| re.is_match(&path_str))
            .unwrap_or(false)
    } else {
        // Exact match
        path_str == pattern
    }
}

/// Process a single file for transformation
pub fn process_file(
    file_path: &Path,
    transform_type: TransformType,
    preview_only: bool,
    exclude_patterns: &[Regex],
    extensions: &Option<Vec<String>>,
    stats: &mut Stats,
) -> Result<(), Box<dyn Error>> {
    let file_path_str = file_path.to_string_lossy();

    // Skip if the file matches an exclude pattern
    if exclude_patterns.iter().any(|pattern| pattern.is_match(&file_path_str)) {
        stats.skipped += 1;
        return Ok(());
    }

    // Skip if we're filtering by extension and this file doesn't match
    if let Some(exts) = extensions {
        if let Some(ext) = file_path.extension() {
            let file_ext = ext.to_string_lossy().to_lowercase();
            if !exts.contains(&file_ext) {
                stats.skipped += 1;
                return Ok(());
            }
        } else {
            // No extension
            stats.skipped += 1;
            return Ok(());
        }
    }

    // Get filename and directory
    let Some(filename) = file_path.file_name().map(|f| f.to_string_lossy().to_string()) else {
        stats.errors += 1;
        return Ok(());
    };
    let Some(directory) = file_path.parent() else {
        stats.errors += 1;
        return Ok(());
    };

    // Apply the transformation
    let new_name = transform(&filename, transform_type);

    // If the name didn't change, we're done
    if new_name == filename {
        stats.processed += 1;
        return Ok(());
    }

    let new_path = directory.join(&new_name);

    // Check if the new name would conflict with an existing file
    if new_path.exists() && file_path != &new_path {
        println!("{}: Cannot rename \"{}\" to \"{}\" - file already exists", 
            "Error".red(), file_path_str, new_path.to_string_lossy());
        stats.errors += 1;
        return Ok(());
    }

    // Log the rename operation
    println!("{}{}\"{}\" → \"{}\"",
        if preview_only { "[PREVIEW] ".blue() } else { "".into() },
        "Rename: ".green(),
        filename,
        new_name
    );

    // Perform the rename if not in preview mode
    if !preview_only {
        // Create backup directory for the history manager
        let backup_dir = home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config")
            .join("smv")
            .join("backups");
        
        // Ensure backup directory exists
        fs::create_dir_all(&backup_dir)?;
        
        // Create history manager and record operation
        let mut history_manager = HistoryManager::new(50, &backup_dir);
        history_manager.record(file_path.to_path_buf(), new_path.clone())?;
        
        match fs::rename(file_path, &new_path) {
            Ok(_) => stats.renamed += 1,
            Err(e) => {
                println!("{}: Renaming \"{}\": {}", "Error".red(), file_path_str, e);
                stats.errors += 1;
            }
        }
    } else {
        stats.renamed += 1;
    }

    stats.processed += 1;
    Ok(())
}

// These imports are already at the top of the file