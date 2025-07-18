use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use colored::*;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct FileOpConfig {
    pub recursive: bool,
    pub force: bool,
    pub no_clobber: bool,
    pub interactive: bool,
    pub preserve_metadata: bool,
    pub dereference_symlinks: bool,
    pub follow_symlinks: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FileOpStats {
    pub processed: u32,
    pub moved: u32,
    pub copied: u32,
    pub errors: u32,
    pub skipped: u32,
}

pub fn move_files(
    sources: &[PathBuf],
    destination: &Path,
    config: &FileOpConfig,
) -> Result<FileOpStats, Box<dyn Error>> {
    let mut stats = FileOpStats::default();
    let dest_is_dir = destination.is_dir();

    for source in sources {
        stats.processed += 1;

        let dest_path = if dest_is_dir {
            destination.join(source.file_name().unwrap_or_default())
        } else {
            destination.to_path_buf()
        };

        if let Err(e) = move_single_item(source, &dest_path, config) {
            eprintln!(
                "{}: Failed to move {}: {}",
                "Error".red(),
                source.display(),
                e
            );
            stats.errors += 1;
        } else {
            stats.moved += 1;
        }
    }

    Ok(stats)
}

pub fn copy_files(
    sources: &[PathBuf],
    destination: &Path,
    config: &FileOpConfig,
) -> Result<FileOpStats, Box<dyn Error>> {
    let mut stats = FileOpStats::default();
    let dest_is_dir = destination.is_dir();

    for source in sources {
        stats.processed += 1;

        let dest_path = if dest_is_dir {
            destination.join(source.file_name().unwrap_or_default())
        } else {
            destination.to_path_buf()
        };

        match copy_single_item(source, &dest_path, config) {
            Ok(item_stats) => {
                stats.copied += item_stats.copied;
                stats.processed += item_stats.processed - 1; // -1 because we already counted this in the outer loop
                stats.errors += item_stats.errors;
                stats.skipped += item_stats.skipped;
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to copy {}: {}",
                    "Error".red(),
                    source.display(),
                    e
                );
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}

fn move_single_item(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    if !source.exists() {
        return Err(format!("Source does not exist: {}", source.display()).into());
    }

    if destination.exists() && !config.force {
        if config.no_clobber {
            return Ok(());
        }

        if config.interactive && !prompt_overwrite(source, destination)? {
            return Ok(());
        }
    }

    if source.is_dir() {
        if config.recursive {
            move_directory_recursive(source, destination, config)?;
        } else {
            return Err(format!(
                "Source is a directory, use -r flag for recursive move: {}",
                source.display()
            )
            .into());
        }
    } else if source.is_file() {
        move_file(source, destination, config)?;
    } else if source.is_symlink() {
        move_symlink(source, destination, config)?;
    } else {
        return Err(format!("Unsupported file type: {}", source.display()).into());
    }

    Ok(())
}

fn copy_single_item(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<FileOpStats, Box<dyn Error>> {
    if !source.exists() {
        return Err(format!("Source does not exist: {}", source.display()).into());
    }

    if destination.exists() && !config.force {
        if config.no_clobber {
            return Ok(FileOpStats {
                processed: 1,
                skipped: 1,
                ..Default::default()
            });
        }

        if config.interactive && !prompt_overwrite(source, destination)? {
            return Ok(FileOpStats {
                processed: 1,
                skipped: 1,
                ..Default::default()
            });
        }
    }

    if source.is_dir() {
        if config.recursive {
            let recursive_stats = copy_directory_recursive(source, destination, config)?;
            return Ok(recursive_stats);
        } else {
            return Err(format!(
                "Source is a directory, use -r flag for recursive copy: {}",
                source.display()
            )
            .into());
        }
    } else if source.is_file() {
        copy_file(source, destination, config)?;
    } else if source.is_symlink() {
        copy_symlink(source, destination, config)?;
    } else {
        return Err(format!("Unsupported file type: {}", source.display()).into());
    }

    Ok(FileOpStats {
        processed: 1,
        copied: 1,
        ..Default::default()
    })
}

fn move_file(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(source, destination)?;

    if config.preserve_metadata {
        preserve_metadata(source, destination)?;
    }

    Ok(())
}

fn copy_file(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, destination)?;

    if config.preserve_metadata {
        preserve_metadata(source, destination)?;
    }

    Ok(())
}

fn move_symlink(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    if config.dereference_symlinks {
        let target = fs::read_link(source)?;
        let resolved_target = if target.is_absolute() {
            target
        } else {
            source.parent().unwrap_or(Path::new(".")).join(target)
        };

        if resolved_target.is_file() {
            move_file(&resolved_target, destination, config)?;
        } else if resolved_target.is_dir() && config.recursive {
            move_directory_recursive(&resolved_target, destination, config)?;
        }
    } else {
        let target = fs::read_link(source)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, destination)?;

        #[cfg(windows)]
        if target.is_file() {
            std::os::windows::fs::symlink_file(&target, destination)?;
        } else {
            std::os::windows::fs::symlink_dir(&target, destination)?;
        }

        fs::remove_file(source)?;
    }

    Ok(())
}

fn copy_symlink(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    if config.dereference_symlinks {
        let target = fs::read_link(source)?;
        let resolved_target = if target.is_absolute() {
            target
        } else {
            source.parent().unwrap_or(Path::new(".")).join(target)
        };

        if resolved_target.is_file() {
            copy_file(&resolved_target, destination, config)?;
        } else if resolved_target.is_dir() && config.recursive {
            copy_directory_recursive(&resolved_target, destination, config)?;
        }
    } else {
        let target = fs::read_link(source)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, destination)?;

        #[cfg(windows)]
        if target.is_file() {
            std::os::windows::fs::symlink_file(&target, destination)?;
        } else {
            std::os::windows::fs::symlink_dir(&target, destination)?;
        }
    }

    Ok(())
}

fn move_directory_recursive(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;

    for entry in WalkDir::new(source).min_depth(1).max_depth(1) {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry_path.file_name().unwrap_or_default());

        if entry_path.is_dir() {
            move_directory_recursive(entry_path, &dest_path, config)?;
        } else {
            move_single_item(entry_path, &dest_path, config)?;
        }
    }

    fs::remove_dir(source)?;

    if config.preserve_metadata {
        preserve_metadata(source, destination)?;
    }

    Ok(())
}

