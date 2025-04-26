mod history;
mod repl;
mod transformers;

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

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Smart Move - An enhanced mv command with transformation capabilities",
    long_about = None
)]
struct Args {
    /// Files or patterns to move/rename
    #[arg(value_name = "SOURCE")]
    source: Vec<String>,

    /// Destination file or directory
    #[arg(value_name = "DESTINATION")]
    destination: Option<String>,

    /// Interactive mode - launch REPL interface
    #[arg(short, long, action = ArgAction::SetTrue)]
    interactive: bool,

    /// Preview changes without applying them
    #[arg(short, long, action = ArgAction::SetTrue)]
    preview: bool,

    /// Process subdirectories recursively
    #[arg(short, long, action = ArgAction::SetTrue)]
    recursive: bool,

    /// Comma-separated list of file extensions to process
    #[arg(short, long, value_name = "EXTENSIONS")]
    extensions: Option<String>,

    /// Remove accents
    #[arg(short = 'a', long, action = ArgAction::SetTrue)]
    remove_accents: bool,

    /// Convert to clean format (remove special chars, normalize spaces)
    #[arg(long = "clean", action = ArgAction::SetTrue)]
    clean: bool,

    /// Convert to snake_case
    #[arg(long = "snake", action = ArgAction::SetTrue)]
    snake: bool,

    /// Convert to kebab-case
    #[arg(long = "kebab", action = ArgAction::SetTrue)]
    kebab: bool,

    /// Convert to Title Case
    #[arg(long = "title", action = ArgAction::SetTrue)]
    title: bool,

    /// Convert to camelCase
    #[arg(long = "camel", action = ArgAction::SetTrue)]
    camel: bool,

    /// Convert to PascalCase
    #[arg(long = "pascal", action = ArgAction::SetTrue)]
    pascal: bool,

    /// Convert to lowercase
    #[arg(long = "lower", action = ArgAction::SetTrue)]
    lower: bool,

    /// Convert to UPPERCASE
    #[arg(long = "upper", action = ArgAction::SetTrue)]
    upper: bool,

    /// Same as preview - show what would change without making changes
    #[arg(long, action = ArgAction::SetTrue)]
    dry_run: bool,

    /// Comma-separated patterns to exclude
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
    // Parse command line arguments
    let mut args = Args::parse();

    // If both preview and dry-run are specified, enable preview mode
    if args.dry_run {
        args.preview = true;
    }

    // If interactive mode is enabled, launch REPL
    if args.interactive {
        run_interactive_mode(args.max_history_size)?;
        return Ok(());
    }

