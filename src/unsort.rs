use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Moves all files from subdirectories into the root directory
pub fn flatten_directory(root: &str, dry_run: bool) -> Result<()> {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        let target = Path::new(root).join(path.file_name().unwrap());

        if path != target {
            let mut final_target = target.clone();
            if final_target.exists() {
                let timestamp = Local::now().format("%Y%m%d%H%M%S");
                let base = path.file_stem().unwrap().to_string_lossy();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                final_target = Path::new(root).join(format!("{}_{}.{}", base, timestamp, ext));
            }

            println!("Moving {} → {}", path.display(), final_target.display());
            if !dry_run {
                fs::rename(path, final_target)?;
            }
        }
    }
    Ok(())
}

/// Deletes empty directories recursively
pub fn remove_empty_dirs(root: &str, dry_run: bool) -> Result<()> {
    // Collect all directories first, then sort by depth to process deepest first
    let mut dirs = Vec::new();

    for entry in WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
    {
        dirs.push(entry.path().to_path_buf());
    }

    // Sort directories by depth (deepest first)
    dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    // Process directories from deepest to shallowest
    for path in dirs {
        if fs::read_dir(&path)?.next().is_none() && path != Path::new(root) {
            println!("Deleting empty directory: {}", path.display());
            if !dry_run {
                fs::remove_dir(&path)?;
            }
        }
    }

    Ok(())
}
