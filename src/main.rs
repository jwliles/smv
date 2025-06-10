mod history;
mod repl;
mod sort;
mod transformers;
mod ui;
mod unsort;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::{ArgAction, Parser};
use colored::*;
use dirs::home_dir;
use regex::Regex;
use walkdir::WalkDir;

use history::HistoryManager;
use repl::InteractiveSession;
use transformers::{transform, TransformType};
use ui::UserInterface;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Smart Move - An enhanced mv command with transformation capabilities",
    long_about = "Command Philosophy: <tool> [scope] [targets] [modifiers]

SCOPE (choose one primary operation):
  -transform      Transform filenames (optional - inferred from transformation flags)
  -move           Move/rename files (standard mv operation) 
  -sort           Sort and organize files (with -group/-flatten/-by-type/-by-date/-by-size)
  -interactive    Launch interactive REPL interface
  -tui            Launch terminal UI file explorer
  -undo           Undo the last operation

EXAMPLES:
  smv --pascal . md --preview
  smv --snake . txt pdf
  smv --sort --group downloads/ --preview
  smv --move backup/ . log
  smv --interactive
  smv --tui"
)]
struct Args {
    // === SCOPE/MODE (primary operation) ===
    /// [SCOPE] Transform filenames using specified transformation type
    #[arg(long, action = ArgAction::SetTrue, group = "scope")]
    transform: bool,

    /// [SCOPE] Move/rename files (standard mv operation)
    #[arg(long, action = ArgAction::SetTrue, group = "scope")]
    move_files: bool,

    /// [SCOPE] Sort and organize files by type, date, or other criteria
    #[arg(long, action = ArgAction::SetTrue, group = "scope")]
    sort: bool,

    /// [SCOPE] Interactive mode - launch REPL interface
    #[arg(short, long, action = ArgAction::SetTrue, group = "scope")]
    interactive: bool,

    /// [SCOPE] Terminal UI mode - launch the TUI file explorer
    #[arg(short = 'T', long = "tui", action = ArgAction::SetTrue, group = "scope")]
    tui: bool,

    /// [SCOPE] Undo the last operation
    #[arg(long, action = ArgAction::SetTrue, group = "scope")]
    undo: bool,

    // === TRANSFORMATION TYPES (choose one - automatically enables transform mode) ===
    /// [TRANSFORM] Convert to snake_case
    #[arg(long, action = ArgAction::SetTrue)]
    snake: bool,

    /// [TRANSFORM] Convert to kebab-case
    #[arg(long, action = ArgAction::SetTrue)]
    kebab: bool,

    /// [TRANSFORM] Convert to PascalCase
    #[arg(long, action = ArgAction::SetTrue)]
    pascal: bool,

    /// [TRANSFORM] Convert to camelCase
    #[arg(long, action = ArgAction::SetTrue)]
    camel: bool,

    /// [TRANSFORM] Convert to Title Case
    #[arg(long, action = ArgAction::SetTrue)]
    title: bool,

    /// [TRANSFORM] Convert to lowercase
    #[arg(long, action = ArgAction::SetTrue)]
    lower: bool,

    /// [TRANSFORM] Convert to UPPERCASE
    #[arg(long, action = ArgAction::SetTrue)]
    upper: bool,

    /// [TRANSFORM] Clean filenames (remove special chars)
    #[arg(long, action = ArgAction::SetTrue)]
    clean: bool,

    // === SORT CRITERIA (for --sort scope) ===
    /// [SORT OPTION] Sort by file type (extension)
    #[arg(long, action = ArgAction::SetTrue, requires = "sort")]
    by_type: bool,

    /// [SORT OPTION] Sort by modification date
    #[arg(long, action = ArgAction::SetTrue, requires = "sort")]
    by_date: bool,

    /// [SORT OPTION] Sort by file size
    #[arg(long, action = ArgAction::SetTrue, requires = "sort")]
    by_size: bool,

    /// [SORT OPTION] Group files by basename into directories
    #[arg(long, action = ArgAction::SetTrue, requires = "sort")]
    group: bool,

    /// [SORT OPTION] Flatten directory structure
    #[arg(long, action = ArgAction::SetTrue, requires = "sort")]
    flatten: bool,

