mod cnp_grammar;
mod file_ops;
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

use cnp_grammar::{CnpCommand, CnpGrammarParser};
use file_ops::{FileOpConfig, copy_files, expand_glob_patterns, move_files, remove_files};
use history::HistoryManager;
use repl::InteractiveSession;
use transformers::{TransformType, transform};
use ui::UserInterface;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Smart Move - Enhanced file operations with XFD syntax",
    long_about = "CNP Smart Move (SMV) - Enhanced file operations with transformations

SYNTAX:
  smv [COMMAND] [OPTIONS] [TARGET] [FLAGS]

COMMANDS:
  CHANGE \"old\" INTO \"new\" [target]  Replace substring in filenames (use empty \"\" to remove prefix)
  snake, kebab, pascal, camel           Transform filename case/format
  sort, group, flatten                  Organize files
  mv source destination                 Move files/directories
  cp source destination                 Copy files/directories
  rm targets...                         Remove files/directories
  rm path FILTER... [-flags]           Query-based deletion with CNP filters
  -cd directories...                    Create directories (flag combo)
  -cF files...                          Create/touch files (flag combo)
  interactive, tui, undo                Special modes

FLAGS:
  -r       Recursive (process subdirectories)
  -p       Preview (show changes without applying)
  -f       Force (skip confirmations/overwrite files)
  -n       No-clobber (do not overwrite existing files)
  -i       Case-insensitive pattern matching
  -I       Interactive mode
  -T       Terminal UI mode
  -u       Undo last operation
  -v       Verbose output
  -a       Include hidden files (default: excluded)
  --preserve  Preserve file attributes

EXAMPLES:
  smv snake . -rp                      # Preview snake_case transformation
  smv CHANGE \"old\" INTO \"new\" . -r    # Replace substring in filenames
  smv CHANGE \"IMG_\" INTO \"\" . -rp      # Preview removal of IMG_ prefix from all files
  smv CHANGE \"DSC_\" INTO \"\" pictures/  # Remove DSC_ prefix from files in pictures/
  smv mv file.txt newname.txt          # Move/rename file
  smv cp *.txt backup/ -r              # Copy files recursively
  smv rm . NAME:- -p                   # Preview delete files containing \"-\"
  smv rm . 'NAME:index_of*' -rip       # Delete files starting with \"index_of\" (case-insensitive)
  smv rm . EXT:log                     # Delete all .log files
  smv rm . 'NAME:*Test*' -i            # Delete files containing \"test\" (any case)
  smv rm . TYPE:file SIZE>1MB -r       # Delete files larger than 1MB recursively
  smv -cd newdir                       # Create directory
  smv -cF newfile.txt                  # Create file
  smv -cF edt_pitch.md                 # Create markdown file
  smv tui                              # Launch terminal UI
  smv -I                               # Interactive mode
  
  Note: For -cF flag, put flags BEFORE filenames.
  WRONG: smv . -cF filename.txt         # This treats '.' as a file to create!

