use std::path::PathBuf;

/// Command types that can be executed in the application
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// Types of transformations available via command mode
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TransformType {
    Snake,
    Kebab,
    Title,
    Camel,
    Pascal,
    Lower,
    Upper,
    Clean,
    Sentence,
    Start,
    Studly,
    SplitSnake,
    SplitKebab,
    SplitTitle,
    SplitCamel,
    SplitPascal,
    SplitLower,
    SplitUpper,
    SplitSentence,
    SplitStart,
    SplitStudly,
    Replace(String, String),
    ReplaceRegex(String, String),
}

/// Parse a command string into a Command
#[allow(dead_code)]
pub fn parse_command(command: &str) -> Option<Command> {
    let trimmed = command.trim();
    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).unwrap_or(&"").trim();

    match cmd {
        "cd" | "chdir" => {
            if args.is_empty() {
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
        "sentence" => Some(Command::Transform(TransformType::Sentence)),
        "start" => Some(Command::Transform(TransformType::Start)),
        "studly" => Some(Command::Transform(TransformType::Studly)),
        "split" => match args {
            "snake" => Some(Command::Transform(TransformType::SplitSnake)),
            "kebab" => Some(Command::Transform(TransformType::SplitKebab)),
            "title" => Some(Command::Transform(TransformType::SplitTitle)),
            "camel" => Some(Command::Transform(TransformType::SplitCamel)),
            "pascal" => Some(Command::Transform(TransformType::SplitPascal)),
            "lower" => Some(Command::Transform(TransformType::SplitLower)),
            "upper" => Some(Command::Transform(TransformType::SplitUpper)),
            "sentence" => Some(Command::Transform(TransformType::SplitSentence)),
            "start" => Some(Command::Transform(TransformType::SplitStart)),
            "studly" => Some(Command::Transform(TransformType::SplitStudly)),
            _ => None,
        },
        "replace" => parse_replace_args(args).map(|(f, r)| {
            Command::Transform(TransformType::Replace(f, r))
        }),
        "regex" => parse_replace_args(args).map(|(p, r)| {
            Command::Transform(TransformType::ReplaceRegex(p, r))
        }),
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

/// Parse `"find" "replace"` or `find replace` arguments
fn parse_replace_args(args: &str) -> Option<(String, String)> {
    // Try quoted form: "find" "replace"
    if args.starts_with('"') {
        let args = &args[1..];
        if let Some(end) = args.find('"') {
            let find = args[..end].to_string();
            let rest = args[end + 1..].trim();
            let replacement = if rest.starts_with('"') {
                let rest = &rest[1..];
                rest.find('"').map(|e| rest[..e].to_string())
            } else {
                Some(rest.to_string())
            };
            return replacement.map(|r| (find, r));
        }
    }
    // Plain form: find replace
    let parts: Vec<&str> = args.splitn(2, ' ').collect();
    if parts.len() == 2 && !parts[0].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}
