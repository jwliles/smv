use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use colored::*;
use glob::glob;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::Helper;
use rustyline::{CompletionType, Config, Editor, Result as RustylineResult};

use crate::history::HistoryManager;
use crate::transformers::{transform, TransformType};

// Custom command completer
struct CommandCompleter {
    commands: Vec<String>,
    file_completer: FilenameCompleter,
}

impl CommandCompleter {
    fn new() -> Self {
        let commands = vec![
            "preview".to_string(),
            "apply".to_string(),
            "undo".to_string(),
            "cd".to_string(),
            "ls".to_string(),
            "rename".to_string(),
            "help".to_string(),
            "quit".to_string(),
            "exit".to_string(),
            "clean".to_string(),
            "snake".to_string(),
            "kebab".to_string(),
            "title".to_string(),
            "camel".to_string(),
            "pascal".to_string(),
            "lower".to_string(),
            "upper".to_string(),
        ];

        Self {
            commands,
            file_completer: FilenameCompleter::new(),
        }
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        // Split line into words
        let words: Vec<&str> = line[..pos].split_whitespace().collect();

        // If we're on the first word, complete commands
        if words.len() <= 1 {
            let word = words.first().map_or("", |w| *w);
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();

            return Ok((0, matches));
        }

        // Otherwise, complete filenames
        self.file_completer.complete(line, pos, ctx)
    }
}

// Helper functions for rustyline integration
impl Hinter for CommandCompleter {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CommandCompleter {}
impl Validator for CommandCompleter {}
impl Helper for CommandCompleter {}

/// Interactive REPL for SMV
pub struct InteractiveSession {
    editor: Editor<CommandCompleter, DefaultHistory>,
    history_manager: HistoryManager,
    current_dir: PathBuf,
}

impl InteractiveSession {
    pub fn new(max_history_size: usize, backup_dir: &Path) -> Result<Self, Box<dyn Error>> {
        // Create a rustyline editor with custom configuration
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut editor = Editor::with_config(config)?;

        // Set the helper for completion
        let helper = CommandCompleter::new();
        editor.set_helper(Some(helper));

        // Set the current directory
        let current_dir = env::current_dir()?;

        // Create history manager
        let history_manager = HistoryManager::new(max_history_size, backup_dir);

        Ok(Self {
            editor,
            history_manager,
            current_dir,
        })
    }

    /// Run the REPL session
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.display_welcome();