Use 'smv --help' for full documentation."
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

    /// Additional arguments for CNP grammar or legacy extensions
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,

    // === XFD FLAGS (single character, stackable) ===
    /// Stackable flags: r(ecursive), p(review), f(orce), i(nteractive), T(ui), u(ndo), c(reate), d(irectory)
    #[arg(short = 'r', action = ArgAction::SetTrue, help = "Recursive - process subdirectories")]
    recursive: bool,

    #[arg(short = 'p', action = ArgAction::SetTrue, help = "Preview - show changes without applying")]
    preview: bool,

    #[arg(short = 'f', action = ArgAction::SetTrue, help = "Force - skip confirmations")]
    force: bool,

    #[arg(short = 'i', action = ArgAction::SetTrue, help = "Case-insensitive pattern matching")]
    case_insensitive: bool,

    #[arg(long = "ignore-case", action = ArgAction::SetTrue, help = "Case-insensitive matching (CNP standard)")]
    ignore_case: bool,

    #[arg(short = 'I', action = ArgAction::SetTrue, help = "Interactive - launch REPL interface")]
    interactive: bool,

    #[arg(short = 'T', action = ArgAction::SetTrue, help = "TUI - launch terminal UI file explorer")]
    tui: bool,

    #[arg(short = 'u', action = ArgAction::SetTrue, help = "Undo - reverse last operation")]
    undo: bool,

    #[arg(short = 'c', action = ArgAction::SetTrue, help = "Create - must be combined with -d (directories) or -F (files)")]
    create: bool,

    #[arg(short = 'd', action = ArgAction::SetTrue, help = "Directory - when combined with -c, creates directories")]
    directory: bool,

    #[arg(short = 'F', action = ArgAction::SetTrue, help = "File - when combined with -c, creates/touches files")]
    file_flag: bool,

    // === BASIC FILE OPERATIONS ===
    #[arg(short = 'n', action = ArgAction::SetTrue, help = "No-clobber - do not overwrite existing files")]
    no_clobber: bool,

    #[arg(short = 'L', action = ArgAction::SetTrue, help = "Dereference symbolic links")]
    dereference: bool,

    #[arg(short = 'P', action = ArgAction::SetTrue, help = "Do not follow symbolic links")]
    no_follow: bool,

    #[arg(long = "preserve", action = ArgAction::SetTrue, help = "Preserve file attributes (mode, ownership, timestamps)")]
    preserve: bool,

    #[arg(long = "interactive-confirm", action = ArgAction::SetTrue, help = "Prompt before overwriting files")]
    interactive_confirm: bool,

    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue, help = "Verbose output")]
    verbose: bool,

    #[arg(short = 'a', long = "hidden", action = ArgAction::SetTrue, help = "Include hidden files (default: excluded)")]
    hidden: bool,

    #[arg(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        help = "Set file mode (permissions) for mkdir"
    )]
    mode: Option<String>,

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

    // Check if we should use CNP grammar parsing
    if should_use_cnp_grammar(&args) {
        return run_cnp_command(&args);
    }

    // Parse legacy XFD command
    let command = match parse_xfd_command(&args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("For help: smv --help");
            process::exit(1);
        }
    };

    // Execute legacy command
    match command {
        XfdCommand::Change { old, new } => {
            let transform_type = TransformType::replace(&old, &new);
            run_transform_command(&args, transform_type)?
        }
        XfdCommand::Regex {
            pattern,
            replacement,
        } => {
            let transform_type = TransformType::replace_regex(&pattern, &replacement);
            run_transform_command(&args, transform_type)?
        }
        XfdCommand::Transform(transform_type) => run_transform_command(&args, transform_type)?,
        XfdCommand::Sort { method } => run_sort_command(&args, method)?,
        XfdCommand::Interactive => run_interactive_mode(args.max_history_size)?,
        XfdCommand::Tui => run_tui_mode()?,
        XfdCommand::Undo => run_undo_mode(args.max_history_size)?,
        XfdCommand::Move {
            sources,
            destination,
        } => run_move_command(&args, &sources, &destination)?,
        XfdCommand::Copy {
            sources,
            destination,
        } => run_copy_command(&args, &sources, &destination)?,
        XfdCommand::Remove { targets } => run_remove_command(&args, &targets)?,
        XfdCommand::Mkdir { directories } => run_mkdir_command(&args, &directories)?,
        XfdCommand::Touch { files } => run_touch_command(&args, &files)?,
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum XfdCommand {
    Change {
        old: String,
        new: String,
    },
    Regex {
        pattern: String,
        replacement: String,
    },
    Transform(TransformType),
    Sort {
        method: SortMethod,
    },
    Interactive,
    Tui,
    Undo,
    Move {
        sources: Vec<String>,
        destination: String,
    },
    Copy {
        sources: Vec<String>,
        destination: String,
    },
    Remove {
        targets: Vec<String>,
    },
    Mkdir {
        directories: Vec<String>,
    },
    Touch {
        files: Vec<String>,
    },
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
    // Check for composable flags first (highest priority)
    if args.create && args.directory {
        // -cd flag combination: create directories
        let mut directories = Vec::new();

        // Collect directory names from all possible argument positions
        if let Some(ref command) = args.command {
            directories.push(command.clone());
        }
        if let Some(ref arg1) = args.arg1 {
            directories.push(arg1.clone());
        }
        if let Some(ref into_keyword) = args.into_keyword {
            if into_keyword != "INTO" {
                directories.push(into_keyword.clone());
            }
        }
        if let Some(ref arg2) = args.arg2 {
            directories.push(arg2.clone());
        }
        if let Some(ref target) = args.target {
            directories.push(target.clone());
        }
        directories.extend(args.args.iter().cloned());

        if directories.is_empty() {
            return Err("Create directory requires at least one directory name".into());
        }

        return Ok(XfdCommand::Mkdir { directories });
    }

    if args.create && args.file_flag {
        // -cF flag combination: create/touch files
        let mut files = Vec::new();

        // Collect file names from all possible argument positions
        if let Some(ref command) = args.command {
            files.push(command.clone());
        }
        if let Some(ref arg1) = args.arg1 {
            files.push(arg1.clone());
        }
        if let Some(ref into_keyword) = args.into_keyword {
            if into_keyword != "INTO" {
                files.push(into_keyword.clone());
            }
        }
        if let Some(ref arg2) = args.arg2 {
            files.push(arg2.clone());
        }
        if let Some(ref target) = args.target {
            files.push(target.clone());
        }
        files.extend(args.args.iter().cloned());

        if files.is_empty() {
            return Err("Create file requires at least one file name".into());
        }

        return Ok(XfdCommand::Touch { files });
    }

    // Check for single flags
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
            let old = args
                .arg1
                .as_ref()
                .ok_or("Missing old string for CHANGE command")?;
            if args.into_keyword.as_deref() != Some("INTO") {
                return Err("Expected 'INTO' keyword after old string".into());
            }
            let new = args
                .arg2
                .as_ref()
                .ok_or("Missing new string after INTO keyword")?;

            // Handle prefix removal: CHANGE "prefix" INTO ""
            if new.is_empty() {
                Ok(XfdCommand::Transform(TransformType::remove_prefix(old)))
            } else {
                Ok(XfdCommand::Change {
                    old: old.clone(),
                    new: new.clone(),
                })
            }
        }
        Some("REGEX") => {
            let pattern = args
                .arg1
                .as_ref()
                .ok_or("Missing pattern for REGEX command")?;
            if args.into_keyword.as_deref() != Some("INTO") {
                return Err("Expected 'INTO' keyword after pattern".into());
            }
            let replacement = args
                .arg2
                .as_ref()
                .ok_or("Missing replacement after INTO keyword")?;
            Ok(XfdCommand::Regex {
                pattern: pattern.clone(),
                replacement: replacement.clone(),
            })
        }
        Some("snake") => Ok(XfdCommand::Transform(TransformType::Snake)),
        Some("kebab") => Ok(XfdCommand::Transform(TransformType::Kebab)),
        Some("pascal") => Ok(XfdCommand::Transform(TransformType::Pascal)),
        Some("camel") => Ok(XfdCommand::Transform(TransformType::Camel)),
        Some("title") => Ok(XfdCommand::Transform(TransformType::Title)),
        Some("lower") => Ok(XfdCommand::Transform(TransformType::Lower)),
        Some("upper") => Ok(XfdCommand::Transform(TransformType::Upper)),
        Some("clean") => Ok(XfdCommand::Transform(TransformType::Clean)),
        Some("sort") => Ok(XfdCommand::Sort {
            method: SortMethod::Group,
        }), // Default sort method
        Some("group") => Ok(XfdCommand::Sort {
            method: SortMethod::Group,
        }),
        Some("flatten") => Ok(XfdCommand::Sort {
            method: SortMethod::Flatten,
        }),
        Some("interactive") => Ok(XfdCommand::Interactive),
        Some("tui") => Ok(XfdCommand::Tui),
        Some("undo") => Ok(XfdCommand::Undo),
        Some("cp") => parse_copy_command(args),
        Some("mv") => parse_move_command(args),
        Some("rm") => parse_remove_command(args),
        Some("mkdir") => parse_mkdir_command(args),
        Some(unknown) => Err(format!("Unknown command: {unknown}").into()),
        None => {
            // Check if this looks like a basic file operation (source(s) and destination)
            if let Some(ref command) = args.command {
                // First arg is source, check if we have destination
                if let Some(ref target) = args.target {
                    // This looks like: smv source dest
                    let sources = vec![command.clone()];
                    return Ok(XfdCommand::Move {
                        sources,
                        destination: target.clone(),
                    });
                }
            }

            // Check for multiple sources in args
            if args.args.len() >= 2 {
                // Last arg is destination, rest are sources
                let mut sources = Vec::new();
                if let Some(ref command) = args.command {
                    sources.push(command.clone());
                }
                if let Some(ref arg1) = args.arg1 {
                    sources.push(arg1.clone());
                }
                if let Some(ref arg2) = args.arg2 {
                    sources.push(arg2.clone());
                }

                // Add all but the last arg as sources
                for i in 0..args.args.len() - 1 {
                    sources.push(args.args[i].clone());
                }

                let destination = args.args.last().unwrap().clone();
                return Ok(XfdCommand::Move {
                    sources,
                    destination,
                });
            }

            Err("No command specified. Use: CHANGE \"old\" INTO \"new\", transform commands, or basic file operations".into())
        }
    }
}

fn parse_copy_command(args: &Args) -> Result<XfdCommand, Box<dyn Error>> {
    let mut sources = Vec::new();

    // For cp command, arg1 is the first source
    if let Some(ref arg1) = args.arg1 {
        sources.push(arg1.clone());
    }

    // Collect potential sources from all non-destination arguments
    if let Some(ref into_val) = args.into_keyword {
        if into_val != "INTO" {
            // If we have arg2, target, or args, into_keyword is a source, not destination
            if args.arg2.is_some() || args.target.is_some() || !args.args.is_empty() {
                sources.push(into_val.clone());
            } else {
                // This is actually the destination (only if no other args)
                if sources.is_empty() {
                    return Err("Copy command requires at least one source".into());
                }
                return Ok(XfdCommand::Copy {
                    sources,
                    destination: into_val.clone(),
                });
            }
        }
    }

    // If we have arg2, it's either a source or destination
    if let Some(ref arg2) = args.arg2 {
        if args.into_keyword.is_none() || args.into_keyword.as_deref() != Some("INTO") {
            // If there are more arguments after arg2, arg2 is a source
            if args.target.is_some() || !args.args.is_empty() {
                sources.push(arg2.clone());
            } else {
                // arg2 is the destination
                if sources.is_empty() {
                    return Err("Copy command requires at least one source".into());
                }
                return Ok(XfdCommand::Copy {
                    sources,
                    destination: arg2.clone(),
                });
            }
        }
    }

    // If we have a target, it could be a source or destination
    if let Some(ref target) = args.target {
        if args.args.is_empty() {
            // target is the destination
            if sources.is_empty() {
                return Err("Copy command requires at least one source".into());
            }
            return Ok(XfdCommand::Copy {
                sources,
                destination: target.clone(),
            });
        } else {
            // target is another source
            sources.push(target.clone());
        }
    }

    // Add all but last arg as sources, last arg is destination
    if !args.args.is_empty() {
        for i in 0..args.args.len() - 1 {
            sources.push(args.args[i].clone());
        }

        if sources.is_empty() {
            return Err("Copy command requires at least one source".into());
        }

        let destination = args.args.last().unwrap().clone();
        return Ok(XfdCommand::Copy {
            sources,
            destination,
        });
    }

    Err("Copy command requires source and destination".into())
}