    // Determine which operation to perform
    if is_transformation_requested(&args) {
        // Operate in transformation mode
        run_transformation_mode(&args)?;
    } else if !args.source.is_empty() {
        // Operate in standard mv mode
        run_standard_mv_mode(&args)?;
    } else {
        eprintln!("Error: No files specified and no mode selected.");
        eprintln!("Use --help for usage information or -i for interactive mode.");
        process::exit(1);
    }

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

/// Check if any transformation options are enabled
fn is_transformation_requested(args: &Args) -> bool {
    args.clean
        || args.snake
        || args.kebab
        || args.title
        || args.camel
        || args.pascal
        || args.lower
        || args.upper
}

/// Run in transformation mode - rename files according to specified transformation
fn run_transformation_mode(args: &Args) -> Result<(), Box<dyn Error>> {
    if args.source.is_empty() {
        return Err("No source files specified for transformation".into());
    }

    // Determine which transformation to apply
    let transform_type = if args.clean {
        TransformType::Clean
    } else if args.snake {
        TransformType::Snake
    } else if args.kebab {
        TransformType::Kebab
    } else if args.title {
        TransformType::Title
    } else if args.camel {
        TransformType::Camel
    } else if args.pascal {
        TransformType::Pascal
    } else if args.lower {
        TransformType::Lower
    } else if args.upper {
        TransformType::Upper
    } else {
        // Default to clean transformation
        TransformType::Clean
    };

    // Process exclude patterns
    let exclude_patterns: Vec<Regex> = process_exclude_patterns(args.exclude.as_deref())?;

    // Process file extensions
    let extensions: Option<Vec<String>> = args.extensions.as_ref().map(|exts| {
        exts.split(',')
            .map(|ext| ext.trim().to_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect()
    });

    // Print operation mode
    println!(
        "\n{}\n",
        format!(
            "Smart Move - {} Mode",
            if args.preview { "Preview" } else { "Rename" }
        )
        .bold()
    );
    println!("Transformation: {}", transform_type.as_str().green());
    println!(
        "Recursive: {}",
        if args.recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Extensions filter: {}",
        match &extensions {
            Some(exts) if !exts.is_empty() => exts.join(", ").cyan(),
            _ => "None (all files)".yellow(),
        }
    );
    println!(
        "Exclude patterns: {}\n",
        if !exclude_patterns.is_empty() {
            args.exclude.as_deref().unwrap_or_default().cyan()
        } else {
            "None".yellow()
        }
    );

    // Process each source pattern
    let mut stats = Stats::default();
    for source_pattern in &args.source {
        process_pattern(
            source_pattern,
            transform_type,
            args.preview,
            args.recursive,
            &exclude_patterns,
            &extensions,
            &mut stats,
        )?;
    }

    // Print the results
    println!("\n{}:", "Results".bold());
    println!("Files processed: {}", stats.processed.to_string().cyan());
    println!("Files to be renamed: {}", stats.renamed.to_string().green());
    println!("Files skipped: {}", stats.skipped.to_string().yellow());
    println!("Errors encountered: {}", stats.errors.to_string().red());

    if args.preview && stats.renamed > 0 {
        println!(
            "\n{}",
            "This was a preview only. No files were actually renamed."
                .bold()
                .blue()
        );
        println!(
            "{}",
            "To apply these changes, run the command without --preview or --dry-run option.".blue()
        );
    }

    Ok(())
}

/// Run in standard mv mode - move/rename files like the standard mv command
fn run_standard_mv_mode(args: &Args) -> Result<(), Box<dyn Error>> {
    if args.source.is_empty() {
        return Err("No source files specified".into());
    }

    if args.destination.is_none() {
        return Err("No destination specified".into());
    }

    let destination = args.destination.as_ref().unwrap();
    let dest_path = PathBuf::from(destination);

    // Check if destination is a directory
    let dest_is_dir = dest_path.is_dir();

    // If we have multiple sources, destination must be a directory
    if args.source.len() > 1 && !dest_is_dir {
        return Err("When specifying multiple sources, destination must be a directory".into());
    }

    for source in &args.source {
        let source_path = PathBuf::from(source);

        if !source_path.exists() {
            eprintln!("{}: Source file not found: {}", "Error".red(), source);
            continue;
        }

        let target_path = if dest_is_dir {
            // If destination is a directory, preserve source filename
            let source_filename = source_path.file_name().ok_or("Invalid source filename")?;
            dest_path.join(source_filename)
        } else {
            // Otherwise use the destination as-is
            dest_path.clone()
        };

        // Check if target already exists
        if target_path.exists() && source_path != target_path {
            eprintln!(
                "{}: Cannot move '{}' to '{}' - destination exists",
                "Error".red(),
                source,
                target_path.display()
            );
            continue;
        }

        // Perform the move
        if args.preview {
            println!(
                "{} '{}' → '{}'",
                "Preview:".blue(),
                source,
                target_path.display()
            );
        } else {
            match fs::rename(&source_path, &target_path) {
                Ok(_) => println!("Moved: '{}' → '{}'", source, target_path.display()),
                Err(e) => eprintln!(
                    "{}: Failed to move '{}' to '{}' - {}",
                    "Error".red(),
                    source,
                    target_path.display(),
                    e
                ),
            }
        }
    }

    Ok(())
}

/// Process exclude patterns into Regex objects
fn process_exclude_patterns(patterns: Option<&str>) -> Result<Vec<Regex>, Box<dyn Error>> {
    match patterns {
        Some(patterns) => {
            let result: Vec<Regex> = patterns
                .split(',')
                .filter_map(|p| {
                    let p = p.trim();
                    if p.is_empty() {
                        None
                    } else {
                        match Regex::new(p) {
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

/// Process a single glob pattern for transformation
fn process_pattern(
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

    // Use WalkDir for recursive traversal or just the directory entries
    let entries = if recursive {
        WalkDir::new(&base_dir)
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    } else {
        let paths = fs::read_dir(&base_dir)
            .map_err(|e| format!("Failed to read directory {}: {}", base_dir.display(), e))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect::<Vec<_>>();

        // Convert paths to WalkDir entries
        paths
            .into_iter()
            .filter_map(|path| {
                WalkDir::new(&path)
                    .max_depth(0)
                    .into_iter()
                    .next()
                    .and_then(|e| e.ok())
            })
            .collect::<Vec<_>>()
    };

    for entry in entries {
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
            stats,
        )?;
    }

    Ok(())
}

/// Check if a path matches a glob pattern
fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
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

        Regex::new(&format!("^{pattern_regex}$"))
            .map(|re| re.is_match(&path_str))
            .unwrap_or(false)
    } else {
        // Exact match
        path_str == pattern
    }
}

/// Process a single file for transformation
fn process_file(
    file_path: &Path,
    transform_type: TransformType,
    preview_only: bool,
    exclude_patterns: &[Regex],
    extensions: &Option<Vec<String>>,
    stats: &mut Stats,
) -> Result<(), Box<dyn Error>> {
    let file_path_str = file_path.to_string_lossy();

    // Skip if the file matches an exclude pattern
    if exclude_patterns
        .iter()
        .any(|pattern| pattern.is_match(&file_path_str))
    {
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
    let Some(filename) = file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
    else {
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
    if new_path.exists() && file_path != new_path {
        println!(
            "{}: Cannot rename \"{}\" to \"{}\" - file already exists",
            "Error".red(),
            file_path_str,
            new_path.to_string_lossy()
        );
        stats.errors += 1;
        return Ok(());
    }

    // Log the rename operation
    println!(
        "{}{}\"{}\" → \"{}\"",
        if preview_only {
            "[PREVIEW] ".blue()
        } else {
            "".into()
        },
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
