mod transformers;
mod history;
mod repl;

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::error::Error;

use clap::{App, Arg, ArgMatches};
use colored::*;
use regex::Regex;
use walkdir::WalkDir;
use dirs::home_dir;

use transformers::{TransformType, transform};
use history::HistoryManager;
use repl::InteractiveSession;

struct Args {
    source: Vec<String>,
    destination: Option<String>,
    interactive: bool,
    preview: bool,
    recursive: bool,
    extensions: Option<String>,
    remove_accents: bool,
    clean: bool,
    snake: bool,
    kebab: bool,
    title: bool,
    camel: bool,
    pascal: bool,
    lower: bool,
    upper: bool,
    dry_run: bool,
    exclude: Option<String>,
    max_history_size: usize,
}

impl Args {
    fn parse() -> Self {
        let matches = App::new("smv")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Smart Move - An enhanced mv command with transformation capabilities")
            // Source files
            .arg(Arg::with_name("source")
                .short('s')
                .long("source")
                .multiple_values(true)
                .help("Files or patterns to move/rename")
                .takes_value(true))
            // Destination
            .arg(Arg::with_name("destination")
                .short('d')
                .long("destination")
                .help("Destination file or directory")
                .takes_value(true))
            // Interactive mode
            .arg(Arg::with_name("interactive")
                .short('i')
                .long("interactive")
                .help("Interactive mode - launch REPL interface")
                .takes_value(false))
            // Preview mode
            .arg(Arg::with_name("preview")
                .short('p')
                .long("preview")
                .help("Preview changes without applying them")
                .takes_value(false))
            // Recursive
            .arg(Arg::with_name("recursive")
                .short('r')
                .long("recursive")
                .help("Process subdirectories recursively")
                .takes_value(false))
            // Extensions
            .arg(Arg::with_name("extensions")
                .short('e')
                .long("extensions")
                .help("Comma-separated list of file extensions to process")
                .takes_value(true))
            // Remove accents
            .arg(Arg::with_name("remove_accents")
                .short('a')
                .long("remove-accents")
                .help("Remove accents")
                .takes_value(false))
            // Clean
            .arg(Arg::with_name("clean")
                .long("clean")
                .help("Convert to clean format (remove special chars, normalize spaces)")
                .takes_value(false))
            // Snake case
            .arg(Arg::with_name("snake")
                .long("snake")
                .help("Convert to snake_case")
                .takes_value(false))
            // Kebab case
            .arg(Arg::with_name("kebab")
                .long("kebab")
                .help("Convert to kebab-case")
                .takes_value(false))
            // Title case
            .arg(Arg::with_name("title")
                .long("title")
                .help("Convert to Title Case")
                .takes_value(false))
            // Camel case
            .arg(Arg::with_name("camel")
                .long("camel")
                .help("Convert to camelCase")
                .takes_value(false))
            // Pascal case
            .arg(Arg::with_name("pascal")
                .long("pascal")
                .help("Convert to PascalCase")
                .takes_value(false))
            // Lowercase
            .arg(Arg::with_name("lower")
                .long("lower")
                .help("Convert to lowercase")
                .takes_value(false))
            // Uppercase
            .arg(Arg::with_name("upper")
                .long("upper")
                .help("Convert to UPPERCASE")
                .takes_value(false))
            // Dry run
            .arg(Arg::with_name("dry_run")
                .long("dry-run")
                .help("Same as preview - show what would change without making changes")
                .takes_value(false))
            // Exclude patterns
            .arg(Arg::with_name("exclude")
                .long("exclude")
                .help("Comma-separated patterns to exclude")
                .takes_value(true))
            // Max history size
            .arg(Arg::with_name("max_history_size")
                .long("max-history-size")
                .help("Maximum number of operations to keep in history")
                .default_value("50")
                .takes_value(true))
            .get_matches();

        Self::from_arg_matches(&matches)
    }

    fn from_arg_matches(matches: &ArgMatches) -> Self {
        let source = match matches.values_of("source") {
            Some(values) => values.map(String::from).collect(),
            None => Vec::new(),
        };

        let destination = matches.value_of("destination").map(String::from);

        let max_history_size = matches
            .value_of("max_history_size")
            .unwrap_or("50")
            .parse()
            .unwrap_or(50);

        Self {
            source,
            destination,
            interactive: matches.is_present("interactive"),
            preview: matches.is_present("preview"),
            recursive: matches.is_present("recursive"),
            extensions: matches.value_of("extensions").map(String::from),
            remove_accents: matches.is_present("remove_accents"),
            clean: matches.is_present("clean"),
            snake: matches.is_present("snake"),
            kebab: matches.is_present("kebab"),
            title: matches.is_present("title"),
            camel: matches.is_present("camel"),
            pascal: matches.is_present("pascal"),
            lower: matches.is_present("lower"),
            upper: matches.is_present("upper"),
            dry_run: matches.is_present("dry_run"),
            exclude: matches.value_of("exclude").map(String::from),
            max_history_size,
        }
    }
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
    args.clean || args.snake || args.kebab || args.title || 
    args.camel || args.pascal || args.lower || args.upper
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
    let exclude_patterns = process_exclude_patterns(args.exclude.as_deref())?;

    // Process file extensions
    let extensions: Option<Vec<String>> = args.extensions.as_ref().map(|exts| {
        exts.split(',')
            .map(|ext| ext.trim().to_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect()
    });

    // Print operation mode
    println!("\n{}\n", format!(
        "Smart Move - {} Mode", 
        if args.preview { "Preview" } else { "Rename" }).bold()
    );
    println!("Transformation: {}", transform_type.as_str().green());
    println!("Recursive: {}", if args.recursive { "Yes".green() } else { "No".yellow() });
    println!("Extensions filter: {}", match &extensions {
        Some(exts) if !exts.is_empty() => exts.join(", ").cyan(),
        _ => "None (all files)".yellow(),
    });
    println!("Exclude patterns: {}\n", if !exclude_patterns.is_empty() {
        args.exclude.as_deref().unwrap_or_default().cyan()
    } else {
        "None".yellow()
    });

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
            &mut stats
        )?;
    }

    // Print the results
    println!("\n{}:", "Results".bold());
    println!("Files processed: {}", stats.processed.to_string().cyan());
    println!("Files to be renamed: {}", stats.renamed.to_string().green());
    println!("Files skipped: {}", stats.skipped.to_string().yellow());
    println!("Errors encountered: {}", stats.errors.to_string().red());

    if args.preview && stats.renamed > 0 {
        println!("\n{}", "This was a preview only. No files were actually renamed.".bold().blue());
        println!("{}", "To apply these changes, run the command without --preview or --dry-run option.".blue());
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
            eprintln!("{}: Cannot move '{}' to '{}' - destination exists", 
                "Error".red(), source, target_path.display());
            continue;
        }

        // Perform the move
        if args.preview {
            println!("{} '{}' → '{}'", "Preview:".blue(), source, target_path.display());
        } else {
            match fs::rename(&source_path, &target_path) {
                Ok(_) => println!("Moved: '{}' → '{}'", source, target_path.display()),
                Err(e) => eprintln!("{}: Failed to move '{}' to '{}' - {}", 
                    "Error".red(), source, target_path.display(), e),
            }
        }
    }

    Ok(())
}

/// Process exclude patterns into Regex objects
fn process_exclude_patterns(patterns: Option<&str>) -> Result<Vec<Regex>, Box<dyn Error>> {
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
        
        Regex::new(&format!("^{}$", pattern_regex))
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