fn copy_directory_recursive(
    source: &Path,
    destination: &Path,
    config: &FileOpConfig,
) -> Result<FileOpStats, Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    let mut total_stats = FileOpStats::default();

    for entry in WalkDir::new(source).min_depth(1).max_depth(1) {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry_path.file_name().unwrap_or_default());

        if entry_path.is_dir() {
            let dir_stats = copy_directory_recursive(entry_path, &dest_path, config)?;
            total_stats.processed += dir_stats.processed;
            total_stats.copied += dir_stats.copied;
            total_stats.errors += dir_stats.errors;
            total_stats.skipped += dir_stats.skipped;
        } else {
            let file_stats = copy_single_item(entry_path, &dest_path, config)?;
            total_stats.processed += file_stats.processed;
            total_stats.copied += file_stats.copied;
            total_stats.errors += file_stats.errors;
            total_stats.skipped += file_stats.skipped;
        }
    }

    if config.preserve_metadata {
        preserve_metadata(source, destination)?;
    }

    // Count the directory itself
    total_stats.processed += 1;
    total_stats.copied += 1;

    Ok(total_stats)
}

fn preserve_metadata(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    let metadata = fs::metadata(source)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let permissions = std::fs::Permissions::from_mode(metadata.mode());
        fs::set_permissions(destination, permissions)?;

        if let (Ok(atime), Ok(mtime)) = (metadata.accessed(), metadata.modified()) {
            set_file_times(destination, atime, mtime)?;
        }
    }

    #[cfg(windows)]
    {
        if let (Ok(atime), Ok(mtime)) = (metadata.accessed(), metadata.modified()) {
            set_file_times(destination, atime, mtime)?;
        }
    }

    Ok(())
}