        loop {
            // Display prompt with current directory
            let prompt = format!("smv:{}> ", self.current_dir.display());

            // Read a line of input
            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Process the command
                    if let Err(e) = self.execute_command(line) {
                        eprintln!("{}: {}", "Error".red(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Execute a REPL command
    fn execute_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "preview" => self.cmd_preview(&parts[1..]),
            "apply" => self.cmd_apply(&parts[1..]),
            "undo" => self.cmd_undo(),
            "cd" => self.cmd_cd(&parts[1..]),
            "ls" => self.cmd_ls(&parts[1..]),
            "rename" => self.cmd_rename(&parts[1..]),
            "help" => self.cmd_help(),
            "quit" | "exit" => {
                println!("Goodbye!");
                process::exit(0);
            }
            _ => {
                // Check if the command is a transformation type
                if let Some(transform_type) = TransformType::from_str(parts[0]) {
                    if parts.len() > 1 {
                        // Use as transformation with file pattern
                        self.preview_transform(transform_type, &parts[1..])
                    } else {
                        eprintln!("Usage: {} <file_pattern>", transform_type.as_str());
                        Ok(())
                    }
                } else {
                    Err(format!("Unknown command: {}", parts[0]).into())
                }
            }
        }
    }

    /// Display welcome message
    fn display_welcome(&self) {
        println!("{}", "SMV - Smart Move".bold().green());
        println!("Type {} for available commands", "help".cyan());
    }

    /// Display help text
    fn cmd_help(&self) -> Result<(), Box<dyn Error>> {
        println!("{}:", "Commands".bold());
        println!(
            "  {} <transform> <files>  - Show transformation without applying",
            "preview".cyan()
        );
        println!(
            "  {} <transform> <files>    - Apply transformation",
            "apply".cyan()
        );
        println!(
            "  {} <files> --options     - Interactive renaming wizard",
            "rename".cyan()
        );
        println!(
            "  {}                        - Revert last operation",
            "undo".cyan()
        );
        println!(
            "  {} <directory>              - Change directory",
            "cd".cyan()
        );
        println!("  {} [pattern]                - List files", "ls".cyan());
        println!(
            "  {}                        - Show this help",
            "help".cyan()
        );
        println!("  {}                        - Exit program", "quit".cyan());
        println!();
        println!("{}:", "Transformations".bold());
        println!(
            "  {} - Clean up spaces and special characters",
            "clean".yellow()
        );
        println!("  {} - Convert to snake_case", "snake".yellow());
        println!("  {} - Convert to kebab-case", "kebab".yellow());
        println!("  {} - Convert to Title Case", "title".yellow());
        println!("  {} - Convert to camelCase", "camel".yellow());
        println!("  {} - Convert to PascalCase", "pascal".yellow());
        println!("  {} - Convert to lowercase", "lower".yellow());
        println!("  {} - Convert to UPPERCASE", "upper".yellow());

        Ok(())
    }

    /// Change current directory
    fn cmd_cd(&mut self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if args.is_empty() {
            // Default to home directory if no args
            let home = dirs::home_dir().ok_or("Could not determine home directory")?;
            self.current_dir = home;
        } else {
            let new_dir = Path::new(args[0]);
            let target_dir = if new_dir.is_absolute() {
                new_dir.to_path_buf()
            } else {
                self.current_dir.join(new_dir)
            };

            if target_dir.is_dir() {
                self.current_dir = target_dir;
            } else {
                return Err(format!("Directory not found: {}", args[0]).into());
            }
        }

        env::set_current_dir(&self.current_dir)?;
        Ok(())
    }

    /// List files in current or specified directory
    fn cmd_ls(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let pattern = if args.is_empty() { "*" } else { args[0] };
        let path_pattern = self.current_dir.join(pattern);
        let pattern_str = path_pattern.to_string_lossy();

        // Use glob pattern matching
        let mut entries = Vec::new();
        for entry in glob(&pattern_str)? {
            match entry {
                Ok(path) => {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "".to_string());

                    let formatted = if path.is_dir() {
                        name.blue().bold().to_string()
                    } else {
                        name
                    };

                    entries.push(formatted);
                }
                Err(e) => eprintln!("{}: {}", "Error".red(), e),
            }
        }

        // Sort and print entries
        entries.sort();
        for chunk in entries.chunks(5) {
            for entry in chunk {
                print!("{entry:<20}");
            }
            println!();
        }

        if entries.is_empty() {
            println!("No files found matching pattern: {pattern}");
        }

        Ok(())
    }

    /// Preview transformation without applying
    fn cmd_preview(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if args.len() < 2 {
            return Err("Usage: preview <transform> <file_pattern>".into());
        }

        let transform_type = TransformType::from_str(args[0])
            .ok_or_else(|| format!("Unknown transformation: {}", args[0]))?;

        self.preview_transform(transform_type, &args[1..])
    }

    /// Apply transformation to files
    fn cmd_apply(&mut self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if args.len() < 2 {
            return Err("Usage: apply <transform> <file_pattern>".into());
        }

        let transform_type = TransformType::from_str(args[0])
            .ok_or_else(|| format!("Unknown transformation: {}", args[0]))?;

        self.apply_transform(transform_type, &args[1..])
    }

    /// Undo the last operation
    fn cmd_undo(&mut self) -> Result<(), Box<dyn Error>> {
        match self.history_manager.undo() {
            Ok(_) => {
                println!("Operation undone successfully.");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Interactive rename wizard
    fn cmd_rename(&mut self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if args.is_empty() {
            return Err("Usage: rename <file_pattern> [options]".into());
        }

        // Collect files to rename
        let mut files = Vec::new();
        for pattern in args {
            let path_pattern = self.current_dir.join(pattern);
            let pattern_str = path_pattern.to_string_lossy();

            for entry in glob(&pattern_str)? {
                match entry {
                    Ok(path) => {
                        if path.is_file() {
                            files.push(path);
                        }
                    }
                    Err(e) => eprintln!("{}: {}", "Error".red(), e),
                }
            }
        }

        if files.is_empty() {
            println!("No files found matching pattern.");
            return Ok(());
        }

        // List files to be processed
        println!("\n{} files found:", files.len());
        for (i, file) in files.iter().enumerate() {
            let name = file.file_name().unwrap_or_default().to_string_lossy();
            println!("  {}. {}", i + 1, name);
        }

        // Ask for transformation
        println!("\nSelect transformation:");
        println!("  1. Clean up spaces and special characters");
        println!("  2. Convert to snake_case");
        println!("  3. Convert to kebab-case");
        println!("  4. Convert to Title Case");
        println!("  5. Convert to camelCase");
        println!("  6. Convert to PascalCase");
        println!("  7. Convert to lowercase");
        println!("  8. Convert to UPPERCASE");

        print!("Enter selection [1-8]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let transform_type = match input.trim().parse::<usize>() {
            Ok(1) => TransformType::Clean,
            Ok(2) => TransformType::Snake,
            Ok(3) => TransformType::Kebab,
            Ok(4) => TransformType::Title,
            Ok(5) => TransformType::Camel,
            Ok(6) => TransformType::Pascal,
            Ok(7) => TransformType::Lower,
            Ok(8) => TransformType::Upper,
            _ => {
                println!("Invalid selection. Using Clean transformation.");
                TransformType::Clean
            }
        };

        // Preview transformations
        let mut changes = Vec::new();
        println!("\nPreview of changes:");

        for path in &files {
            let filename = path
                .file_name()
                .ok_or("Invalid file name")?
                .to_string_lossy();

            // Apply the transformation
            let new_name = transform(&filename, transform_type);

            // Skip if no change
            if filename == new_name {
                println!("  \"{filename}\" → (no change needed)");
                continue;
            }

            // Create the new path
            let parent = path.parent().unwrap_or(Path::new(""));
            let new_path = parent.join(&new_name);

            // Check for conflicts
            if new_path.exists() && path != &new_path {
                println!(
                    "  \"{}\" → \"{}\" {}",
                    filename,
                    new_name,
                    "CONFLICT - file exists".red()
                );
                continue;
            }

            changes.push((path.clone(), new_path.clone()));
            println!("  \"{filename}\" → \"{new_name}\"");
        }

        if changes.is_empty() {
            println!("\nNo changes needed or all changes would create conflicts.");
            return Ok(());
        }

        // Confirm with user
        print!("\nApply these changes? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }

        // Apply changes
        for (src, dst) in changes {
            // Record the operation for undo
            self.history_manager.record(src.clone(), dst.clone())?;

            // Perform the rename
            match std::fs::rename(&src, &dst) {
                Ok(_) => {
                    let src_name = src.file_name().unwrap_or_default().to_string_lossy();
                    let dst_name = dst.file_name().unwrap_or_default().to_string_lossy();
                    println!("Renamed: \"{src_name}\" → \"{dst_name}\"");
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to rename \"{}\" - {}",
                        "Error".red(),
                        src.display(),
                        e
                    );
                }
            }
        }

        println!("\nRenaming complete!");
        Ok(())
    }

    /// Preview transformation on files
    fn preview_transform(
        &self,
        transform_type: TransformType,
        patterns: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        let mut changes = Vec::new();

        // Process each file pattern
        for pattern in patterns {
            let path_pattern = self.current_dir.join(pattern);
            let pattern_str = path_pattern.to_string_lossy();

            for entry in glob(&pattern_str)? {
                match entry {
                    Ok(path) => {
                        // Skip directories
                        if path.is_dir() {
                            continue;
                        }

                        // Get the file name
                        let filename = path
                            .file_name()
                            .ok_or("Invalid file name")?
                            .to_string_lossy();

                        // Apply the transformation
                        let new_name = transform(&filename, transform_type);

                        // If the name hasn't changed, skip
                        if filename == new_name {
                            println!("{filename} → {new_name} (no change needed)");
                            continue;
                        }

                        // Create the new path
                        let parent = path.parent().unwrap_or(Path::new(""));
                        let new_path = parent.join(&new_name);

                        // Check for conflicts
                        if new_path.exists() && path != new_path {
                            println!(
                                "{}: Cannot rename \"{}\" to \"{}\" - file already exists",
                                "Conflict".red(),
                                filename,
                                new_name
                            );
                            continue;
                        }

                        changes.push((path.clone(), new_path.clone()));
                        println!("{} \"{}\" → \"{}\"", "Preview:".blue(), filename, new_name);
                    }
                    Err(e) => eprintln!("{}: {}", "Error".red(), e),
                }
            }
        }

        if changes.is_empty() {
            println!("No files found or no changes needed.");
        } else {
            println!("\nFound {} file(s) to rename.", changes.len());
        }

        Ok(())
    }

    /// Apply transformation to files
    fn apply_transform(
        &mut self,
        transform_type: TransformType,
        patterns: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        let mut changes = Vec::new();

        // Process each file pattern
        for pattern in patterns {
            let path_pattern = self.current_dir.join(pattern);
            let pattern_str = path_pattern.to_string_lossy();

            for entry in glob(&pattern_str)? {
                match entry {
                    Ok(path) => {
                        // Skip directories
                        if path.is_dir() {
                            continue;
                        }

                        // Get the file name
                        let filename = path
                            .file_name()
                            .ok_or("Invalid file name")?
                            .to_string_lossy();

                        // Apply the transformation
                        let new_name = transform(&filename, transform_type);

                        // If the name hasn't changed, skip
                        if filename == new_name {
                            continue;
                        }

                        // Create the new path
                        let parent = path.parent().unwrap_or(Path::new(""));
                        let new_path = parent.join(&new_name);

                        // Check for conflicts
                        if new_path.exists() && path != new_path {
                            println!(
                                "{}: Cannot rename \"{}\" to \"{}\" - file already exists",
                                "Conflict".red(),
                                filename,
                                new_name
                            );
                            continue;
                        }

                        changes.push((path.clone(), new_path.clone()));
                    }
                    Err(e) => eprintln!("{}: {}", "Error".red(), e),
                }
            }
        }

        if changes.is_empty() {
            println!("No files found or no changes needed.");
            return Ok(());
        }

        // Display changes
        println!("\nThe following {} file(s) will be renamed:", changes.len());
        for (src, dst) in &changes {
            let src_name = src.file_name().unwrap_or_default().to_string_lossy();
            let dst_name = dst.file_name().unwrap_or_default().to_string_lossy();
            println!("  \"{src_name}\" → \"{dst_name}\"");
        }

        // Confirm with user
        print!("\nApply these changes? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }

        // Apply changes
        for (src, dst) in changes {
            // Record the operation for undo
            self.history_manager.record(src.clone(), dst.clone())?;

            // Perform the rename
            match std::fs::rename(&src, &dst) {
                Ok(_) => {
                    let src_name = src.file_name().unwrap_or_default().to_string_lossy();
                    let dst_name = dst.file_name().unwrap_or_default().to_string_lossy();
                    println!("Renamed: \"{src_name}\" → \"{dst_name}\"");
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to rename \"{}\" - {}",
                        "Error".red(),
                        src.display(),
                        e
                    );
                }
            }
        }

        println!("\nRenaming complete!");
        Ok(())
    }
}