    // === MODIFIERS ===
    /// [MODIFIER] Preview changes without applying them
    #[arg(short, long, action = ArgAction::SetTrue)]
    preview: bool,

    /// [MODIFIER] Process subdirectories recursively
    #[arg(short, long, action = ArgAction::SetTrue)]
    recursive: bool,

    /// [MODIFIER] Force operation without confirmation
    #[arg(short, long, action = ArgAction::SetTrue)]
    force: bool,

    /// [MODIFIER] Comma-separated patterns to exclude (e.g., "*.tmp,test_*")
    #[arg(long, value_name = "PATTERNS")]
    exclude: Option<String>,

    /// Maximum number of operations to keep in history
    #[arg(long, value_name = "SIZE", default_value = "50")]
    max_history_size: usize,

    // === TARGETS ===
    /// [TARGET] Source directory (use '.' for current directory)
    #[arg(value_name = "DIRECTORY")]
    directory: Option<String>,

    /// [TARGET] File extensions to process (e.g., 'pdf', 'txt', 'jpg')
    #[arg(value_name = "EXTENSIONS")]
    extensions: Vec<String>,

    // === LEGACY SUPPORT ===
    /// Destination for move operations (when using legacy syntax)
    #[arg(long, value_name = "DESTINATION")]
    destination: Option<String>,
}

#[derive(Debug, Default)]
struct Stats {
    processed: u32,
    renamed: u32,
    errors: u32,
    skipped: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Handle scope/mode operations
    match determine_operation_mode(&args) {
        OperationMode::Transform => run_transform_mode(&args)?,
        OperationMode::Move => run_move_mode(&args)?,
        OperationMode::Sort => run_sort_mode(&args)?,
        OperationMode::Interactive => run_interactive_mode(args.max_history_size)?,
        OperationMode::Tui => run_tui_mode()?,
        OperationMode::Undo => run_undo_mode(args.max_history_size)?,
        OperationMode::None => {
            eprintln!("Error: No operation mode specified.");
            eprintln!("Use one of: --transform, --move, --sort, --interactive, --tui, or --undo");
            eprintln!("For help: smv --help");
            process::exit(1);
        }
    }

    Ok(())
}

#[derive(Debug)]
enum OperationMode {
    Transform,
    Move,
    Sort,
    Interactive,
    Tui,
    Undo,
    None,
}

fn determine_operation_mode(args: &Args) -> OperationMode {
    // Check for explicit mode flags first
    if args.transform { OperationMode::Transform }
    else if args.move_files { OperationMode::Move }
    else if args.sort { OperationMode::Sort }
    else if args.interactive { OperationMode::Interactive }
    else if args.tui { OperationMode::Tui }
    else if args.undo { OperationMode::Undo }
    // Infer transform mode if any transformation flag is used
    else if args.snake || args.kebab || args.pascal || args.camel || 
            args.title || args.lower || args.upper || args.clean {
        OperationMode::Transform
    }
    else { OperationMode::None }
}

/// Runs the Text-based User Interface (TUI) mode of the application.
fn run_tui_mode() -> Result<(), Box<dyn Error>> {
    // Setup backup directory
    let backup_dir = home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("smv")
        .join("backups");

    // Ensure backup directory exists
    fs::create_dir_all(&backup_dir)?;

    // Create and run TUI application
    let mut app = ui::terminal::App::new()?;
    app.run()?;

    Ok(())
}

/// Launch the interactive REPL session
fn run_interactive_mode(max_history_size: usize) -> Result<(), Box<dyn Error>> {
    // Setup backup directory
    let backup_dir = home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("smv")
        .join("backups");

    // Ensure backup directory exists
    fs::create_dir_all(&backup_dir)?;

    // Create and run interactive session
    let mut session = InteractiveSession::new(max_history_size, &backup_dir)?;
    session.run()?;

    Ok(())
}

