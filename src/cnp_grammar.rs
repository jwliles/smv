use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// CNP Grammar Parser for SMV
/// Implements the full CNP grammar specification with filters, routes, and semantic groups

#[derive(Debug, Clone)]
pub struct CnpCommand {
    pub path: String,
    pub filters: Vec<Filter>,
    pub routes: Vec<Route>,
    pub flags: String,
    pub transform_command: Option<TransformCommand>,
}

#[derive(Debug, Clone)]
pub struct TransformCommand {
    pub command_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Filter {
    Name(String),
    Type(FileType),
    Extension(String),
    SizeGreater(String),
    SizeLess(String),
    DepthGreater(usize),
    DepthLess(usize),
    ModifiedAfter(String),
    ModifiedBefore(String),
    AccessedAfter(String),
    AccessedBefore(String),
    Tag(String),
    Hash(String),
    Where(Vec<Filter>),
    For(SemanticGroup),
}

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Folder,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
pub enum SemanticGroup {
    Notes,    // EXT:md + TYPE:file + common note paths
    Media,    // EXT:jpg/png/gif/webm/mp4 + TYPE:file
    Scripts,  // EXT:sh/py/rb/pl/rs + TYPE:file
    Projects, // TYPE:folder + NAME:src/build/docs
    Configs,  // EXT:conf/ini/yaml/toml/json + TYPE:file
}

#[derive(Debug, Clone)]
pub enum Route {
    To { tool: String, args: Vec<String> }, // TO:tool[:arg1,arg2] - delegate to another CNP tool with optional args
    Into(String),                           // INTO:file - write output to file
    Format(OutputFormat),                   // FORMAT:type - change output format
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    Csv,
    Text,
    Yaml,
}

#[derive(Debug)]
pub struct GrammarParseError {
    pub message: String,
}

impl fmt::Display for GrammarParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CNP Grammar Parse Error: {}", self.message)
    }
}

impl Error for GrammarParseError {}

pub struct CnpGrammarParser;

impl CnpGrammarParser {
    pub fn parse(args: &[String]) -> Result<CnpCommand, Box<dyn Error>> {
        let mut command = CnpCommand {
            path: ".".to_string(),
            filters: Vec::new(),
            routes: Vec::new(),
            flags: String::new(),
            transform_command: None,
        };

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            // Parse CNP filters (UPPERCASE keywords)
            if let Some(filter) = Self::parse_filter(arg)? {
                command.filters.push(filter);
                i += 1;
                continue;
            }

            // Parse CNP routes
            if let Some(route) = Self::parse_route(arg)? {
                command.routes.push(route);
                i += 1;
                continue;
            }

            // Parse SMV transform commands
            if let Some(transform) = Self::parse_transform_command(args, &mut i)? {
                command.transform_command = Some(transform);
                continue;
            }

            // Parse flags (starting with -)
            if arg.starts_with('-') {
                command.flags.push_str(&arg[1..]);
                i += 1;
                continue;
            }

            // Parse path (first non-keyword argument)
            if command.path == "." && !arg.contains(':') && !arg.starts_with('-') {
                command.path = arg.clone();
                i += 1;
                continue;
            }

            i += 1;
        }

