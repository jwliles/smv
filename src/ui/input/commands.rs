use std::path::PathBuf;

/// Command types that can be executed in the application
#[derive(Debug, Clone)]
pub enum Command {
    /// Change directory
    ChangeDirectory(PathBuf),
    /// Find files matching pattern
    Find(String),
    /// Apply transformation to selected files
    Transform(TransformType),
    /// Execute queued operations
    Execute,
    /// Clear the queue
    ClearQueue,
    /// Quit the application
    Quit,
    /// Copy selected files to clipboard
    Copy,
    /// Save the current queue to a YAML file
    SaveQueue(PathBuf),
    /// Load a queue from a YAML file
    LoadQueue(PathBuf),
}

/// Types of transformations
#[derive(Debug, Clone, Copy)]
pub enum TransformType {
    /// Convert to snake_case
    Snake,
    /// Convert to kebab-case
    Kebab,
    /// Convert to Title Case
    Title,
    /// Convert to camelCase
    Camel,
    /// Convert to PascalCase
    Pascal,
    /// Convert to lowercase
    Lower,
    /// Convert to UPPERCASE
    Upper,
    /// Clean spaces and special characters
    Clean,
}

/// Parse a command string
pub fn parse_command(command: &str) -> Option<Command> {
    let trimmed = command.trim();
    
    // Split into command and arguments
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).unwrap_or(&"").trim();
    
    match cmd {
        "cd" | "chdir" => {
            if args.is_empty() {
                // Default to home directory
                dirs::home_dir().map(Command::ChangeDirectory)
            } else {
                Some(Command::ChangeDirectory(PathBuf::from(args)))
            }
        }
        "find" => {
            if !args.is_empty() {
                Some(Command::Find(args.to_string()))
            } else {
                None
            }
        }
        "snake" => Some(Command::Transform(TransformType::Snake)),
        "kebab" => Some(Command::Transform(TransformType::Kebab)),
        "title" => Some(Command::Transform(TransformType::Title)),
        "camel" => Some(Command::Transform(TransformType::Camel)),
        "pascal" => Some(Command::Transform(TransformType::Pascal)),
        "lower" => Some(Command::Transform(TransformType::Lower)),
        "upper" => Some(Command::Transform(TransformType::Upper)),
        "clean" => Some(Command::Transform(TransformType::Clean)),
        "execute" | "exec" => Some(Command::Execute),
        "clear" => Some(Command::ClearQueue),
        "quit" | "q" => Some(Command::Quit),
        "copy" | "cp" => Some(Command::Copy),
        "save" => {
            if args.is_empty() {
                None
            } else {
                Some(Command::SaveQueue(PathBuf::from(args)))
            }
        }
        "load" => {
            if args.is_empty() {
                None
            } else {
                Some(Command::LoadQueue(PathBuf::from(args)))
            }
        }
        _ => None,
    }
}