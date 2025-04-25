//! SMV - Smart Move CLI
//!
//! Command-line interface for the Smart Move utility.

use std::fs;
use std::path::PathBuf;
use std::process;
use std::error::Error;

use clap::{App, Arg, ArgMatches};
use colored::*;

use smv::transformers::TransformType;
use smv::repl::InteractiveSession;

/// Command-line arguments for SMV
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

    // Show help information and exit
    println!("SMV - Smart Move utility");
    println!("Use --help for usage information");
    
    Ok(())
}

/// Launch the interactive REPL session
fn run_interactive_mode(max_history_size: usize) -> Result<(), Box<dyn Error>> {
    // Setup backup directory
    let backup_dir = dirs::home_dir()
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