fn set_file_times(path: &Path, atime: SystemTime, mtime: SystemTime) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    {
        use std::time::UNIX_EPOCH;

        let atime_secs = atime.duration_since(UNIX_EPOCH)?.as_secs();
        let mtime_secs = mtime.duration_since(UNIX_EPOCH)?.as_secs();

        unsafe {
            let c_path = std::ffi::CString::new(path.to_str().unwrap())?;
            let times = [
                libc::timespec {
                    tv_sec: atime_secs as i64,
                    tv_nsec: 0,
                },
                libc::timespec {
                    tv_sec: mtime_secs as i64,
                    tv_nsec: 0,
                },
            ];

            if libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) != 0 {
                return Err("Failed to set file times".into());
            }
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use std::time::UNIX_EPOCH;

        let file = std::fs::OpenOptions::new().write(true).open(path)?;

        let atime_ft = systemtime_to_filetime(atime)?;
        let mtime_ft = systemtime_to_filetime(mtime)?;

        unsafe {
            use std::os::windows::io::AsRawHandle;
            use winapi::um::fileapi::SetFileTime;

            let handle = file.as_raw_handle();
            SetFileTime(handle, std::ptr::null(), &atime_ft, &mtime_ft);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn systemtime_to_filetime(
    time: SystemTime,
) -> Result<winapi::shared::minwindef::FILETIME, Box<dyn Error>> {
    use std::time::UNIX_EPOCH;
    use winapi::shared::minwindef::FILETIME;

    let duration = time.duration_since(UNIX_EPOCH)?;
    let intervals = duration.as_secs() * 10_000_000 + duration.subsec_nanos() as u64 / 100;
    let intervals = intervals + 116_444_736_000_000_000; // Windows epoch adjustment

    Ok(FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    })
}

fn prompt_overwrite(source: &Path, destination: &Path) -> Result<bool, Box<dyn Error>> {
    print!("{} overwrite '{}'? ", "mv:".yellow(), destination.display());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}

pub fn remove_files(
    targets: &[PathBuf],
    config: &FileOpConfig,
) -> Result<FileOpStats, Box<dyn Error>> {
    let mut stats = FileOpStats::default();

    for target in targets {
        stats.processed += 1;

        if let Err(e) = remove_single_item(target, config) {
            eprintln!(
                "{}: Failed to remove {}: {}",
                "Error".red(),
                target.display(),
                e
            );
            stats.errors += 1;
        } else {
            stats.moved += 1; // Use moved count for removed items
        }
    }

    Ok(stats)
}

fn remove_single_item(target: &Path, config: &FileOpConfig) -> Result<(), Box<dyn Error>> {
    if !target.exists() {
        if !config.force {
            return Err(format!("No such file or directory: {}", target.display()).into());
        }
        // With -f flag, silently ignore nonexistent files
        return Ok(());
    }

    // Interactive prompt
    if config.interactive && !prompt_remove(target)? {
        return Ok(());
    }

    if target.is_dir() {
        if config.recursive {
            remove_directory_recursive(target, config)?;
        } else {
            return Err(format!(
                "Is a directory: {} (use -r to remove directories)",
                target.display()
            )
            .into());
        }
    } else {
        fs::remove_file(target)?;
        if config.verbose {
            eprintln!("removed '{}'", target.display());
        }
    }

    Ok(())
}

fn remove_directory_recursive(target: &Path, config: &FileOpConfig) -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new(target).contents_first(true) {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            fs::remove_dir(entry_path)?;
        } else {
            fs::remove_file(entry_path)?;
        }

        if config.verbose {
            eprintln!("removed '{}'", entry_path.display());
        }
    }

    Ok(())
}

fn prompt_remove(target: &Path) -> Result<bool, Box<dyn Error>> {
    print!("remove {}? ", target.display());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase().starts_with('y'))
}

