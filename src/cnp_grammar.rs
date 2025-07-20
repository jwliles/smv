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
    pub remove_command: Option<RemoveCommand>,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone)]
pub struct TransformCommand {
    pub command_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoveCommand {
    pub command_type: String, // "rm"
    pub preview: bool,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Folder,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
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
            remove_command: None,
            case_insensitive: false,
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

            // Parse SMV remove commands FIRST (before transform commands)
            if let Some(remove) = Self::parse_remove_command(args, &mut i)? {
                command.remove_command = Some(remove);
                continue;
            }

            // Parse SMV transform commands
            if let Some(transform) = Self::parse_transform_command(args, &mut i)? {
                command.transform_command = Some(transform);
                continue;
            }

            // Parse flags (starting with -)
            if let Some(flags) = arg.strip_prefix('-') {
                command.flags.push_str(flags);
                // Check for case-insensitive flag (both CNP standard 'ic' and SMV-specific 'i')
                if flags.contains("ic") || flags.contains('i') {
                    command.case_insensitive = true;
                }
                i += 1;
                continue;
            }

            // Check for glob patterns and convert them to appropriate filters
            if Self::is_glob_pattern(arg) {
                if let Some(filter) = Self::convert_glob_to_filter(arg)? {
                    command.filters.push(filter);
                }
                i += 1;
                continue;
            }

            // Parse path (first non-keyword, non-command argument)
            if command.path == "." && !arg.contains(':') && !arg.starts_with('-') && arg != "rm" {
                command.path = arg.clone();
                i += 1;
                continue;
            }

            i += 1;
        }

