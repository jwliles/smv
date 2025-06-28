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
    about = "Smart Move - Enhanced file operations with XFD syntax",
    long_about = "CNP Smart Move Tool - XFD (eXtended Flag Definition) Syntax

SYNTAX:
  smv [COMMAND] [OPTIONS] [TARGET] [FLAGS]
  smv CHANGE \"old\" INTO \"new\" [target] [flags]
  smv [transform] [target] [flags]
  smv [operation] [target] [flags]

COMMANDS:
  CHANGE \"old\" INTO \"new\"  Replace substring in filenames
  REGEX \"pattern\" INTO \"replacement\"  Replace using regex pattern
  snake, kebab, pascal, camel, title, lower, upper, clean  Transform filename case/format
  sort, group, flatten  Organize files
  interactive, tui, undo  Special modes

FLAGS (stackable):
  -r  Recursive (process subdirectories)
  -p  Preview (show changes without applying)
  -f  Force (skip confirmations)
  -i  Interactive mode
  -T  Terminal UI mode
  -u  Undo last operation

EXAMPLES:
  smv CHANGE \"AFN\" INTO \"CNP\" . -rp
  smv REGEX \"\\\\d+\" INTO \"XXX\" . -r
  smv pascal . -p
  smv sort . -rf
  smv -i"
)]
struct Args {
    // === XFD COMMAND SYNTAX ===
    /// Main command or transformation type
    #[arg(value_name = "COMMAND")]
    command: Option<String>,

    /// First argument (for CHANGE command: the old string)
    #[arg(value_name = "ARG1")]
    arg1: Option<String>,

    /// INTO keyword (for CHANGE/REGEX commands)
    #[arg(value_name = "INTO")]
    into_keyword: Option<String>,

    /// Second argument (for CHANGE command: the new string)
    #[arg(value_name = "ARG2")]
    arg2: Option<String>,

    /// Target directory or file pattern
    #[arg(value_name = "TARGET")]
    target: Option<String>,

    /// Additional file extensions or patterns
    #[arg(value_name = "EXTENSIONS")]
    extensions: Vec<String>,

    // === XFD FLAGS (single character, stackable) ===
    /// Stackable flags: r(ecursive), p(review), f(orce), i(nteractive), T(ui), u(ndo)
    #[arg(short = 'r', action = ArgAction::SetTrue, help = "Recursive - process subdirectories")]
    recursive: bool,

    #[arg(short = 'p', action = ArgAction::SetTrue, help = "Preview - show changes without applying")]
    preview: bool,

    #[arg(short = 'f', action = ArgAction::SetTrue, help = "Force - skip confirmations")]
    force: bool,

    #[arg(short = 'i', action = ArgAction::SetTrue, help = "Interactive - launch REPL interface")]
    interactive: bool,

    #[arg(short = 'T', action = ArgAction::SetTrue, help = "TUI - launch terminal UI file explorer")]
    tui: bool,

    #[arg(short = 'u', action = ArgAction::SetTrue, help = "Undo - reverse last operation")]
    undo: bool,

    // === LEGACY SUPPORT ===
    /// Comma-separated patterns to exclude (e.g., "*.tmp,test_*")
    #[arg(long, value_name = "PATTERNS")]
    exclude: Option<String>,

    /// Maximum number of operations to keep in history
    #[arg(long, value_name = "SIZE", default_value = "50")]
    max_history_size: usize,
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

