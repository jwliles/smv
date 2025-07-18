use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use colored::*;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileOpConfig {
    pub recursive: bool,
    pub force: bool,
    pub no_clobber: bool,
    pub interactive: bool,
    pub preserve_metadata: bool,
    pub dereference_symlinks: bool,
    pub follow_symlinks: bool,
}

impl Default for FileOpConfig {
    fn default() -> Self {
        Self {
            recursive: false,
            force: false,
            no_clobber: false,
            interactive: false,
            preserve_metadata: false,
            dereference_symlinks: false,
            follow_symlinks: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileOpStats {
    pub processed: u32,
    pub moved: u32,
    pub copied: u32,
    pub errors: u32,
    pub skipped: u32,
}

impl Default for FileOpStats {
    fn default() -> Self {
        Self {
            processed: 0,
            moved: 0,
            copied: 0,
            errors: 0,
            skipped: 0,
        }
    }
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

        if let Err(e) = copy_single_item(source, &dest_path, config) {
            eprintln!(
                "{}: Failed to copy {}: {}",
                "Error".red(),
                source.display(),
                e
            );
            stats.errors += 1;
        } else {
            stats.copied += 1;
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

        if config.interactive {
            if !prompt_overwrite(source, destination)? {
                return Ok(());
            }
        }
    }

    if source.is_dir() {
        if config.recursive {
            move_directory_recursive(source, destination, config)?;
        } else {
            return Err(format!("Source is a directory, use -r flag for recursive move: {}", source.display()).into());
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
) -> Result<(), Box<dyn Error>> {
    if !source.exists() {
        return Err(format!("Source does not exist: {}", source.display()).into());
    }

    if destination.exists() && !config.force {
        if config.no_clobber {
            return Ok(());
        }

        if config.interactive {
            if !prompt_overwrite(source, destination)? {
                return Ok(());
            }
        }
    }

    if source.is_dir() {
        if config.recursive {
            copy_directory_recursive(source, destination, config)?;
        } else {
            return Err(format!("Source is a directory, use -r flag for recursive copy: {}", source.display()).into());
        }
    } else if source.is_file() {
        copy_file(source, destination, config)?;
    } else if source.is_symlink() {
        copy_symlink(source, destination, config)?;
    } else {
        return Err(format!("Unsupported file type: {}", source.display()).into());
    }

    Ok(())
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
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;

    for entry in WalkDir::new(source).min_depth(1).max_depth(1) {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry_path.file_name().unwrap_or_default());

        if entry_path.is_dir() {
            copy_directory_recursive(entry_path, &dest_path, config)?;
        } else {
            copy_single_item(entry_path, &dest_path, config)?;
        }
    }

    if config.preserve_metadata {
        preserve_metadata(source, destination)?;
    }

    Ok(())
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
        use std::os::unix::fs::MetadataExt;
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
            return Err(format!("File or directory does not exist: {}", pattern).into());
        }
    }

    if expanded.is_empty() {
        return Err("No files matched the given patterns".into());
    }

    Ok(expanded)
}
