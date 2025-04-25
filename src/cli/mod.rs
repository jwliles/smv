//! Command-line interface for SMV
//!
//! This module handles the CLI argument parsing and dispatching to the appropriate
//! functionality.

use clap::{App, Arg, ArgMatches};

/// Command-line arguments for SMV
pub struct Args {
    /// Files or patterns to move/rename
    pub source: Vec<String>,

    /// Destination file or directory
    pub destination: Option<String>,

    /// Interactive mode - launch REPL interface
    pub interactive: bool,

    /// Preview changes without applying them
    pub preview: bool,

    /// Process subdirectories recursively
    pub recursive: bool,

    /// Comma-separated list of file extensions to process
    pub extensions: Option<String>,

    /// Remove accents
    pub remove_accents: bool,

    /// Convert to clean format (remove special chars, normalize spaces)
    pub clean: bool,

    /// Convert to snake_case
    pub snake: bool,

    /// Convert to kebab-case
    pub kebab: bool,

    /// Convert to Title Case
    pub title: bool,

    /// Convert to camelCase
    pub camel: bool,

    /// Convert to PascalCase
    pub pascal: bool,

    /// Convert to lowercase
    pub lower: bool,

    /// Convert to UPPERCASE
    pub upper: bool,

    /// Same as preview - show what would change without making changes
    pub dry_run: bool,

    /// Comma-separated patterns to exclude
    pub exclude: Option<String>,

    /// Maximum number of operations to keep in history
    pub max_history_size: usize,
}

impl Args {
    /// Parse command line arguments
    pub fn parse() -> Self {
        let matches = App::new("smv")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Smart Move - An enhanced mv command with transformation capabilities")
            // Source files
            .arg(Arg::with_name("SOURCE")
                .multiple(true)
                .help("Files or patterns to move/rename"))
            // Destination
            .arg(Arg::with_name("DESTINATION")
                .help("Destination file or directory"))
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

    /// Create Args from ArgMatches
    fn from_arg_matches(matches: &ArgMatches) -> Self {
        let source = match matches.values_of("SOURCE") {
            Some(values) => values.map(String::from).collect(),
            None => Vec::new(),
        };

        let destination = matches.value_of("DESTINATION").map(String::from);

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

/// Check if any transformation options are enabled
pub fn is_transformation_requested(args: &Args) -> bool {
    args.clean || args.snake || args.kebab || args.title || 
    args.camel || args.pascal || args.lower || args.upper
}