pub fn expand_glob_patterns(patterns: &[String]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut expanded = Vec::new();

    for pattern in patterns {
        let path = Path::new(pattern);

        if path.exists() {
            expanded.push(path.to_path_buf());
        } else if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            match glob::glob(pattern) {
                Ok(paths) => {
                    for path in paths {
                        match path {
                            Ok(p) => expanded.push(p),
                            Err(e) => eprintln!("{}: {}", "Warning".yellow(), e),
                        }
                    }
                }
                Err(e) => eprintln!(
                    "{}: Invalid glob pattern '{}': {}",
                    "Warning".yellow(),
                    pattern,
                    e
                ),
            }
        } else {
            return Err(format!("File or directory does not exist: {pattern}").into());
        }
    }

    if expanded.is_empty() {
        return Err("No files matched the given patterns".into());
    }

    Ok(expanded)
}

pub fn create_files(
    files: &[String],
    verbose: bool,
    access_time: Option<SystemTime>,
    modify_time: Option<SystemTime>,
) -> Result<FileOpStats, Box<dyn Error>> {
    let mut stats = FileOpStats::default();

    for file_path_str in files {
        stats.processed += 1;

        let file_path = PathBuf::from(file_path_str);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "{}: Failed to create parent directory for '{}': {}",
                        "Error".red(),
                        file_path.display(),
                        e
                    );
                    stats.errors += 1;
                    continue;
                }
            }
        }

        // Create or update the file
        let file_existed = file_path.exists();
        match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&file_path)
        {
            Ok(_) => {
                // Update timestamps if specified
                if access_time.is_some() || modify_time.is_some() {
                    let atime = access_time.unwrap_or_else(SystemTime::now);
                    let mtime = modify_time.unwrap_or_else(SystemTime::now);

                    if let Err(e) = set_file_times(&file_path, atime, mtime) {
                        eprintln!(
                            "{}: Failed to set timestamps for '{}': {}",
                            "Warning".yellow(),
                            file_path.display(),
                            e
                        );
                    }
                }

                if verbose {
                    if file_existed {
                        eprintln!("touch '{}'", file_path.display());
                    } else {
                        eprintln!("created '{}'", file_path.display());
                    }
                }
                stats.moved += 1; // Using moved count for created/touched files
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to create/touch file '{}': {}",
                    "Error".red(),
                    file_path.display(),
                    e
                );
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}

pub fn create_directories(
    directories: &[String],
    create_parents: bool,
    mode: Option<u32>,
    verbose: bool,
) -> Result<FileOpStats, Box<dyn Error>> {
    let mut stats = FileOpStats::default();

    for dir_path_str in directories {
        stats.processed += 1;

        let dir_path = PathBuf::from(dir_path_str);

        if dir_path.exists() {
            if dir_path.is_dir() {
                if verbose {
                    eprintln!("directory '{}' already exists", dir_path.display());
                }
                stats.skipped += 1;
            } else {
                eprintln!(
                    "{}: '{}' exists but is not a directory",
                    "Error".red(),
                    dir_path.display()
                );
                stats.errors += 1;
            }
            continue;
        }

        let result = if create_parents {
            fs::create_dir_all(&dir_path)
        } else {
            fs::create_dir(&dir_path)
        };

        match result {
            Ok(()) => {
                // Set permissions if mode is specified
                if let Some(mode_val) = mode {
                    if let Err(e) = set_directory_mode(&dir_path, mode_val) {
                        eprintln!(
                            "{}: Failed to set mode for '{}': {}",
                            "Warning".yellow(),
                            dir_path.display(),
                            e
                        );
                    }
                }

                if verbose {
                    eprintln!("created directory '{}'", dir_path.display());
                }
                stats.moved += 1; // Using moved count for created directories
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to create directory '{}': {}",
                    "Error".red(),
                    dir_path.display(),
                    e
                );
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}

fn set_directory_mode(path: &Path, mode: u32) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        fs::set_permissions(path, permissions)?;
    }

    #[cfg(windows)]
    {
        // Windows doesn't support Unix-style permissions
        // We could implement Windows ACL here if needed
        eprintln!(
            "{}: Mode setting not supported on Windows",
            "Warning".yellow()
        );
    }

    Ok(())
}
