use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Groups files in a directory by their base name (ignores extension) and moves them into folders.
pub fn group_by_basename(dir: &str, dry_run: bool) -> Result<()> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
            groups.entry(file_name).or_default().push(path);
        }
    }

    for (base, files) in groups {
        let target_dir = Path::new(dir).join(&base);
        if !target_dir.exists() && !dry_run {
            fs::create_dir(&target_dir)?;
            println!("Created directory: {}", target_dir.display());
        }

        for file in files {
            let new_path = target_dir.join(file.file_name().unwrap());
            println!("Moving {} → {}", file.display(), new_path.display());
            if !dry_run {
                fs::rename(&file, &new_path)?;
            }
        }
    }

    Ok(())
}