        Ok(command)
    }

    fn parse_filter(arg: &str) -> Result<Option<Filter>, Box<dyn Error>> {
        if !arg.contains(':')
            && !arg.starts_with("SIZE")
            && !arg.starts_with("DEPTH")
            && !arg.starts_with("MODIFIED")
            && !arg.starts_with("ACCESSED")
        {
            return Ok(None);
        }

        // Handle SIZE comparisons
        if arg.starts_with("SIZE>") {
            return Ok(Some(Filter::SizeGreater(arg[5..].to_string())));
        }
        if arg.starts_with("SIZE<") {
            return Ok(Some(Filter::SizeLess(arg[5..].to_string())));
        }

        // Handle DEPTH comparisons
        if arg.starts_with("DEPTH>") {
            let value = arg[6..].parse::<usize>().map_err(|_| GrammarParseError {
                message: format!("Invalid depth value: {}", &arg[6..]),
            })?;
            return Ok(Some(Filter::DepthGreater(value)));
        }
        if arg.starts_with("DEPTH<") {
            let value = arg[6..].parse::<usize>().map_err(|_| GrammarParseError {
                message: format!("Invalid depth value: {}", &arg[6..]),
            })?;
            return Ok(Some(Filter::DepthLess(value)));
        }

        // Handle timestamp comparisons
        if arg.starts_with("MODIFIED>") {
            return Ok(Some(Filter::ModifiedAfter(arg[9..].to_string())));
        }
        if arg.starts_with("MODIFIED<") {
            return Ok(Some(Filter::ModifiedBefore(arg[9..].to_string())));
        }
        if arg.starts_with("ACCESSED>") {
            return Ok(Some(Filter::AccessedAfter(arg[9..].to_string())));
        }
        if arg.starts_with("ACCESSED<") {
            return Ok(Some(Filter::AccessedBefore(arg[9..].to_string())));
        }

        // Handle colon-separated filters
        if let Some(colon_pos) = arg.find(':') {
            let key = &arg[..colon_pos];
            let value = &arg[colon_pos + 1..];

            match key {
                "NAME" => Ok(Some(Filter::Name(value.to_string()))),
                "TYPE" => {
                    let file_type = match value.to_lowercase().as_str() {
                        "file" => FileType::File,
                        "folder" | "dir" | "directory" => FileType::Folder,
                        "symlink" | "link" => FileType::Symlink,
                        "other" => FileType::Other,
                        _ => {
                            return Err(Box::new(GrammarParseError {
                                message: format!("Invalid file type: {}", value),
                            }))
                        }
                    };
                    Ok(Some(Filter::Type(file_type)))
                }
                "EXT" => Ok(Some(Filter::Extension(value.to_string()))),
                "TAG" => Ok(Some(Filter::Tag(value.to_string()))),
                "HASH" => Ok(Some(Filter::Hash(value.to_string()))),
                "FOR" => {
                    let semantic_group = match value.to_lowercase().as_str() {
                        "notes" => SemanticGroup::Notes,
                        "media" => SemanticGroup::Media,
                        "scripts" => SemanticGroup::Scripts,
                        "projects" => SemanticGroup::Projects,
                        "configs" => SemanticGroup::Configs,
                        _ => {
                            return Err(Box::new(GrammarParseError {
                                message: format!("Invalid semantic group: {}", value),
                            }))
                        }
                    };
                    Ok(Some(Filter::For(semantic_group)))
                }
                _ => Ok(None), // Unknown filter, ignore
            }
        } else {
            Ok(None)
        }
    }

    fn parse_route(arg: &str) -> Result<Option<Route>, Box<dyn Error>> {
        if !arg.contains(':') {
            return Ok(None);
        }

        if let Some(colon_pos) = arg.find(':') {
            let key = &arg[..colon_pos];
            let value = &arg[colon_pos + 1..];

            match key {
                "TO" => {
                    // Parse TO:tool or TO:tool:arg1,arg2 syntax
                    if let Some(tool_args_pos) = value.find(':') {
                        // Extended syntax: TO:tool:arg1,arg2
                        let tool = value[..tool_args_pos].to_string();
                        let args_str = &value[tool_args_pos + 1..];
                        let args: Vec<String> = args_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        Ok(Some(Route::To { tool, args }))
                    } else {
                        // Basic syntax: TO:tool
                        Ok(Some(Route::To {
                            tool: value.to_string(),
                            args: Vec::new(),
                        }))
                    }
                }
                "INTO" => Ok(Some(Route::Into(value.to_string()))),
                "FORMAT" => {
                    let format = match value.to_lowercase().as_str() {
                        "json" => OutputFormat::Json,
                        "csv" => OutputFormat::Csv,
                        "text" | "txt" => OutputFormat::Text,
                        "yaml" | "yml" => OutputFormat::Yaml,
                        _ => {
                            return Err(Box::new(GrammarParseError {
                                message: format!("Invalid output format: {}", value),
                            }))
                        }
                    };
                    Ok(Some(Route::Format(format)))
                }
                _ => Ok(None), // Unknown route, ignore
            }
        } else {
            Ok(None)
        }
    }

    fn parse_transform_command(
        args: &[String],
        i: &mut usize,
    ) -> Result<Option<TransformCommand>, Box<dyn Error>> {
        if *i >= args.len() {
            return Ok(None);
        }

        let arg = &args[*i];

        // Check for SMV transform commands
        match arg.to_lowercase().as_str() {
            "change" => {
                if *i + 3 < args.len() && args[*i + 2] == "INTO" {
                    let old_value = args[*i + 1].clone();
                    let new_value = args[*i + 3].clone();
                    *i += 4;
                    return Ok(Some(TransformCommand {
                        command_type: "change".to_string(),
                        old_value: Some(old_value),
                        new_value: Some(new_value),
                    }));
                }
            }
            "regex" => {
                if *i + 3 < args.len() && args[*i + 2] == "INTO" {
                    let pattern = args[*i + 1].clone();
                    let replacement = args[*i + 3].clone();
                    *i += 4;
                    return Ok(Some(TransformCommand {
                        command_type: "regex".to_string(),
                        old_value: Some(pattern),
                        new_value: Some(replacement),
                    }));
                }
            }
            "snake" | "kebab" | "pascal" | "camel" | "title" | "lower" | "upper" | "clean" => {
                *i += 1;
                return Ok(Some(TransformCommand {
                    command_type: arg.clone(),
                    old_value: None,
                    new_value: None,
                }));
            }
            _ => {}
        }

        *i += 1;
        Ok(None)
    }

    /// Expand semantic groups into concrete filters
    pub fn expand_semantic_groups(filters: &[Filter]) -> Vec<Filter> {
        let mut expanded = Vec::new();

        for filter in filters {
            match filter {
                Filter::For(group) => match group {
                    SemanticGroup::Notes => {
                        expanded.push(Filter::Extension("md".to_string()));
                        expanded.push(Filter::Type(FileType::File));
                    }
                    SemanticGroup::Media => {
                        for ext in ["jpg", "png", "gif", "webm", "mp4", "jpeg", "webp", "svg"] {
                            expanded.push(Filter::Extension(ext.to_string()));
                        }
                        expanded.push(Filter::Type(FileType::File));
                    }
                    SemanticGroup::Scripts => {
                        for ext in ["sh", "py", "rb", "pl", "rs", "js", "ts", "bash", "zsh"] {
                            expanded.push(Filter::Extension(ext.to_string()));
                        }
                        expanded.push(Filter::Type(FileType::File));
                    }
                    SemanticGroup::Projects => {
                        expanded.push(Filter::Type(FileType::Folder));
                        for name in ["src", "build", "docs", "target", "dist", "bin"] {
                            expanded.push(Filter::Name(name.to_string()));
                        }
                    }
                    SemanticGroup::Configs => {
                        for ext in [
                            "conf", "ini", "yaml", "yml", "toml", "json", "config", "cfg",
                        ] {
                            expanded.push(Filter::Extension(ext.to_string()));
                        }
                        expanded.push(Filter::Type(FileType::File));
                    }
                },
                _ => expanded.push(filter.clone()),
            }
        }

        expanded
    }
}