fn parse_remove_command(args: &Args) -> Result<XfdCommand, Box<dyn Error>> {
    let mut targets = Vec::new();

    // For rm command, arg1 is the first target
    if let Some(ref arg1) = args.arg1 {
        targets.push(arg1.clone());
    }

    // Collect all additional targets
    if let Some(ref into_val) = args.into_keyword {
        if into_val != "INTO" {
            targets.push(into_val.clone());
        }
    }

    if let Some(ref arg2) = args.arg2 {
        targets.push(arg2.clone());
    }

    if let Some(ref target) = args.target {
        targets.push(target.clone());
    }

    // Add all remaining arguments as targets
    targets.extend(args.args.iter().cloned());

    if targets.is_empty() {
        return Err("Remove command requires at least one target".into());
    }

    Ok(XfdCommand::Remove { targets })
}

fn parse_mkdir_command(args: &Args) -> Result<XfdCommand, Box<dyn Error>> {
    let mut directories = Vec::new();

    // For mkdir command, arg1 is the first directory
    if let Some(ref arg1) = args.arg1 {
        directories.push(arg1.clone());
    }

    // Collect all additional directories
    if let Some(ref into_val) = args.into_keyword {
        if into_val != "INTO" {
            directories.push(into_val.clone());
        }
    }

    if let Some(ref arg2) = args.arg2 {
        directories.push(arg2.clone());
    }

    if let Some(ref target) = args.target {
        directories.push(target.clone());
    }

    // Add all remaining arguments as directories
    directories.extend(args.args.iter().cloned());

    if directories.is_empty() {
        return Err("Mkdir command requires at least one directory".into());
    }

    Ok(XfdCommand::Mkdir { directories })
}

fn parse_move_command(args: &Args) -> Result<XfdCommand, Box<dyn Error>> {
    let mut sources = Vec::new();

    // For mv command, arg1 is the first source
    if let Some(ref arg1) = args.arg1 {
        sources.push(arg1.clone());
    }

    // Collect potential sources from all non-destination arguments
    if let Some(ref into_val) = args.into_keyword {
        if into_val != "INTO" {
            // If we have arg2, target, or args, into_keyword is a source, not destination
            if args.arg2.is_some() || args.target.is_some() || !args.args.is_empty() {
                sources.push(into_val.clone());
            } else {
                // This is actually the destination (only if no other args)
                if sources.is_empty() {
                    return Err("Move command requires at least one source".into());
                }
                return Ok(XfdCommand::Move {
                    sources,
                    destination: into_val.clone(),
                });
            }
        }
    }

    // If we have arg2, it's either a source or destination
    if let Some(ref arg2) = args.arg2 {
        if args.into_keyword.is_none() || args.into_keyword.as_deref() != Some("INTO") {
            // If there are more arguments after arg2, arg2 is a source
            if args.target.is_some() || !args.args.is_empty() {
                sources.push(arg2.clone());
            } else {
                // arg2 is the destination
                if sources.is_empty() {
                    return Err("Move command requires at least one source".into());
                }
                return Ok(XfdCommand::Move {
                    sources,
                    destination: arg2.clone(),
                });
            }
        }
    }

    // If we have a target, it could be a source or destination
    if let Some(ref target) = args.target {
        if args.args.is_empty() {
            // target is the destination
            if sources.is_empty() {
                return Err("Move command requires at least one source".into());
            }
            return Ok(XfdCommand::Move {
                sources,
                destination: target.clone(),
            });
        } else {
            // target is another source
            sources.push(target.clone());
        }
    }

    // Add all but last arg as sources, last arg is destination
    if !args.args.is_empty() {
        for i in 0..args.args.len() - 1 {
            sources.push(args.args[i].clone());
        }

        if sources.is_empty() {
            return Err("Move command requires at least one source".into());
        }

        let destination = args.args.last().unwrap().clone();
        return Ok(XfdCommand::Move {
            sources,
            destination,
        });
    }

    Err("Move command requires source and destination".into())
}