    // Parse XFD command
    let command = match parse_xfd_command(&args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("For help: smv --help");
            process::exit(1);
        }
    };

    // Execute command
    match command {
        XfdCommand::Change { old, new } => {
            let transform_type = TransformType::replace(&old, &new);
            run_transform_command(&args, transform_type)?
        },
        XfdCommand::Regex { pattern, replacement } => {
            let transform_type = TransformType::replace_regex(&pattern, &replacement);
            run_transform_command(&args, transform_type)?
        },
        XfdCommand::Transform(transform_type) => {
            run_transform_command(&args, transform_type)?
        },
        XfdCommand::Sort { method } => {
            run_sort_command(&args, method)?
        },
        XfdCommand::Interactive => run_interactive_mode(args.max_history_size)?,
        XfdCommand::Tui => run_tui_mode()?,
        XfdCommand::Undo => run_undo_mode(args.max_history_size)?,
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum XfdCommand {
    Change { old: String, new: String },
    Regex { pattern: String, replacement: String },
    Transform(TransformType),
    Sort { method: SortMethod },
    Interactive,
    Tui,
    Undo,
}

#[derive(Debug, Clone)]
enum SortMethod {
    Group,
    Flatten,
    ByType,
    ByDate,
    BySize,
}

fn parse_xfd_command(args: &Args) -> Result<XfdCommand, Box<dyn Error>> {
    // Check for flags first (highest priority)
    if args.interactive {
        return Ok(XfdCommand::Interactive);
    }
    if args.tui {
        return Ok(XfdCommand::Tui);
    }
    if args.undo {
        return Ok(XfdCommand::Undo);
    }

    // Parse command structure
    match args.command.as_deref() {
        Some("CHANGE") => {
            let old = args.arg1.as_ref().ok_or("Missing old string for CHANGE command")?;
            if args.into_keyword.as_deref() != Some("INTO") {
                return Err("Expected 'INTO' keyword after old string".into());
            }
            let new = args.arg2.as_ref().ok_or("Missing new string after INTO keyword")?;
            Ok(XfdCommand::Change { old: old.clone(), new: new.clone() })
        },
        Some("REGEX") => {
            let pattern = args.arg1.as_ref().ok_or("Missing pattern for REGEX command")?;
            if args.into_keyword.as_deref() != Some("INTO") {
                return Err("Expected 'INTO' keyword after pattern".into());
            }
            let replacement = args.arg2.as_ref().ok_or("Missing replacement after INTO keyword")?;
            Ok(XfdCommand::Regex { pattern: pattern.clone(), replacement: replacement.clone() })
        },
        Some("snake") => Ok(XfdCommand::Transform(TransformType::Snake)),
        Some("kebab") => Ok(XfdCommand::Transform(TransformType::Kebab)),
        Some("pascal") => Ok(XfdCommand::Transform(TransformType::Pascal)),
        Some("camel") => Ok(XfdCommand::Transform(TransformType::Camel)),
        Some("title") => Ok(XfdCommand::Transform(TransformType::Title)),
        Some("lower") => Ok(XfdCommand::Transform(TransformType::Lower)),
        Some("upper") => Ok(XfdCommand::Transform(TransformType::Upper)),
        Some("clean") => Ok(XfdCommand::Transform(TransformType::Clean)),
        Some("sort") => Ok(XfdCommand::Sort { method: SortMethod::Group }), // Default sort method
        Some("group") => Ok(XfdCommand::Sort { method: SortMethod::Group }),
        Some("flatten") => Ok(XfdCommand::Sort { method: SortMethod::Flatten }),
        Some("interactive") => Ok(XfdCommand::Interactive),
        Some("tui") => Ok(XfdCommand::Tui),
        Some("undo") => Ok(XfdCommand::Undo),
        Some(unknown) => Err(format!("Unknown command: {}", unknown).into()),
        None => Err("No command specified. Use: CHANGE \"old\" INTO \"new\", transform commands, or flags".into()),
    }
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

/// Run transform command using XFD syntax
fn run_transform_command(args: &Args, transform_type: TransformType) -> Result<(), Box<dyn Error>> {

    // Get target directory (default to current directory)
    let directory = args.target.as_deref().unwrap_or(".");
    
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
    println!("\n{}", format!("CNP Smart Move - {} Mode", 
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
        process_item_transformation(&item_path, &transform_type, args.preview, &mut stats)?;
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
    transform_type: &TransformType,
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


/// Run sort command using XFD syntax
fn run_sort_command(args: &Args, method: SortMethod) -> Result<(), Box<dyn Error>> {
    let directory = args.target.as_deref().unwrap_or(".");
    
    match method {
        SortMethod::Group => {
            println!("\n{}\n", "CNP Smart Move - Group Files by Basename".bold());
            println!("Processing directory: {}", directory.cyan());
            sort::group_by_basename(directory, args.preview)?
        },
        SortMethod::Flatten => {
            println!("\n{}\n", "CNP Smart Move - Flatten Directory Structure".bold());
            println!("Processing directory: {}", directory.cyan());
            unsort::flatten_directory(directory, args.preview)?;
            
            // Also remove empty directories
            println!("\nRemoving empty directories:");
            unsort::remove_empty_dirs(directory, args.preview)?
        },
        SortMethod::ByType => {
            println!("Sort by type not yet implemented.");
        },
        SortMethod::ByDate => {
            println!("Sort by date not yet implemented.");
        },
        SortMethod::BySize => {
            println!("Sort by size not yet implemented.");
        },
    }

    if args.preview {
        println!("\n{}", "This was a preview only. No files were actually moved.".bold().blue());
        println!("{}", "To apply these changes, run the same command without the -p flag.".blue());
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