/// Undo the last operation
fn run_undo_mode(max_history_size: usize) -> Result<(), Box<dyn Error>> {
    // Setup backup directory
    let backup_dir = home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("smv")
        .join("backups");

    // Ensure backup directory exists
    fs::create_dir_all(&backup_dir)?;

    // Create history manager
    let mut history_manager = HistoryManager::new(max_history_size, &backup_dir);

    // Attempt to undo the last operation
    match history_manager.undo() {
        Ok(_) => {
            println!("Operation undone successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
            Err(e)
        }
    }
}

/// Run transform mode using the new command structure
fn run_transform_mode(args: &Args) -> Result<(), Box<dyn Error>> {
    // Determine which transformation type was specified
    let transform_type = if args.snake {
        TransformType::Snake
    } else if args.kebab {
        TransformType::Kebab
    } else if args.pascal {
        TransformType::Pascal
    } else if args.camel {
        TransformType::Camel
    } else if args.title {
        TransformType::Title
    } else if args.lower {
        TransformType::Lower
    } else if args.upper {
        TransformType::Upper
    } else if args.clean {
        TransformType::Clean
    } else {
        return Err("Transform type is required. Use one of: --snake, --kebab, --pascal, --camel, --title, --lower, --upper, --clean".into());
    };

    // Get directory (default to current directory)
    let directory = args.directory.as_deref().unwrap_or(".");
    
    // Validate directory exists
    if !std::path::Path::new(directory).exists() {
        return Err(format!("Directory does not exist: {}", directory).into());
    }

    // Get extensions (if specified)
    let extensions = if args.extensions.is_empty() {
        None
    } else {
        Some(args.extensions.clone())
    };

    // Process exclude patterns
    let exclude_patterns: Vec<regex::Regex> = process_exclude_patterns(args.exclude.as_deref())?;

    // Print operation mode
    println!("\n{}", format!("Smart Move - {} Mode", 
        if args.preview { "Preview" } else { "Transform" }).bold());
    println!("Transformation: {}", transform_type.as_str().green());
    println!("Directory: {}", directory.cyan());
    println!("Extensions: {}", match &extensions {
        Some(exts) => exts.join(", ").cyan(),
        None => "All files".yellow(),
    });
    println!("Recursive: {}", if args.recursive { "Yes".green() } else { "No".yellow() });
    println!();

    // Build file list based on directory and extensions
    let files = build_file_list(directory, &extensions, args.recursive, &exclude_patterns)?;
    
    if files.is_empty() {
        println!("No files or directories found matching criteria.");
        return Ok(());
    }

    // Process files and directories for transformation
    let mut stats = Stats::default();
    for item_path in files {
        process_item_transformation(&item_path, transform_type, args.preview, &mut stats)?;
    }

    // Print results
    print_transformation_results(&stats, args.preview);

    Ok(())
}

/// Build list of files and directories to process based on directory and extensions
fn build_file_list(
    directory: &str,
    extensions: &Option<Vec<String>>,
    recursive: bool,
    exclude_patterns: &[regex::Regex],
) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
    use walkdir::WalkDir;
    
    let mut items = Vec::new();
    let walker = if recursive {
        WalkDir::new(directory)
    } else {
        WalkDir::new(directory).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(Result::ok) {
        let path = entry.path();
        
        // Skip the root directory itself to avoid self-transformation
        if path == std::path::Path::new(directory) {
            continue;
        }

        // For extension filtering, only apply to files (directories don't have extensions)
        if path.is_file() {
            // Check extensions if specified
            if let Some(target_extensions) = extensions {
                if let Some(file_ext) = path.extension() {
                    let file_ext_str = file_ext.to_string_lossy().to_lowercase();
                    if !target_extensions.iter().any(|ext| ext.to_lowercase() == file_ext_str) {
                        continue;
                    }
                } else {
                    // File has no extension, skip if extensions were specified
                    continue;
                }
            }
        }
        // For directories, we always include them regardless of extension filters
        // since directories don't have extensions

        // Check exclude patterns
        let path_str = path.to_string_lossy();
        if exclude_patterns.iter().any(|pattern| pattern.is_match(&path_str)) {
            continue;
        }

        items.push(path.to_path_buf());
    }

    Ok(items)
}

/// Process a single file or directory for transformation
fn process_item_transformation(
    item_path: &std::path::Path,
    transform_type: TransformType,
    preview_only: bool,
    stats: &mut Stats,
) -> Result<(), Box<dyn Error>> {
    let item_name = item_path.file_name()
        .ok_or("Invalid item name")?
        .to_string_lossy();

    let new_name = transform(&item_name, transform_type);

    stats.processed += 1;

    // If name unchanged, nothing to do
    if new_name == item_name {
        return Ok(());
    }

    let new_path = item_path.parent()
        .ok_or("Invalid parent directory")?
        .join(&new_name);

    // Check for conflicts
    if new_path.exists() && item_path != new_path {
        let item_type = if item_path.is_dir() { "directory" } else { "file" };
        println!("{}Conflict: {} \"{}\" → \"{}\" (target exists)", 
            if preview_only { "[PREVIEW] " } else { "" },
            item_type, item_name, new_name);
        stats.errors += 1;
        return Ok(());
    }

    // Log the operation
    let item_type = if item_path.is_dir() { "directory" } else { "file" };
    println!("{}Rename {}: \"{}\" → \"{}\"",
        if preview_only { "[PREVIEW] " } else { "" },
        item_type, item_name, new_name);

    if !preview_only {
        std::fs::rename(item_path, &new_path)?;
    }

    stats.renamed += 1;
    Ok(())
}

/// Print transformation results
fn print_transformation_results(stats: &Stats, preview_only: bool) {
    println!("\n{}:", "Results".bold());
    println!("Items processed: {}", stats.processed.to_string().cyan());
    println!("Items to be renamed: {}", stats.renamed.to_string().green());
    println!("Errors encountered: {}", stats.errors.to_string().red());

    if preview_only && stats.renamed > 0 {
        println!("\n{}", "This was a preview only. No items were actually renamed.".bold().blue());
        println!("{}", "To apply these changes, run the same command without --preview.".blue());
    }
}

/// Run move mode (replaces old mv functionality)
fn run_move_mode(_args: &Args) -> Result<(), Box<dyn Error>> {
    // For now, maintain compatibility with legacy move operations
    eprintln!("Move mode not yet implemented in new structure.");
    eprintln!("Use legacy syntax for now or use --interactive mode.");
    Ok(())
}

/// Run sort mode (replaces old group/flatten functionality)  
fn run_sort_mode(args: &Args) -> Result<(), Box<dyn Error>> {
    let directory = args.directory.as_deref().unwrap_or(".");
    
    if args.group {
        println!("\n{}\n", "Smart Move - Group Files by Basename".bold());
        println!("Processing directory: {}", directory.cyan());
        sort::group_by_basename(directory, args.preview)?;
    } else if args.flatten {
        println!("\n{}\n", "Smart Move - Flatten Directory Structure".bold());
        println!("Processing directory: {}", directory.cyan());
        unsort::flatten_directory(directory, args.preview)?;
        
        // Also remove empty directories
        println!("\nRemoving empty directories:");
        unsort::remove_empty_dirs(directory, args.preview)?;
    } else if args.by_type {
        println!("Sort by type not yet implemented.");
    } else if args.by_date {
        println!("Sort by date not yet implemented.");
    } else if args.by_size {
        println!("Sort by size not yet implemented.");
    } else {
        eprintln!("Error: Sort mode requires a sort criteria.");
        eprintln!("Use: --group, --flatten, --by-type, --by-date, or --by-size");
        process::exit(1);
    }

    if args.preview {
        println!("\n{}", "This was a preview only. No files were actually moved.".bold().blue());
        println!("{}", "To apply these changes, run the same command without --preview.".blue());
    }

    Ok(())
}

/// Process exclude patterns into Regex objects
fn process_exclude_patterns(patterns: Option<&str>) -> Result<Vec<regex::Regex>, Box<dyn Error>> {
    match patterns {
        Some(patterns) => {
            let result: Vec<regex::Regex> = patterns
                .split(',')
                .filter_map(|p| {
                    let p = p.trim();
                    if p.is_empty() {
                        None
                    } else {
                        match regex::Regex::new(p) {
                            Ok(re) => Some(re),
                            Err(e) => {
                                eprintln!("{}: {}", "Invalid regex pattern".red(), e);
                                None
                            }
                        }
                    }
                })
                .collect();
            Ok(result)
        }
        None => Ok(Vec::new()),
    }
}