        Ok(command)
    }

    /// Check if an argument is a glob pattern
    fn is_glob_pattern(arg: &str) -> bool {
        arg.contains('*') || arg.contains('?') || arg.contains('[') || arg.contains('{')
    }

    /// Convert a glob pattern to an appropriate CNP filter
    fn convert_glob_to_filter(pattern: &str) -> Result<Option<Filter>, Box<dyn Error>> {
        // Handle common cases like *.ext -> EXT:ext
        if pattern.starts_with("*.") && !pattern[2..].contains('.') && !pattern[2..].contains('*') {
            let extension = &pattern[2..];
            return Ok(Some(Filter::Extension(extension.to_string())));
        }

        // Handle patterns like *filename* -> NAME:filename
        if pattern.starts_with('*') && pattern.ends_with('*') && pattern.len() > 2 {
            let name_part = &pattern[1..pattern.len() - 1];
            if !name_part.contains('*') && !name_part.contains('?') {
                return Ok(Some(Filter::Name(name_part.to_string())));
            }
        }

        // Handle patterns like filename* -> NAME:filename (startswith)
        if pattern.ends_with('*') && !pattern[..pattern.len() - 1].contains('*') {
            let name_part = &pattern[..pattern.len() - 1];
            return Ok(Some(Filter::Name(format!("{}*", name_part))));
        }

        // Handle patterns like *filename -> NAME:filename (endswith)
        if pattern.starts_with('*') && !pattern[1..].contains('*') {
            let name_part = &pattern[1..];
            return Ok(Some(Filter::Name(format!("*{}", name_part))));
        }

        // For complex patterns, use a generic name filter
        Ok(Some(Filter::Name(pattern.to_string())))
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
        if let Some(stripped) = arg.strip_prefix("SIZE>") {
            return Ok(Some(Filter::SizeGreater(stripped.to_string())));
        }
        if let Some(stripped) = arg.strip_prefix("SIZE<") {
            return Ok(Some(Filter::SizeLess(stripped.to_string())));
        }

        // Handle DEPTH comparisons
        if let Some(stripped) = arg.strip_prefix("DEPTH>") {
            let value = stripped.parse::<usize>().map_err(|_| GrammarParseError {
                message: format!("Invalid depth value: {}", stripped),
            })?;
            return Ok(Some(Filter::DepthGreater(value)));
        }
        if let Some(stripped) = arg.strip_prefix("DEPTH<") {
            let value = stripped.parse::<usize>().map_err(|_| GrammarParseError {
                message: format!("Invalid depth value: {}", stripped),
            })?;
            return Ok(Some(Filter::DepthLess(value)));
        }

        // Handle timestamp comparisons
        if let Some(stripped) = arg.strip_prefix("MODIFIED>") {
            return Ok(Some(Filter::ModifiedAfter(stripped.to_string())));
        }
        if let Some(stripped) = arg.strip_prefix("MODIFIED<") {
            return Ok(Some(Filter::ModifiedBefore(stripped.to_string())));
        }
        if let Some(stripped) = arg.strip_prefix("ACCESSED>") {
            return Ok(Some(Filter::AccessedAfter(stripped.to_string())));
        }
        if let Some(stripped) = arg.strip_prefix("ACCESSED<") {
            return Ok(Some(Filter::AccessedBefore(stripped.to_string())));
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
                                message: format!("Invalid file type: {value}"),
                            }));
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
                                message: format!("Invalid semantic group: {value}"),
                            }));
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
                                message: format!("Invalid output format: {value}"),
                            }));
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

                // Check if the next argument is a glob pattern and convert it to a filter
                if *i < args.len() && Self::is_glob_pattern(&args[*i]) {
                    // Don't consume the glob pattern here - let it be processed by the main parser loop
                }

                return Ok(Some(TransformCommand {
                    command_type: arg.clone(),
                    old_value: None,
                    new_value: None,
                }));
            }
            _ => {}
        }

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

    fn parse_remove_command(
        args: &[String],
        i: &mut usize,
    ) -> Result<Option<RemoveCommand>, Box<dyn Error>> {
        if *i >= args.len() {
            return Ok(None);
        }

        let arg = &args[*i];

        // Check for rm command
        if arg.to_lowercase() == "rm" {
            *i += 1;
            return Ok(Some(RemoveCommand {
                command_type: "rm".to_string(),
                preview: false, // Will be set based on flags later
            }));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_glob_pattern() {
        // Test glob pattern detection
        assert!(CnpGrammarParser::is_glob_pattern("*.txt"));
        assert!(CnpGrammarParser::is_glob_pattern("file?.log"));
        assert!(CnpGrammarParser::is_glob_pattern("file[0-9].txt"));
        assert!(CnpGrammarParser::is_glob_pattern("file{a,b,c}.txt"));

        // Test non-glob patterns
        assert!(!CnpGrammarParser::is_glob_pattern("file.txt"));
        assert!(!CnpGrammarParser::is_glob_pattern("directory"));
        assert!(!CnpGrammarParser::is_glob_pattern("TYPE:file"));
        assert!(!CnpGrammarParser::is_glob_pattern("EXT:md"));
    }

    #[test]
    fn test_convert_glob_to_filter() -> Result<(), Box<dyn std::error::Error>> {
        // Test extension patterns
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*.txt")?,
            Some(Filter::Extension("txt".to_string()))
        );
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*.org")?,
            Some(Filter::Extension("org".to_string()))
        );

        // Test name patterns
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*filename*")?,
            Some(Filter::Name("filename".to_string()))
        );
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("prefix*")?,
            Some(Filter::Name("prefix*".to_string()))
        );
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*suffix")?,
            Some(Filter::Name("*suffix".to_string()))
        );

        // Test complex patterns
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("file?.txt")?,
            Some(Filter::Name("file?.txt".to_string()))
        );

        Ok(())
    }

    #[test]
    fn test_glob_pattern_parsing() -> Result<(), Box<dyn std::error::Error>> {
        // Test transform command with glob pattern
        let args = vec![
            "title".to_string(),
            "*.org".to_string(),
            "test_dir".to_string(),
        ];
        let result = CnpGrammarParser::parse(&args)?;

        assert_eq!(result.path, "test_dir");
        assert!(result.transform_command.is_some());
        assert_eq!(result.transform_command.unwrap().command_type, "title");
        assert_eq!(result.filters.len(), 1);

        match &result.filters[0] {
            Filter::Extension(ext) => assert_eq!(ext, "org"),
            _ => panic!("Expected Extension filter"),
        }

        Ok(())
    }

    #[test]
    fn test_multiple_glob_patterns() -> Result<(), Box<dyn std::error::Error>> {
        // Test multiple glob patterns
        let args = vec![
            "kebab".to_string(),
            "*.txt".to_string(),
            "*.md".to_string(),
            ".".to_string(),
        ];
        let result = CnpGrammarParser::parse(&args)?;

        assert_eq!(result.path, ".");
        assert!(result.transform_command.is_some());
        assert_eq!(result.transform_command.unwrap().command_type, "kebab");
        assert_eq!(result.filters.len(), 2);

        // Check both extension filters
        let mut has_txt = false;
        let mut has_md = false;
        for filter in &result.filters {
            match filter {
                Filter::Extension(ext) if ext == "txt" => has_txt = true,
                Filter::Extension(ext) if ext == "md" => has_md = true,
                _ => {}
            }
        }
        assert!(has_txt && has_md);

        Ok(())
    }

    #[test]
    fn test_glob_with_cnp_filters() -> Result<(), Box<dyn std::error::Error>> {
        // Test glob pattern mixed with CNP filters
        let args = vec![
            "snake".to_string(),
            "*.rs".to_string(),
            "TYPE:file".to_string(),
            "src".to_string(),
        ];
        let result = CnpGrammarParser::parse(&args)?;

        assert_eq!(result.path, "src");
        assert!(result.transform_command.is_some());
        assert_eq!(result.filters.len(), 2);

        // Check we have both extension and type filters
        let mut has_ext = false;
        let mut has_type = false;
        for filter in &result.filters {
            match filter {
                Filter::Extension(_) => has_ext = true,
                Filter::Type(_) => has_type = true,
                _ => {}
            }
        }
        assert!(has_ext && has_type);

        Ok(())
    }

    #[test]
    fn test_edge_case_glob_patterns() -> Result<(), Box<dyn std::error::Error>> {
        // Test edge cases
        // *. becomes just * prefix matching which converts to a Name filter
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*.*")?,
            Some(Filter::Name(".".to_string()))
        );

        // Complex extension patterns fall back to Name filter
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*.tar.gz")?,
            Some(Filter::Name("*.tar.gz".to_string()))
        );

        // Single * becomes a generic Name filter
        assert_eq!(
            CnpGrammarParser::convert_glob_to_filter("*")?,
            Some(Filter::Name("*".to_string()))
        );

        Ok(())
    }
}