fn run_move_command(
    args: &Args,
    sources: &[String],
    destination: &str,
) -> Result<(), Box<dyn Error>> {
    let config = build_file_op_config(args);

    println!("\n{}", "CNP Smart Move - Move Operation".bold());
    println!("Sources: {}", sources.join(", ").cyan());
    println!("Destination: {}", destination.cyan());
    println!(
        "Recursive: {}",
        if config.recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Force: {}",
        if config.force {
            "Yes".red()
        } else {
            "No".green()
        }
    );
    println!(
        "No-clobber: {}",
        if config.no_clobber {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Interactive: {}",
        if config.interactive {
            "Yes".cyan()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Preserve metadata: {}",
        if config.preserve_metadata {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Expand glob patterns
    let expanded_sources = expand_glob_patterns(sources)?;
    let dest_path = Path::new(destination);

    // Execute move operation
    let stats = move_files(&expanded_sources, dest_path, &config)?;

    // Print results
    println!("\n{}:", "Results".bold());
    println!("Files processed: {}", stats.processed.to_string().cyan());
    println!("Files moved: {}", stats.moved.to_string().green());
    println!("Errors: {}", stats.errors.to_string().red());
    println!("Skipped: {}", stats.skipped.to_string().yellow());

    Ok(())
}

fn run_remove_command(args: &Args, targets: &[String]) -> Result<(), Box<dyn Error>> {
    let config = build_file_op_config(args);

    println!("\n{}", "CNP Smart Move - Remove Operation".bold());
    println!("Targets: {}", targets.join(", ").cyan());
    println!(
        "Recursive: {}",
        if config.recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Force: {}",
        if config.force {
            "Yes".red()
        } else {
            "No".green()
        }
    );
    println!(
        "Interactive: {}",
        if config.interactive {
            "Yes".cyan()
        } else {
            "No".yellow()
        }
    );

    println!();

    // Expand globs and get source paths
    let expanded_targets = expand_glob_patterns(targets)?;

    if expanded_targets.is_empty() {
        return Err("No files match the specified targets".into());
    }

    // Perform the remove operation
    let stats = remove_files(&expanded_targets, &config)?;

    println!("\n{}", "Results:".bold());
    println!("Files processed: {}", stats.processed);
    println!("Files removed: {}", stats.moved); // Using moved count for removed
    println!("Errors: {}", stats.errors);
    println!("Skipped: {}", stats.skipped);

    Ok(())
}

fn run_mkdir_command(args: &Args, directories: &[String]) -> Result<(), Box<dyn Error>> {
    let config = build_file_op_config(args);

    println!("\n{}", "CNP Smart Move - Create Directories".bold());
    println!("Directories: {}", directories.join(", ").cyan());
    println!(
        "Parents: {}",
        if args.recursive {
            "Yes (create parent directories)".green()
        } else {
            "No".yellow()
        }
    );
    println!("Mode: {}", args.mode.as_deref().unwrap_or("default").cyan());
    println!(
        "Verbose: {}",
        if config.verbose {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Parse mode if provided
    let mode = if let Some(mode_str) = &args.mode {
        parse_mode_string(mode_str)?
    } else {
        None
    };

    // Perform the mkdir operation
    let stats = file_ops::create_directories(directories, args.recursive, mode, config.verbose)?;

    println!("\n{}", "Results:".bold());
    println!("Directories processed: {}", stats.processed);
    println!("Directories created: {}", stats.moved); // Using moved count for created
    println!("Errors: {}", stats.errors);
    println!("Skipped: {}", stats.skipped);

    Ok(())
}

fn run_touch_command(args: &Args, files: &[String]) -> Result<(), Box<dyn Error>> {
    let config = build_file_op_config(args);

    println!("\n{}", "CNP Smart Move - Create/Touch Files".bold());
    println!("Files: {}", files.join(", ").cyan());
    println!(
        "Verbose: {}",
        if config.verbose {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Perform the touch operation first
    let stats = file_ops::create_files(files, config.verbose, None, None)?;

    // Initialize history manager for undo support
    let backup_dir = home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("smv")
        .join("backups");
    fs::create_dir_all(&backup_dir)?;
    let mut history_manager = HistoryManager::new(args.max_history_size, &backup_dir);

    // Record only successfully created files for undo support
    for file_path in files {
        let path = PathBuf::from(file_path);
        if path.exists() {
            // For file creation, source is empty path (nothing) and destination is the new file
            let empty_source = PathBuf::new(); // Represents "created from nothing"
            history_manager.record(empty_source, path)?;
        }
    }

    println!("\n{}", "Results:".bold());
    println!("Files processed: {}", stats.processed);
    println!("Files created/touched: {}", stats.moved); // Using moved count for created/touched
    println!("Errors: {}", stats.errors);
    println!("Skipped: {}", stats.skipped);

    Ok(())
}

fn run_copy_command(
    args: &Args,
    sources: &[String],
    destination: &str,
) -> Result<(), Box<dyn Error>> {
    let config = build_file_op_config(args);

    println!("\n{}", "CNP Smart Move - Copy Operation".bold());
    println!("Sources: {}", sources.join(", ").cyan());
    println!("Destination: {}", destination.cyan());
    println!(
        "Recursive: {}",
        if config.recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Force: {}",
        if config.force {
            "Yes".red()
        } else {
            "No".green()
        }
    );
    println!(
        "No-clobber: {}",
        if config.no_clobber {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Interactive: {}",
        if config.interactive {
            "Yes".cyan()
        } else {
            "No".yellow()
        }
    );
    println!(
        "Preserve metadata: {}",
        if config.preserve_metadata {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Expand glob patterns
    let expanded_sources = expand_glob_patterns(sources)?;
    let dest_path = Path::new(destination);

    // Execute copy operation
    let stats = copy_files(&expanded_sources, dest_path, &config)?;

    // Print results
    println!("\n{}:", "Results".bold());
    println!("Files processed: {}", stats.processed.to_string().cyan());
    println!("Files copied: {}", stats.copied.to_string().green());
    println!("Errors: {}", stats.errors.to_string().red());
    println!("Skipped: {}", stats.skipped.to_string().yellow());

    Ok(())
}

fn build_file_op_config(args: &Args) -> FileOpConfig {
    FileOpConfig {
        recursive: args.recursive,
        force: args.force,
        no_clobber: args.no_clobber,
        interactive: args.interactive_confirm,
        preserve_metadata: args.preserve,
        dereference_symlinks: args.dereference,
        follow_symlinks: !args.no_follow,
        verbose: args.verbose,
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
    // Get target directory or pattern (default to current directory)
    let target = args.target.as_deref().unwrap_or(".");

    // Detect if target is a glob pattern or directory
    let is_glob_pattern = target.contains('*') || target.contains('?') || target.contains('[');

    let (directory, pattern) = if is_glob_pattern {
        // Extract directory and pattern from glob
        let path = Path::new(target);
        if let Some(parent) = path.parent() {
            (
                parent.to_string_lossy().to_string(),
                Some(target.to_string()),
            )
        } else {
            (".".to_string(), Some(target.to_string()))
        }
    } else {
        // Validate directory exists
        if !Path::new(target).exists() {
            return Err(format!("Directory does not exist: {target}").into());
        }
        (target.to_string(), None)
    };

    // Get extensions from args (legacy support)
    let extensions = if args.args.is_empty() {
        None
    } else {
        // Filter args to only include file extensions (not CNP keywords)
        let ext_args: Vec<String> = args
            .args
            .iter()
            .filter(|arg| {
                !arg.contains(':')
                    && !arg.starts_with('-')
                    && !arg.starts_with("SIZE")
                    && !arg.starts_with("DEPTH")
                    && !arg.starts_with("MODIFIED")
                    && !arg.starts_with("ACCESSED")
            })
            .cloned()
            .collect();
        if ext_args.is_empty() {
            None
        } else {
            Some(ext_args)
        }
    };

    // Process exclude patterns
    let exclude_patterns: Vec<regex::Regex> = process_exclude_patterns(args.exclude.as_deref())?;

    // Print operation mode
    println!(
        "\n{}",
        format!(
            "CNP Smart Move - {} Mode",
            if args.preview { "Preview" } else { "Transform" }
        )
        .bold()
    );
    println!("Transformation: {}", transform_type.as_str().green());

    if let Some(ref pat) = pattern {
        println!("Pattern: {}", pat.cyan());
        println!("Base Directory: {}", directory.cyan());
    } else {
        println!("Directory: {}", directory.cyan());
    }

    println!(
        "Extensions: {}",
        match &extensions {
            Some(exts) => exts.join(", ").cyan(),
            None => "All files".yellow(),
        }
    );
    println!(
        "Recursive: {}",
        if args.recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Build file list - use DSC for glob patterns, fallback to original for directories
    let files = if let Some(pattern_str) = pattern {
        println!("Using DSC for pattern matching...");
        build_file_list_with_dsc(
            &pattern_str,
            &extensions,
            args.recursive,
            &exclude_patterns,
            args.hidden,
        )?
    } else {
        build_file_list(
            &directory,
            &extensions,
            args.recursive,
            &exclude_patterns,
            args.hidden,
        )?
    };

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

/// Check if a path or any of its parent directories is hidden
fn is_path_or_parent_hidden(path: &std::path::Path, base_dir: &std::path::Path) -> bool {
    // Get the relative path from the base directory
    if let Ok(relative_path) = path.strip_prefix(base_dir) {
        // Check each component of the relative path
        for component in relative_path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if name.starts_with('.') {
                    return true;
                }
            }
        }
    } else {
        // If we can't get a relative path, check each component of the full path
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if name.starts_with('.') {
                    return true;
                }
            }
        }
    }
    false
}

/// Build list of files and directories to process based on directory and extensions
fn build_file_list(
    directory: &str,
    extensions: &Option<Vec<String>>,
    recursive: bool,
    exclude_patterns: &[regex::Regex],
    include_hidden: bool,
) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
    use walkdir::WalkDir;

    let mut items = Vec::new();
    let base_dir = std::path::Path::new(directory);
    let walker = if recursive {
        WalkDir::new(directory)
    } else {
        WalkDir::new(directory).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(Result::ok) {
        let path = entry.path();

        // Skip the root directory itself to avoid self-transformation
        if path == base_dir {
            continue;
        }

        // Check for hidden files/directories recursively (skip if not including hidden)
        if !include_hidden && is_path_or_parent_hidden(path, base_dir) {
            continue;
        }

        // For extension filtering, only apply to files (directories don't have extensions)
        if path.is_file() {
            // Check extensions if specified
            if let Some(target_extensions) = extensions {
                if let Some(file_ext) = path.extension() {
                    let file_ext_str = file_ext.to_string_lossy().to_lowercase();
                    if !target_extensions
                        .iter()
                        .any(|ext| ext.to_lowercase() == file_ext_str)
                    {
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
        if exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
        {
            continue;
        }

        items.push(path.to_path_buf());
    }

    Ok(items)
}

/// Build list of files using DSC for pattern matching and discovery
fn build_file_list_with_dsc(
    pattern: &str,
    extensions: &Option<Vec<String>>,
    recursive: bool,
    exclude_patterns: &[regex::Regex],
    include_hidden: bool,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    // Determine the base path and pattern
    let (base_path, file_pattern) = if pattern.contains('/') {
        // Extract directory and pattern components
        let path = Path::new(pattern);
        if let Some(parent) = path.parent() {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "*".to_string());
            (parent.to_string_lossy().to_string(), file_name)
        } else {
            (".".to_string(), pattern.to_string())
        }
    } else {
        (".".to_string(), pattern.to_string())
    };

    // Build DSC command
    let mut dsc_cmd = Command::new("dsc");
    dsc_cmd.arg(&base_path);
    dsc_cmd.arg("--paths"); // Get file paths only
    dsc_cmd.arg("--glob"); // Use glob pattern matching
    dsc_cmd.arg(&file_pattern); // Pass the pattern as trailing argument
    dsc_cmd.stdout(Stdio::piped());

    if !recursive {
        // DSC doesn't have exact max_depth=1, but we can filter afterwards
        // For now, let DSC recurse and we'll filter
    }

    // Execute DSC as subprocess
    let mut child = dsc_cmd.spawn().map_err(|e| {
        format!("Failed to spawn DSC process: {e}. Make sure DSC is installed and in PATH.")
    })?;

    let stdout = child.stdout.take().ok_or("Failed to capture DSC stdout")?;

    let reader = BufReader::new(stdout);
    let mut items = Vec::new();

    // DSC handles the pattern matching, so we just need to parse its output

    // Read DSC output and filter
    for line in reader.lines() {
        let line = line?;
        let path = PathBuf::from(line.trim());

        // Skip if not a file or directory
        if !path.exists() {
            continue;
        }

        // DSC already applied pattern matching, so no need to filter again

        // Check for hidden files/directories recursively (skip if not including hidden)
        if !include_hidden {
            let base_path_obj = Path::new(&base_path);
            if is_path_or_parent_hidden(&path, base_path_obj) {
                continue;
            }
        }

        // Apply recursive filtering if needed
        if !recursive {
            let base_path_obj = Path::new(&base_path);
            if let Ok(relative) = path.strip_prefix(base_path_obj) {
                // Skip if in subdirectory (more than one component)
                if relative.components().count() > 1 {
                    continue;
                }
            }
        }

        // Apply extension filtering for files
        if path.is_file() {
            if let Some(target_extensions) = extensions {
                if let Some(file_ext) = path.extension() {
                    let file_ext_str = file_ext.to_string_lossy().to_lowercase();
                    if !target_extensions
                        .iter()
                        .any(|ext| ext.to_lowercase() == file_ext_str)
                    {
                        continue;
                    }
                } else {
                    // File has no extension, skip if extensions were specified
                    continue;
                }
            }
        }

        // Apply exclude patterns
        let path_str = path.to_string_lossy();
        if exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
        {
            continue;
        }

        items.push(path);
    }

    // Wait for DSC to complete
    let output = child.wait()?;
    if !output.success() {
        return Err(format!("DSC process failed with exit code: {:?}", output.code()).into());
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
    // Check if the item still exists (it might have been moved already)
    if !item_path.exists() {
        // Skip silently - this item was likely moved as part of a parent directory rename
        return Ok(());
    }

    let item_name = item_path
        .file_name()
        .ok_or("Invalid item name")?
        .to_string_lossy();

    let new_name = transform(&item_name, transform_type);

    stats.processed += 1;

    // If name unchanged, nothing to do
    if new_name == item_name {
        return Ok(());
    }

    let new_path = item_path
        .parent()
        .ok_or("Invalid parent directory")?
        .join(&new_name);

    // Check for conflicts
    if new_path.exists() && item_path != new_path {
        let item_type = if item_path.is_dir() {
            "directory"
        } else {
            "file"
        };
        println!(
            "{}Conflict: {} \"{}\" → \"{}\" (target exists)",
            if preview_only { "[PREVIEW] " } else { "" },
            item_type,
            item_name,
            new_name
        );
        stats.errors += 1;
        return Ok(());
    }

    // Log the operation
    let item_type = if item_path.is_dir() {
        "directory"
    } else {
        "file"
    };
    println!(
        "{}Rename {}: \"{}\" → \"{}\"",
        if preview_only { "[PREVIEW] " } else { "" },
        item_type,
        item_name,
        new_name
    );

    if !preview_only {
        // Double-check the item still exists before renaming
        if !item_path.exists() {
            // Item was moved/renamed by a previous operation, skip silently
            return Ok(());
        }
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
        println!(
            "\n{}",
            "This was a preview only. No items were actually renamed."
                .bold()
                .blue()
        );
        println!(
            "{}",
            "To apply these changes, run the same command without --preview.".blue()
        );
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
        }
        SortMethod::Flatten => {
            println!(
                "\n{}\n",
                "CNP Smart Move - Flatten Directory Structure".bold()
            );
            println!("Processing directory: {}", directory.cyan());
            unsort::flatten_directory(directory, args.preview)?;

            // Also remove empty directories
            println!("\nRemoving empty directories:");
            unsort::remove_empty_dirs(directory, args.preview)?
        }
        SortMethod::ByType => {
            println!("Sort by type not yet implemented.");
        }
        SortMethod::ByDate => {
            println!("Sort by date not yet implemented.");
        }
        SortMethod::BySize => {
            println!("Sort by size not yet implemented.");
        }
    }

    if args.preview {
        println!(
            "\n{}",
            "This was a preview only. No files were actually moved."
                .bold()
                .blue()
        );
        println!(
            "{}",
            "To apply these changes, run the same command without the -p flag.".blue()
        );
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

/// Check if we should use CNP grammar parsing instead of legacy syntax
fn should_use_cnp_grammar(args: &Args) -> bool {
    // Collect all arguments to check for CNP keywords
    let mut all_args = Vec::new();

    if let Some(ref cmd) = args.command {
        all_args.push(cmd.as_str());
    }
    if let Some(ref arg1) = args.arg1 {
        all_args.push(arg1.as_str());
    }
    if let Some(ref into_kw) = args.into_keyword {
        all_args.push(into_kw.as_str());
    }
    if let Some(ref arg2) = args.arg2 {
        all_args.push(arg2.as_str());
    }
    if let Some(ref target) = args.target {
        all_args.push(target.as_str());
    }

    for arg in &args.args {
        all_args.push(arg.as_str());
    }

    // Check for CNP keywords
    let has_cnp_keywords = all_args.iter().any(|arg| {
        arg.contains(':')
            || arg.starts_with("SIZE")
            || arg.starts_with("DEPTH")
            || arg.starts_with("MODIFIED")
            || arg.starts_with("ACCESSED")
            || *arg == "WHERE"
            || matches!(*arg, "TO" | "INTO" | "FORMAT")
    });

    // Check for glob patterns that should use CNP parsing
    let has_glob_patterns = all_args.iter().any(|arg| {
        arg.contains('*') || arg.contains('?') || arg.contains('[') || arg.contains('{')
    });

    // Special case: rm command with CNP filters
    let has_rm_with_filters = args.command.as_deref() == Some("rm") && has_cnp_keywords;

    has_cnp_keywords || has_glob_patterns || has_rm_with_filters
}

/// Run CNP grammar command
fn run_cnp_command(args: &Args) -> Result<(), Box<dyn Error>> {
    // Build arguments for CNP parser
    let mut cnp_args = Vec::new();

    // Add command if present
    if let Some(ref cmd) = args.command {
        cnp_args.push(cmd.clone());
    }
    if let Some(ref arg1) = args.arg1 {
        cnp_args.push(arg1.clone());
    }
    if let Some(ref into_kw) = args.into_keyword {
        cnp_args.push(into_kw.clone());
    }
    if let Some(ref arg2) = args.arg2 {
        cnp_args.push(arg2.clone());
    }
    if let Some(ref target) = args.target {
        cnp_args.push(target.clone());
    }

    // Add trailing arguments
    cnp_args.extend(args.args.iter().cloned());

    // Add flags as arguments
    if args.recursive {
        cnp_args.push("-r".to_string());
    }
    if args.preview {
        cnp_args.push("-p".to_string());
    }
    if args.force {
        cnp_args.push("-f".to_string());
    }
    if args.case_insensitive {
        cnp_args.push("-i".to_string());
    }
    if args.ignore_case {
        cnp_args.push("-ic".to_string());
    }
    if args.interactive {
        cnp_args.push("-I".to_string());
    }
    if args.tui {
        cnp_args.push("-T".to_string());
    }
    if args.undo {
        cnp_args.push("-u".to_string());
    }

    // Parse CNP command
    let cnp_command = CnpGrammarParser::parse(&cnp_args)?;

    // Handle special flags first
    if args.interactive || cnp_command.flags.contains('I') {
        return run_interactive_mode(args.max_history_size);
    }
    if args.tui || cnp_command.flags.contains('T') {
        return run_tui_mode();
    }
    if args.undo || cnp_command.flags.contains('u') {
        return run_undo_mode(args.max_history_size);
    }

    // Handle routes (tool delegation)
    if let Some(route) = cnp_command.routes.first() {
        match route {
            cnp_grammar::Route::To { tool, args } => {
                return run_tool_delegation(&cnp_command, tool, args);
            }
            cnp_grammar::Route::Into(file) => {
                return run_output_to_file(&cnp_command, file);
            }
            cnp_grammar::Route::Format(format) => {
                return run_formatted_output(&cnp_command, format);
            }
        }
    }

    // Handle transform command
    if let Some(ref transform_cmd) = cnp_command.transform_command {
        let transform_type = match transform_cmd.command_type.as_str() {
            "change" => {
                let old = transform_cmd
                    .old_value
                    .as_ref()
                    .ok_or("Missing old value for CHANGE")?;
                let new = transform_cmd
                    .new_value
                    .as_ref()
                    .ok_or("Missing new value for CHANGE")?;
                TransformType::replace(old, new)
            }
            "regex" => {
                let pattern = transform_cmd
                    .old_value
                    .as_ref()
                    .ok_or("Missing pattern for REGEX")?;
                let replacement = transform_cmd
                    .new_value
                    .as_ref()
                    .ok_or("Missing replacement for REGEX")?;
                TransformType::replace_regex(pattern, replacement)
            }
            "snake" => TransformType::Snake,
            "kebab" => TransformType::Kebab,
            "pascal" => TransformType::Pascal,
            "camel" => TransformType::Camel,
            "title" => TransformType::Title,
            "lower" => TransformType::Lower,
            "upper" => TransformType::Upper,
            "clean" => TransformType::Clean,
            _ => {
                return Err(
                    format!("Unknown transform command: {}", transform_cmd.command_type).into(),
                );
            }
        };

        return run_cnp_transform_command(&cnp_command, transform_type);
    }

    // Handle remove command
    if let Some(ref remove_cmd) = cnp_command.remove_command {
        return run_cnp_remove_command(&cnp_command);
    }

    Err("No valid CNP command found".into())
}

/// Run transform command with CNP grammar
fn run_cnp_transform_command(
    cnp_command: &CnpCommand,
    transform_type: TransformType,
) -> Result<(), Box<dyn Error>> {
    let path = &cnp_command.path;
    let recursive = cnp_command.flags.contains('r');
    let preview = cnp_command.flags.contains('p');

    // Expand semantic groups
    let expanded_filters =
        cnp_grammar::CnpGrammarParser::expand_semantic_groups(&cnp_command.filters);

    println!(
        "\n{}",
        format!(
            "CNP Smart Move - {} Mode",
            if preview { "Preview" } else { "Transform" }
        )
        .bold()
    );
    println!("Transformation: {}", transform_type.as_str().green());
    println!("Path: {}", path.cyan());
    println!(
        "Filters: {} active",
        expanded_filters.len().to_string().cyan()
    );
    println!(
        "Recursive: {}",
        if recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );
    println!();

    // Build file list based on CNP filters
    let include_hidden = cnp_command.flags.contains('a');
    let files = build_cnp_file_list(
        path,
        &expanded_filters,
        recursive,
        include_hidden,
        cnp_command.case_insensitive,
    )?;

    if files.is_empty() {
        println!("No files found matching CNP filter criteria.");
        return Ok(());
    }

    // Process files for transformation
    let mut stats = Stats::default();
    for item_path in files {
        process_item_transformation(&item_path, &transform_type, preview, &mut stats)?;
    }

    // Print results
    print_transformation_results(&stats, preview);

    Ok(())
}

/// Build file list based on CNP filters
fn build_cnp_file_list(
    path: &str,
    filters: &[cnp_grammar::Filter],
    recursive: bool,
    include_hidden: bool,
    case_insensitive: bool,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    use cnp_grammar::{FileType, Filter};
    use walkdir::WalkDir;

    let mut items = Vec::new();
    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(Result::ok) {
        let entry_path = entry.path();

        // Skip the root directory itself
        if entry_path == std::path::Path::new(path) {
            continue;
        }

        // Check for hidden files/directories recursively (skip if not including hidden)
        if !include_hidden {
            let base_path_obj = std::path::Path::new(path);
            if is_path_or_parent_hidden(entry_path, base_path_obj) {
                continue;
            }
        }

        // Apply CNP filters
        let mut matches = true;

        for filter in filters {
            match filter {
                Filter::Name(name) => {
                    if let Some(filename) = entry_path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        let match_result =
                            if name.contains('*') || name.contains('?') || name.contains('[') {
                                // Glob pattern matching
                                let pattern = if case_insensitive {
                                    glob::Pattern::new(&name.to_lowercase())?
                                } else {
                                    glob::Pattern::new(name)?
                                };
                                let test_str = if case_insensitive {
                                    filename_str.to_lowercase()
                                } else {
                                    filename_str.to_string()
                                };
                                pattern.matches(&test_str)
                            } else {
                                // Substring matching
                                if case_insensitive {
                                    filename_str.to_lowercase().contains(&name.to_lowercase())
                                } else {
                                    filename_str.contains(name)
                                }
                            };

                        if !match_result {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                }
                Filter::Type(file_type) => {
                    let entry_matches = match file_type {
                        FileType::File => entry_path.is_file(),
                        FileType::Folder => entry_path.is_dir(),
                        FileType::Symlink => entry_path.is_symlink(),
                        FileType::Other => {
                            !entry_path.is_file()
                                && !entry_path.is_dir()
                                && !entry_path.is_symlink()
                        }
                    };
                    if !entry_matches {
                        matches = false;
                        break;
                    }
                }
                Filter::Extension(ext) => {
                    if let Some(entry_ext) = entry_path.extension() {
                        if entry_ext.to_string_lossy().to_lowercase() != ext.to_lowercase() {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                }
                cnp_grammar::Filter::SizeGreater(size_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(size_bytes) = parse_size_string(size_str) {
                            if metadata.len() <= size_bytes {
                                matches = false;
                                break;
                            }
                        }
                    }
                }
                cnp_grammar::Filter::SizeLess(size_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(size_bytes) = parse_size_string(size_str) {
                            if metadata.len() >= size_bytes {
                                matches = false;
                                break;
                            }
                        }
                    }
                }
                cnp_grammar::Filter::DepthGreater(max_depth) => {
                    let entry_depth = entry_path.components().count();
                    let base_depth = std::path::Path::new(path).components().count();
                    let relative_depth = entry_depth.saturating_sub(base_depth);
                    if relative_depth <= *max_depth {
                        matches = false;
                        break;
                    }
                }
                cnp_grammar::Filter::DepthLess(min_depth) => {
                    let entry_depth = entry_path.components().count();
                    let base_depth = std::path::Path::new(path).components().count();
                    let relative_depth = entry_depth.saturating_sub(base_depth);
                    if relative_depth >= *min_depth {
                        matches = false;
                        break;
                    }
                }
                cnp_grammar::Filter::ModifiedAfter(date_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(target_time) = parse_date_string(date_str) {
                                if modified <= target_time {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                cnp_grammar::Filter::ModifiedBefore(date_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(target_time) = parse_date_string(date_str) {
                                if modified >= target_time {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                cnp_grammar::Filter::AccessedAfter(date_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(accessed) = metadata.accessed() {
                            if let Ok(target_time) = parse_date_string(date_str) {
                                if accessed <= target_time {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                cnp_grammar::Filter::AccessedBefore(date_str) => {
                    if let Ok(metadata) = entry_path.metadata() {
                        if let Ok(accessed) = metadata.accessed() {
                            if let Ok(target_time) = parse_date_string(date_str) {
                                if accessed >= target_time {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                cnp_grammar::Filter::Tag(_tag) => {
                    // Tag filtering would require integration with file tagging system
                    // For now, skip tags
                    continue;
                }
                cnp_grammar::Filter::Hash(_hash) => {
                    // Hash filtering would require file hash computation
                    // For now, skip hash filters
                    continue;
                }
                cnp_grammar::Filter::Where(_sub_filters) => {
                    // WHERE filters should be expanded during parsing
                    // For now, skip WHERE groups
                    continue;
                }
                cnp_grammar::Filter::For(_semantic_group) => {
                    // FOR filters should be expanded by semantic group expansion
                    // If we encounter one here, it means expansion didn't work properly
                    // Skip it for now
                    continue;
                }
            }
        }

        if matches {
            items.push(entry_path.to_path_buf());
        }
    }

    Ok(items)
}

/// Handle tool delegation
fn run_tool_delegation(
    cnp_command: &CnpCommand,
    tool: &str,
    additional_args: &[String],
) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    println!("Delegating to tool: {}", tool.cyan());

    // Build the file list first using current filters
    let expanded_filters =
        cnp_grammar::CnpGrammarParser::expand_semantic_groups(&cnp_command.filters);
    let recursive = cnp_command.flags.contains('r');
    let include_hidden = cnp_command.flags.contains('a');
    let files = build_cnp_file_list(
        &cnp_command.path,
        &expanded_filters,
        recursive,
        include_hidden,
        cnp_command.case_insensitive,
    )?;

    if files.is_empty() {
        println!("No files found to delegate to {tool}");
        return Ok(());
    }

    // Create subprocess for tool delegation
    let mut cmd = Command::new(tool);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Add base arguments based on the tool
    match tool {
        "say" => {
            // For SAY tool, we need to pass the operation type
            if let Some(ref transform_cmd) = cnp_command.transform_command {
                match transform_cmd.command_type.as_str() {
                    "snake" | "kebab" | "pascal" | "camel" | "title" | "lower" | "upper"
                    | "clean" => {
                        cmd.arg(&transform_cmd.command_type);
                    }
                    _ => {
                        cmd.arg("split_and_titlecase"); // Default SAY operation
                    }
                }
            } else {
                cmd.arg("split_and_titlecase");
            }
        }
        "dff" => {
            cmd.arg("find_duplicates");
        }
        "xfd" => {
            cmd.arg("interactive_select");
        }
        "dsc" => {
            cmd.arg(&cnp_command.path);
            cmd.arg("--paths");
        }
        _ => {
            // Generic tool delegation
            cmd.arg(&cnp_command.path);
        }
    }

    // Add user-provided additional arguments
    if !additional_args.is_empty() {
        println!("Adding additional arguments: {additional_args:?}");
        for arg in additional_args {
            cmd.arg(arg);
        }
    }

    println!("Spawning {} with {} files...", tool, files.len());

    // Spawn the process
    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn {tool} process: {e}. Make sure {tool} is installed and in PATH.")
    })?;

    // Send file paths to the tool via stdin
    if let Some(mut stdin) = child.stdin.take() {
        for file_path in &files {
            writeln!(stdin, "{}", file_path.display())?;
        }
    }

    // Wait for completion and capture output
    let output = child.wait_with_output()?;

    if output.status.success() {
        if !output.stdout.is_empty() {
            println!("Tool output:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        println!(
            "Tool delegation to '{}' completed successfully.",
            tool.green()
        );
    } else {
        if !output.stderr.is_empty() {
            eprintln!("Tool error output:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        return Err(format!(
            "Tool '{}' failed with exit code: {:?}",
            tool,
            output.status.code()
        )
        .into());
    }

    Ok(())
}

/// Handle output to file
fn run_output_to_file(cnp_command: &CnpCommand, file: &str) -> Result<(), Box<dyn Error>> {
    use std::fs::File;
    use std::io::Write;

    println!("Writing output to file: {}", file.cyan());

    // Build the file list using current filters
    let expanded_filters =
        cnp_grammar::CnpGrammarParser::expand_semantic_groups(&cnp_command.filters);
    let recursive = cnp_command.flags.contains('r');
    let include_hidden = cnp_command.flags.contains('a');
    let files = build_cnp_file_list(
        &cnp_command.path,
        &expanded_filters,
        recursive,
        include_hidden,
        cnp_command.case_insensitive,
    )?;

    if files.is_empty() {
        println!("No files found to write to output file");
        return Ok(());
    }

    // Create output file
    let mut output_file = File::create(file)?;

    // Write header with command info
    writeln!(output_file, "# SMV CNP Output")?;
    writeln!(output_file, "# Command: {cnp_command:?}")?;
    writeln!(
        output_file,
        "# Generated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output_file, "# Path: {}", cnp_command.path)?;
    writeln!(output_file, "# Filters: {} active", expanded_filters.len())?;
    writeln!(output_file, "# Files found: {}", files.len())?;
    writeln!(output_file)?;

    // Write file paths
    for file_path in &files {
        writeln!(output_file, "{}", file_path.display())?;
    }

    println!(
        "Successfully wrote {} file paths to {}",
        files.len().to_string().green(),
        file.cyan()
    );
    Ok(())
}

/// Handle formatted output
fn run_formatted_output(
    cnp_command: &CnpCommand,
    format: &cnp_grammar::OutputFormat,
) -> Result<(), Box<dyn Error>> {
    use cnp_grammar::OutputFormat;

    println!("Formatting output as: {format:?}");

    // Build the file list using current filters
    let expanded_filters =
        cnp_grammar::CnpGrammarParser::expand_semantic_groups(&cnp_command.filters);
    let recursive = cnp_command.flags.contains('r');
    let include_hidden = cnp_command.flags.contains('a');
    let files = build_cnp_file_list(
        &cnp_command.path,
        &expanded_filters,
        recursive,
        include_hidden,
        cnp_command.case_insensitive,
    )?;

    if files.is_empty() {
        println!("No files found for formatted output");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            println!("{{");
            println!("  \"command\": {cnp_command:?},");
            println!("  \"path\": \"{}\",", cnp_command.path);
            println!("  \"filters_count\": {},", expanded_filters.len());
            println!("  \"files_found\": {},", files.len());
            println!("  \"files\": [");
            for (i, file_path) in files.iter().enumerate() {
                let comma = if i < files.len() - 1 { "," } else { "" };
                println!("    \"{}\"{}", file_path.display(), comma);
            }
            println!("  ]");
            println!("}}");
        }
        OutputFormat::Csv => {
            println!("path,type,size,modified");
            for file_path in &files {
                let metadata = file_path.metadata().unwrap_or_else(|_| {
                    // Create dummy metadata for error cases
                    std::fs::metadata(".").unwrap()
                });
                let file_type = if file_path.is_file() {
                    "file"
                } else if file_path.is_dir() {
                    "directory"
                } else {
                    "other"
                };
                let size = if file_path.is_file() {
                    metadata.len()
                } else {
                    0
                };
                let modified = metadata
                    .modified()
                    .map(|t| format!("{t:?}"))
                    .unwrap_or_else(|_| "unknown".to_string());

                println!(
                    "{},{},{},{}",
                    file_path.display(),
                    file_type,
                    size,
                    modified
                );
            }
        }
        OutputFormat::Yaml => {
            println!("command: {cnp_command:?}");
            println!("path: \"{}\"", cnp_command.path);
            println!("filters_count: {}", expanded_filters.len());
            println!("files_found: {}", files.len());
            println!("files:");
            for file_path in &files {
                println!("  - \"{}\"", file_path.display());
            }
        }
        OutputFormat::Text => {
            println!("SMV CNP Output");
            println!("==============");
            println!("Path: {}", cnp_command.path);
            println!("Filters: {} active", expanded_filters.len());
            println!("Files found: {}", files.len());
            println!();
            for file_path in &files {
                println!("{}", file_path.display());
            }
        }
    }

    Ok(())
}

/// Parse mode strings like "755", "644", "u+rw" into mode values
fn parse_mode_string(mode_str: &str) -> Result<Option<u32>, Box<dyn Error>> {
    if mode_str.chars().all(|c| c.is_ascii_digit()) {
        // Octal mode (e.g., "755", "644")
        let mode = u32::from_str_radix(mode_str, 8)?;
        if mode > 0o777 {
            return Err("Mode value must be between 000 and 777 (octal)".into());
        }
        Ok(Some(mode))
    } else {
        // Symbolic mode not implemented yet (e.g., "u+rw", "go-w")
        Err("Symbolic mode notation not yet implemented. Use octal notation (e.g., 755)".into())
    }
}

/// Parse size strings like "1MB", "500KB", "2GB" into bytes
fn parse_size_string(size_str: &str) -> Result<u64, Box<dyn Error>> {
    let size_str = size_str.to_uppercase();

    if let Some(num_str) = size_str.strip_suffix("B") {
        return Ok(num_str.parse::<u64>()?);
    }
    if let Some(num_str) = size_str.strip_suffix("KB") {
        return Ok(num_str.parse::<u64>()? * 1024);
    }
    if let Some(num_str) = size_str.strip_suffix("MB") {
        return Ok(num_str.parse::<u64>()? * 1024 * 1024);
    }
    if let Some(num_str) = size_str.strip_suffix("GB") {
        return Ok(num_str.parse::<u64>()? * 1024 * 1024 * 1024);
    }
    if let Some(num_str) = size_str.strip_suffix("TB") {
        return Ok(num_str.parse::<u64>()? * 1024 * 1024 * 1024 * 1024);
    }

    // If no suffix, assume bytes
    Ok(size_str.parse::<u64>()?)
}

/// Run remove command with CNP grammar
fn run_cnp_remove_command(cnp_command: &CnpCommand) -> Result<(), Box<dyn Error>> {
    let path = &cnp_command.path;
    let recursive = cnp_command.flags.contains('r');
    let preview = cnp_command.flags.contains('p');
    let force = cnp_command.flags.contains('f');
    let include_hidden = cnp_command.flags.contains('a');
    println!(
        "Debug: flags='{}', include_hidden={}",
        cnp_command.flags, include_hidden
    );

    // Expand semantic groups
    let expanded_filters =
        cnp_grammar::CnpGrammarParser::expand_semantic_groups(&cnp_command.filters);

    println!(
        "\n{}",
        format!(
            "CNP Smart Move - {} Remove Operation",
            if preview { "Preview" } else { "Execute" }
        )
        .bold()
    );
    println!("Path: {}", path.cyan());
    println!(
        "Filters: {} active",
        expanded_filters.len().to_string().cyan()
    );
    println!(
        "Recursive: {}",
        if recursive {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );

    if preview {
        println!(
            "{}",
            "PREVIEW MODE - No files will be deleted".bold().blue()
        );
    }
    println!();

    // Build file list based on CNP filters
    let include_hidden = cnp_command.flags.contains('a');
    let files = build_cnp_file_list(
        path,
        &expanded_filters,
        recursive,
        include_hidden,
        cnp_command.case_insensitive,
    )?;

    if files.is_empty() {
        println!("No files found matching CNP filter criteria.");
        return Ok(());
    }

    // Show what will be deleted
    println!("Files to be deleted:");
    for file_path in &files {
        let file_type = if file_path.is_file() {
            "file"
        } else if file_path.is_dir() {
            "directory"
        } else {
            "other"
        };

        if preview {
            println!(
                "{} [{}] {}",
                "[PREVIEW]".blue().bold(),
                file_type,
                file_path.display()
            );
        } else {
            println!("{} {}", file_type, file_path.display());
        }
    }

    if preview {
        println!(
            "\n{} {} files would be deleted.",
            "[PREVIEW]".blue().bold(),
            files.len()
        );
        println!("Use without -p flag to actually delete these files.");
        return Ok(());
    }

    // Safety check - require confirmation for dangerous operations
    if !force && !ask_for_confirmation(&files)? {
        println!("Operation cancelled.");
        return Ok(());
    }

    // Build config for file operations
    let config = FileOpConfig {
        recursive,
        force,
        no_clobber: false,
        interactive: false,
        preserve_metadata: false,
        dereference_symlinks: false,
        follow_symlinks: false,
        verbose: true,
    };

    // Perform the removal
    let stats = file_ops::remove_files(&files, &config)?;

    println!("\n{}", "Results:".bold());
    println!("Files processed: {}", stats.processed);
    println!("Files removed: {}", stats.moved); // Using moved count for removed
    println!("Errors: {}", stats.errors);
    println!("Skipped: {}", stats.skipped);

    Ok(())
}

/// Ask for user confirmation before deleting files
fn ask_for_confirmation(files: &[PathBuf]) -> Result<bool, Box<dyn Error>> {
    use std::io::{self, Write};

    if files.len() > 10 {
        println!(
            "\n{} You are about to delete {} files.",
            "WARNING:".red().bold(),
            files.len()
        );
    } else {
        println!(
            "\n{} You are about to delete these {} files.",
            "WARNING:".red().bold(),
            files.len()
        );
    }

    print!("Are you sure you want to continue? (type 'yes' to confirm): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response == "yes")
}

/// Parse date strings like "2024-01-01", "2023-12-25" into SystemTime
fn parse_date_string(date_str: &str) -> Result<std::time::SystemTime, Box<dyn Error>> {
    use std::time::{Duration, UNIX_EPOCH};

    // Simple date parsing for YYYY-MM-DD format
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err("Date must be in YYYY-MM-DD format".into());
    }

    let year: u32 = parts[0].parse()?;
    let month: u32 = parts[1].parse()?;
    let day: u32 = parts[2].parse()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("Invalid date values".into());
    }

    // Simple approximation: convert to days since epoch
    let days_since_epoch = (year as u64 - 1970) * 365 + (month as u64 - 1) * 30 + day as u64;
    let seconds_since_epoch = days_since_epoch * 24 * 60 * 60;

    Ok(UNIX_EPOCH + Duration::from_secs(seconds_since_epoch))
}
