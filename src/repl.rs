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
            let prompt = format!(
                "{}{}> ",
                "smv:".green().bold(),
                self.current_dir.display().to_string().cyan()
            );

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
        println!("{}", "━".repeat(60).dimmed());
        println!("{}", " SMV - Smart Move Utility ".bold().green().on_black());
        println!("{}", "━".repeat(60).dimmed());
        println!(
            "  {} Rename files easily using various transformations",
            "•".green()
        );
        println!(
            "  {} Supports multiple rename patterns and batch operations",
            "•".green()
        );
        println!(
            "  {} Type {} for available commands",
            "•".green(),
            "help".cyan().bold()
        );
        println!("{}", "━".repeat(60).dimmed());
    }

    /// Display help text
    fn cmd_help(&self) -> Result<(), Box<dyn Error>> {
        println!("\n{}", "━".repeat(60).dimmed());
        println!("{}", "SMV Help".green().bold());
        println!("{}", "━".repeat(60).dimmed());

        println!("\n{}", "Commands:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());

        let cmd_width = 12;
        let desc_width = 48;

        // File operations
        println!("  {}", "File Operations:".yellow());
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "ls".cyan(),
            "List files and directories in current location"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "ls <pattern>".cyan(),
            "List files matching a pattern (e.g., ls *.txt)"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "cd <dir>".cyan(),
            "Change to specified directory"
        );

        // Transformation commands
        println!("\n  {}", "Transformation Commands:".yellow());
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "preview".cyan(),
            "Show preview of transformation without applying"
        );
        println!(
            "    {:<cmd_width$} {:<desc_width$}",
            "preview <transform> <files>".white().dimmed(),
            "Example: preview snake *.txt"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "apply".cyan(),
            "Apply transformation to files"
        );
        println!(
            "    {:<cmd_width$} {:<desc_width$}",
            "apply <transform> <files>".white().dimmed(),
            "Example: apply snake *.txt"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "<transform>".cyan(),
            "Shorthand for preview (e.g., snake *.txt)"
        );

        // Other commands
        println!("\n  {}", "Other Commands:".yellow());
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "rename".cyan(),
            "Interactive rename wizard"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "undo".cyan(),
            "Revert the last operation"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "help".cyan(),
            "Display this help information"
        );
        println!(
            "  {:<cmd_width$} {:<desc_width$}",
            "quit/exit".cyan(),
            "Exit the program"
        );

        // Transformations
        println!("\n{}", "Transformations:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());

        // Display transformations in a table format
        let transforms = [
            (
                "clean".yellow().to_string(),
                "Remove special chars & normalize spaces",
            ),
            (
                "snake".yellow().to_string(),
                "Convert to snake_case (underscores)",
            ),
            (
                "kebab".yellow().to_string(),
                "Convert to kebab-case (hyphens)",
            ),
            ("title".yellow().to_string(), "Convert to Title Case"),
            ("camel".yellow().to_string(), "Convert to camelCase"),
            ("pascal".yellow().to_string(), "Convert to PascalCase"),
            ("lower".yellow().to_string(), "Convert to lowercase"),
            ("upper".yellow().to_string(), "Convert to UPPERCASE"),
        ];

        for (name, desc) in &transforms {
            println!("  {:<20} {}", format!("{}...", name), desc);
        }

        // Examples section
        println!("\n{}", "Examples:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        println!("  {:<40}", "Preview snake_case transformation:".yellow());
        println!("  {}", "preview snake *.txt".white());

        println!("\n  {:<40}", "Apply kebab-case to all MP3 files:".yellow());
        println!("  {}", "apply kebab *.mp3".white());

        println!("\n  {:<40}", "Quick preview of title case:".yellow());
        println!("  {}", "title my-files-*.jpg".white());

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

        // Display header
        println!("\n{}", "━".repeat(60).dimmed());
        println!(
            "{} {}",
            "Directory:".blue().bold(),
            self.current_dir.display().to_string().white()
        );
        if pattern != "*" {
            println!("{} {}", "Pattern:".blue().bold(), pattern.yellow());
        }
        println!("{}", "━".repeat(60).dimmed());

        // Use glob pattern matching
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        let mut total_size: u64 = 0;

        for entry in glob(&pattern_str)? {
            match entry {
                Ok(path) => {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "".to_string());

                    if path.is_dir() {
                        dirs.push(name);
                    } else {
                        // Get file size if possible
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            total_size += metadata.len();
                        }
                        files.push(name);
                    }
                }
                Err(e) => eprintln!("  {} {}", "Error:".red().bold(), e),
            }
        }

        // Sort entries
        dirs.sort();
        files.sort();

        // Format and display directories
        if !dirs.is_empty() {
            println!("\n{}", "Directories:".cyan().bold());

            let mut output = String::new();
            for (i, dir) in dirs.iter().enumerate() {
                let formatted = format!("  {}/", dir).blue().bold().to_string();
                output.push_str(&formatted);

                // Add padding and handle line breaks
                if (i + 1) % 3 == 0 || i == dirs.len() - 1 {
                    output.push('\n');
                } else {
                    output.push_str("  ");
                }
            }
            print!("{}", output);
        }

        // Format and display files
        if !files.is_empty() {
            println!("\n{}", "Files:".green().bold());

            let mut output = String::new();
            for (i, file) in files.iter().enumerate() {
                let formatted = format!("  {}", file).white().to_string();
                output.push_str(&formatted);

                // Add padding and handle line breaks
                if (i + 1) % 3 == 0 || i == files.len() - 1 {
                    output.push('\n');
                } else {
                    output.push_str("  ");
                }
            }
            print!("{}", output);
        }

        // Display summary
        println!("\n{}", "Summary:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        println!("  {} directories", dirs.len().to_string().blue().bold());
        println!("  {} files", files.len().to_string().green().bold());

        // Format size in a human-readable way
        if total_size > 0 {
            let size_str = if total_size < 1024 {
                format!("{} B", total_size)
            } else if total_size < 1024 * 1024 {
                format!("{:.1} KB", total_size as f64 / 1024.0)
            } else if total_size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", total_size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0))
            };
            println!("  {} total", size_str.white().bold());
        }

        if files.is_empty() && dirs.is_empty() {
            println!(
                "\n{}",
                "No files or directories found matching pattern.".yellow()
            );
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
            let new_name = transform(&filename, &transform_type);

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
        let mut no_changes = Vec::new();
        let mut conflicts = Vec::new();

        // Display header
        println!("\n{}", "━".repeat(60).dimmed());
        println!(
            "{} {} {}",
            "Preview:".blue().bold(),
            transform_type.as_str().yellow().bold(),
            format!("({})", patterns.join(", ")).dimmed()
        );
        println!("{}", "━".repeat(60).dimmed());

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
                        let new_name = transform(&filename, &transform_type);

                        // Create the new path
                        let parent = path.parent().unwrap_or(Path::new(""));
                        let new_path = parent.join(&new_name);

                        // If the name hasn't changed, track but don't show
                        if filename == new_name {
                            no_changes.push(filename.to_string());
                            continue;
                        }

                        // Check for conflicts
                        if new_path.exists() && path != new_path {
                            conflicts.push((filename.to_string(), new_name.to_string()));
                            continue;
                        }

                        changes.push((
                            path.clone(),
                            new_path.clone(),
                            filename.to_string(),
                            new_name.to_string(),
                        ));
                    }
                    Err(e) => eprintln!("  {} {}", "Error:".red().bold(), e),
                }
            }
        }

        // Display the results in a structured way
        if !changes.is_empty() {
            println!("\n{}", "Files to rename:".green().bold());
            println!("{}", "┈".repeat(60).dimmed());
            for (_, _, src_name, dst_name) in &changes {
                println!("  \"{}\" {}", src_name.white(), "→".dimmed());
                println!("     \"{}\"", dst_name.green());
            }
        }

        if !conflicts.is_empty() {
            println!("\n{}", "Conflicts detected:".red().bold());
            println!("{}", "┈".repeat(60).dimmed());
            for (src_name, dst_name) in &conflicts {
                println!("  \"{}\" {}", src_name, "→".dimmed());
                println!(
                    "     \"{}\" {}",
                    dst_name.dimmed(),
                    "File already exists".red()
                );
            }
        }

        // Summary
        println!("\n{}", "Summary:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        println!(
            "  {} files matched pattern",
            (changes.len() + conflicts.len() + no_changes.len())
                .to_string()
                .white()
                .bold()
        );
        println!(
            "  {} files ready to rename",
            changes.len().to_string().green().bold()
        );
        println!(
            "  {} files with conflicts",
            conflicts.len().to_string().red().bold()
        );
        println!(
            "  {} files with no changes needed",
            no_changes.len().to_string().yellow()
        );

        // Instructions
        if !changes.is_empty() {
            println!("\n{}", "To apply these changes:".cyan());
            println!(
                "  {}",
                format!("apply {} {}", transform_type.as_str(), patterns.join(" ")).white()
            );
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
        let mut no_changes = Vec::new();
        let mut conflicts = Vec::new();

        // Display header
        println!("\n{}", "━".repeat(60).dimmed());
        println!(
            "{} {} {}",
            "Apply:".blue().bold(),
            transform_type.as_str().yellow().bold(),
            format!("({})", patterns.join(", ")).dimmed()
        );
        println!("{}", "━".repeat(60).dimmed());

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
                        let new_name = transform(&filename, &transform_type);

                        // If the name hasn't changed, track but don't show
                        if filename == new_name {
                            no_changes.push(filename.to_string());
                            continue;
                        }

                        // Create the new path
                        let parent = path.parent().unwrap_or(Path::new(""));
                        let new_path = parent.join(&new_name);

                        // Check for conflicts
                        if new_path.exists() && path != new_path {
                            conflicts.push((filename.to_string(), new_name.to_string()));
                            continue;
                        }

                        changes.push((
                            path.clone(),
                            new_path.clone(),
                            filename.to_string(),
                            new_name.to_string(),
                        ));
                    }
                    Err(e) => eprintln!("  {} {}", "Error:".red().bold(), e),
                }
            }
        }

        if changes.is_empty() && conflicts.is_empty() {
            println!("\n{}", "No files found or no changes needed.".yellow());
            return Ok(());
        }

        // Display the results in a structured way
        if !changes.is_empty() {
            println!("\n{}", "Files to rename:".green().bold());
            println!("{}", "┈".repeat(60).dimmed());
            for (_src, _dst, src_name, dst_name) in &changes {
                println!("  \"{}\" {}", src_name.white(), "→".dimmed());
                println!("     \"{}\"", dst_name.green());
            }
        }

        if !conflicts.is_empty() {
            println!("\n{}", "Conflicts detected:".red().bold());
            println!("{}", "┈".repeat(60).dimmed());
            for (src_name, dst_name) in &conflicts {
                println!("  \"{}\" {}", src_name, "→".dimmed());
                println!(
                    "     \"{}\" {}",
                    dst_name.dimmed(),
                    "File already exists".red()
                );
            }
        }

        // Summary
        println!("\n{}", "Summary:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        println!(
            "  {} files matched pattern",
            (changes.len() + conflicts.len() + no_changes.len())
                .to_string()
                .white()
                .bold()
        );
        println!(
            "  {} files ready to rename",
            changes.len().to_string().green().bold()
        );
        println!(
            "  {} files with conflicts",
            conflicts.len().to_string().red().bold()
        );
        println!(
            "  {} files with no changes needed",
            no_changes.len().to_string().yellow()
        );

        if changes.is_empty() {
            println!("\n{}", "No changes to apply.".yellow());
            return Ok(());
        }

        // Confirm with user
        println!("\n{}", "Confirmation:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        print!("Apply these changes? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }

        // Apply changes section
        println!("\n{}", "Applying changes:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());

        let mut success_count = 0;
        let mut error_count = 0;

        for (src, dst, _, _) in changes {
            // Record the operation for undo
            self.history_manager.record(src.clone(), dst.clone())?;

            // Perform the rename
            match std::fs::rename(&src, &dst) {
                Ok(_) => {
                    let src_name = src.file_name().unwrap_or_default().to_string_lossy();
                    let dst_name = dst.file_name().unwrap_or_default().to_string_lossy();
                    println!(
                        "  {} \"{}\" {} \"{}\"",
                        "✓".green(),
                        src_name.white(),
                        "→".dimmed(),
                        dst_name.green()
                    );
                    success_count += 1;
                }
                Err(e) => {
                    let src_name = src.file_name().unwrap_or_default().to_string_lossy();
                    eprintln!(
                        "  {} {} \"{}\" - {}",
                        "✗".red(),
                        "Failed to rename".red(),
                        src_name,
                        e
                    );
                    error_count += 1;
                }
            }
        }

        // Result summary
        println!("\n{}", "Results:".cyan().bold());
        println!("{}", "┈".repeat(60).dimmed());
        println!(
            "  {} successfully renamed",
            success_count.to_string().green().bold()
        );

        if error_count > 0 {
            println!(
                "  {} failed to rename",
                error_count.to_string().red().bold()
            );
        }

        if success_count > 0 {
            println!(
                "\n{}",
                "Use 'undo' command to revert these changes if needed.".cyan()
            );
        }

        Ok(())
    }
}
