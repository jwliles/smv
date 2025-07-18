use deunicode::deunicode;
use once_cell::sync::Lazy;
use regex::Regex;

/// Transformation types available for filename conversion
///
/// This enum represents all the different ways a filename can be transformed:
/// - `Clean`: Removes special characters and normalizes spaces
/// - `Snake`: Converts to snake_case (lowercase with underscores)
/// - `Kebab`: Converts to kebab-case (lowercase with hyphens)
/// - `Title`: Converts to Title Case (capitalized words with spaces)
/// - `Camel`: Converts to camelCase (no separators, first word lowercase)
/// - `Pascal`: Converts to PascalCase (no separators, all words capitalized)
/// - `Lower`: Converts to all lowercase
/// - `Upper`: Converts to all uppercase
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformType {
    /// Removes special characters and normalizes spaces
    Clean,
    /// Converts to snake_case (lowercase with underscores)
    Snake,
    /// Converts to kebab-case (lowercase with hyphens)
    Kebab,
    /// Converts to Title Case (capitalized words with spaces)
    Title,
    /// Converts to camelCase (no separators, first word lowercase)
    Camel,
    /// Converts to PascalCase (no separators, all words capitalized)
    Pascal,
    /// Converts to all lowercase
    Lower,
    /// Converts to all uppercase
    Upper,
    /// Replace substring (find, replace)
    Replace(String, String),
    /// Replace using regex pattern (pattern, replacement)
    ReplaceRegex(String, String),
}

impl TransformType {
    /// Convert a string representation to TransformType
    ///
    /// This method attempts to parse a string into a TransformType.
    /// It is case-insensitive for better user experience.
    ///
    /// # Arguments
    /// * `s` - The string to parse
    ///
    /// # Returns
    /// * `Some(TransformType)` if the string matches a known transformation type
    /// * `None` if the string doesn't match any known transformation type
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "clean" => Some(TransformType::Clean),
            "snake" => Some(TransformType::Snake),
            "kebab" => Some(TransformType::Kebab),
            "title" => Some(TransformType::Title),
            "camel" => Some(TransformType::Camel),
            "pascal" => Some(TransformType::Pascal),
            "lower" => Some(TransformType::Lower),
            "upper" => Some(TransformType::Upper),
            _ => None,
        }
    }

    /// Create a Replace transformation from find and replace strings
    pub fn replace(find: &str, replace: &str) -> Self {
        TransformType::Replace(find.to_string(), replace.to_string())
    }

    /// Create a ReplaceRegex transformation from pattern and replacement strings
    pub fn replace_regex(pattern: &str, replacement: &str) -> Self {
        TransformType::ReplaceRegex(pattern.to_string(), replacement.to_string())
    }

    /// Get string representation of the transform type
    ///
    /// This method returns the string representation of a TransformType.
    /// Useful for printing or serializing.
    ///
    /// # Returns
    /// A string representing the transformation type
    pub fn as_str(&self) -> String {
        match self {
            TransformType::Clean => "clean".to_string(),
            TransformType::Snake => "snake".to_string(),
            TransformType::Kebab => "kebab".to_string(),
            TransformType::Title => "title".to_string(),
            TransformType::Camel => "camel".to_string(),
            TransformType::Pascal => "pascal".to_string(),
            TransformType::Lower => "lower".to_string(),
            TransformType::Upper => "upper".to_string(),
            TransformType::Replace(find, replace) => format!("replace({} → {})", find, replace),
            TransformType::ReplaceRegex(pattern, replacement) => {
                format!("replace-regex({} → {})", pattern, replacement)
            }
        }
    }
}

// Regular expressions used for transformations
static SPECIAL_CHARS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\w\s.-]").unwrap());
static MULTIPLE_SPACES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static LEADING_TRAILING_SPECIALS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[-\s.]+|[-\s.]+$").unwrap());
static WORD_SEPARATORS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s_.-]+").unwrap());
static WORD_SEPARATORS_WITH_DOTS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s_.-]+").unwrap());

/// Transform a filename according to the specified transformation type
///
/// This is the main public function for transforming filenames. It dispatches
/// to the appropriate transformation function based on the specified transformation type.
/// It preserves file extensions for case transformations.
///
/// # Arguments
/// * `name` - The filename string to transform
/// * `transform_type` - The type of transformation to apply
///
/// # Returns
/// A new string transformed according to the specified transformation type
pub fn transform(name: &str, transform_type: &TransformType) -> String {
    match transform_type {
        TransformType::Clean => clean(name),
        TransformType::Snake => snake_case_preserve_extension(name),
        TransformType::Kebab => kebab_case_preserve_extension(name),
        TransformType::Title => title_case_preserve_extension(name),
        TransformType::Camel => camel_case_preserve_extension(name),
        TransformType::Pascal => pascal_case_preserve_extension(name),
        TransformType::Lower => name.to_lowercase(),
        TransformType::Upper => name.to_uppercase(),
        TransformType::Replace(find, replace) => replace_substring(name, find, replace),
        TransformType::ReplaceRegex(pattern, replacement) => {
            replace_regex(name, pattern, replacement)
        }
    }
}

/// Clean a filename by removing special characters and normalizing spaces
///
/// This function performs basic cleaning operations:
/// 1. Trimming leading and trailing whitespace
/// 2. Normalizing multiple spaces to single spaces
/// 3. Removing special characters
/// 4. Removing leading and trailing special characters
///
/// # Arguments
/// * `name` - The filename string to clean
///
/// # Returns
/// A cleaned string with normalized spacing and no special characters
fn clean(name: &str) -> String {
    let trimmed = name.trim();
    let normalized_spaces = MULTIPLE_SPACES_RE.replace_all(trimmed, " ");
    let no_special_chars = SPECIAL_CHARS_RE.replace_all(&normalized_spaces, "");
    LEADING_TRAILING_SPECIALS_RE
        .replace_all(&no_special_chars, "")
        .to_string()
}

/// Convert a filename to snake_case
///
/// This function handles conversion of a string to snake_case by:
/// 1. Converting CamelCase transitions to snake_case
/// 2. Converting spaces and hyphens to underscores
/// 3. Converting all characters to lowercase
///
/// # Arguments
/// * `name` - The filename string to transform
///
/// # Returns
/// A new string in snake_case format
fn snake_case(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_snake(&tokens)
}

/// Convert a filename to snake_case while preserving the file extension
fn snake_case_preserve_extension(name: &str) -> String {
    if let Some(dot_pos) = name.rfind('.') {
        if dot_pos > 0 && dot_pos < name.len() - 1 {
            // File has an extension
            let (basename, extension) = name.split_at(dot_pos);
            let transformed_basename = snake_case(basename);
            let transformed_extension = extension[1..].to_lowercase(); // Remove the dot and lowercase extension
            format!("{}.{}", transformed_basename, transformed_extension)
        } else {
            // Dot at beginning or end, treat as regular filename
            snake_case(name)
        }
    } else {
        // No extension
        snake_case(name)
    }
}

/// Convert a filename to kebab-case
///
/// This function handles conversion of a string to kebab-case by:
/// 1. Converting CamelCase transitions to kebab-case
/// 2. Converting spaces and underscores to hyphens
/// 3. Converting all characters to lowercase
///
/// # Arguments
/// * `name` - The filename string to transform
///
/// # Returns
/// A new string in kebab-case format
fn kebab_case(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_kebab(&tokens)
}

/// Convert a filename to kebab-case while preserving the file extension
fn kebab_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, kebab_case)
}

/// Helper function to apply a transformation while preserving file extension
fn preserve_extension_transform<F>(name: &str, transform_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    if let Some(dot_pos) = name.rfind('.') {
        if dot_pos > 0 && dot_pos < name.len() - 1 {
            // File has an extension
            let (basename, extension) = name.split_at(dot_pos);
            let transformed_basename = transform_fn(basename);
            let transformed_extension = extension[1..].to_lowercase(); // Remove the dot and lowercase extension
            format!("{}.{}", transformed_basename, transformed_extension)
        } else {
            // Dot at beginning or end, treat as regular filename
            transform_fn(name)
        }
    } else {
        // No extension
        transform_fn(name)
    }
}

/// Convert a filename to Title Case
///
/// This function handles conversion of a string to Title Case by:
/// 1. Splitting the string by common word separators (spaces, underscores, hyphens)
/// 2. Capitalizing the first letter of each word
/// 3. Joining the words with spaces
///
/// # Arguments
/// * `name` - The filename string to transform
///
/// # Returns
/// A new string in Title Case format
fn title_case(name: &str) -> String {
    let tokens = tokenize(name, true);
    format_title(&tokens)
}

/// Convert a filename to Title Case while preserving the file extension
fn title_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, title_case)
}

/// Convert a filename to camelCase
///
/// This function handles conversion of a string to camelCase by:
/// 1. Splitting the string by common word separators (spaces, underscores, hyphens)
/// 2. Converting the first word to lowercase
/// 3. Capitalizing the first letter of each subsequent word
/// 4. Joining all words without separators
///
/// # Arguments
/// * `name` - The filename string to transform
///
/// # Returns
/// A new string in camelCase format
fn camel_case(name: &str) -> String {
    let tokens = tokenize(name, true);
    format_camel(&tokens)
}

/// Convert a filename to camelCase while preserving the file extension
fn camel_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, camel_case)
}

/// Convert a filename to PascalCase
///
/// This function handles conversion of a string to PascalCase by:
/// 1. Splitting the string by common word separators (spaces, underscores, hyphens)
/// 2. Capitalizing the first letter of each word
/// 3. Joining all words without separators
///
/// # Arguments
/// * `name` - The filename string to transform
///
/// # Returns
/// A new string in PascalCase format
fn pascal_case(name: &str) -> String {
    let tokens = tokenize(name, true);
    format_pascal(&tokens)
}

/// Convert a filename to PascalCase while preserving the file extension
fn pascal_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, pascal_case)
}

/// Tokenize a string into constituent words, handling all separators and camelCase
fn tokenize(name: &str, include_dots: bool) -> Vec<String> {
    // Apply unicode normalization first
    let normalized = deunicode(name);

    let separator_regex = if include_dots {
        &WORD_SEPARATORS_WITH_DOTS_RE
    } else {
        &WORD_SEPARATORS_RE
    };

    separator_regex
        .split(&normalized)
        .filter(|s| !s.is_empty())
        .flat_map(|word| split_camel_case_word(word))
        .collect()
}

/// Helper function to split a word into constituent words, handling camelCase
fn split_camel_case_word(word: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let chars: Vec<char> = word.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase()
            && i > 0
            && (
                // Previous char is lowercase or digit
                chars[i-1].is_lowercase() || chars[i-1].is_ascii_digit() ||
            // Current char is followed by lowercase (handles XMLDocument -> XML Document)
            (i + 1 < chars.len() && chars[i+1].is_lowercase())
            )
        {
            if !current_word.is_empty() {
                result.push(current_word.clone());
                current_word.clear();
            }
        }
        current_word.push(ch);
    }

    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}

/// Format tokens as snake_case
fn format_snake(tokens: &[String]) -> String {
    tokens.join("_").replace('-', "_").to_lowercase()
}

/// Format tokens as kebab-case
fn format_kebab(tokens: &[String]) -> String {
    tokens.join("-").replace('_', "-").to_lowercase()
}

/// Format tokens as camelCase
fn format_camel(tokens: &[String]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let mut result = tokens[0].to_lowercase();
    for token in tokens.iter().skip(1) {
        result.push_str(&capitalize_first(token));
    }
    result
}

/// Format tokens as PascalCase
fn format_pascal(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join("")
}

/// Format tokens as Title Case
fn format_title(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Helper function to capitalize the first letter of a string
///
/// # Arguments
/// * `s` - The string to capitalize
///
/// # Returns
/// A new string with the first letter capitalized and the rest unchanged
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Replace substring in a filename
///
/// This function performs exact string replacement on the filename.
/// It replaces all occurrences of the find string with the replace string.
///
/// # Arguments
/// * `name` - The filename string to transform
/// * `find` - The substring to find
/// * `replace` - The substring to replace with
///
/// # Returns
/// A new string with all occurrences of `find` replaced with `replace`
fn replace_substring(name: &str, find: &str, replace: &str) -> String {
    name.replace(find, replace)
}

/// Replace using regex pattern in a filename
///
/// This function performs regex-based replacement on the filename.
/// It replaces all matches of the regex pattern with the replacement string.
///
/// # Arguments
/// * `name` - The filename string to transform
/// * `pattern` - The regex pattern to match
/// * `replacement` - The replacement string (can include capture groups like $1, $2)
///
/// # Returns
/// A new string with all pattern matches replaced, or the original string if regex is invalid
fn replace_regex(name: &str, pattern: &str, replacement: &str) -> String {
    match Regex::new(pattern) {
        Ok(re) => re.replace_all(name, replacement).to_string(),
        Err(_) => {
            eprintln!(
                "Warning: Invalid regex pattern '{}', returning original string",
                pattern
            );
            name.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean() {
        assert_eq!(clean("  My File (1) !!  "), "My File 1");
        assert_eq!(clean("..leading-dots"), "leading-dots");
        assert_eq!(clean("trailing-dots.."), "trailing-dots");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(snake_case("HelloWorld"), "hello_world");
        assert_eq!(snake_case("My-File.txt"), "my_file_txt"); // No extension preservation in direct function
        assert_eq!(snake_case("already_snake"), "already_snake");
        assert_eq!(snake_case("Words With Spaces"), "words_with_spaces");
        assert_eq!(
            snake_case("Mix-of spaces_and-hyphens"),
            "mix_of_spaces_and_hyphens"
        );

        // Test the extension-preserving version
        assert_eq!(snake_case_preserve_extension("My-File.txt"), "my_file.txt");
        assert_eq!(
            snake_case_preserve_extension("HelloWorld.pdf"),
            "hello_world.pdf"
        );
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(kebab_case("HelloWorld"), "hello-world");
        assert_eq!(kebab_case("My_File.txt"), "my-file-txt"); // No extension preservation in direct function
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
        assert_eq!(kebab_case("Words With Spaces"), "words-with-spaces");
        assert_eq!(
            kebab_case("Mix-of spaces_and_underscores"),
            "mix-of-spaces-and-underscores"
        );
        // Test the specific bug case from BUGS.md - no extension preservation in direct function
        assert_eq!(kebab_case("Dir Template.txt"), "dir-template-txt");

        // Test the extension-preserving version
        assert_eq!(kebab_case_preserve_extension("My_File.txt"), "my-file.txt");
        assert_eq!(
            kebab_case_preserve_extension("Dir Template.txt"),
            "dir-template.txt"
        );
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("hello_world"), "Hello World");
        assert_eq!(title_case("my-file.txt"), "My File Txt");
        assert_eq!(title_case("already Title Case"), "Already Title Case");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(camel_case("hello_world"), "helloWorld");
        assert_eq!(camel_case("my-file.txt"), "myFileTxt");
        assert_eq!(camel_case("Words With Spaces"), "wordsWithSpaces");
        assert_eq!(camel_case("multiple   spaces"), "multipleSpaces");
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(pascal_case("hello_world"), "HelloWorld");
        assert_eq!(pascal_case("my-file.txt"), "MyFileTxt");
        assert_eq!(pascal_case("Words With Spaces"), "WordsWithSpaces");
        assert_eq!(pascal_case("multiple   spaces"), "MultipleSpaces");
    }

    #[test]
    fn test_replace_substring() {
        assert_eq!(
            replace_substring("hello_world.txt", "hello", "hi"),
            "hi_world.txt"
        );
        assert_eq!(
            replace_substring("AFN_project.rs", "AFN", "CNP"),
            "CNP_project.rs"
        );
        assert_eq!(
            replace_substring("test_AFN_file.txt", "AFN", "CNP"),
            "test_CNP_file.txt"
        );
        assert_eq!(
            replace_substring("no_match.txt", "xyz", "abc"),
            "no_match.txt"
        );
        assert_eq!(
            replace_substring("multiple_AFN_AFN.txt", "AFN", "CNP"),
            "multiple_CNP_CNP.txt"
        );
    }

    #[test]
    fn test_replace_regex() {
        assert_eq!(replace_regex("file123.txt", r"\d+", "456"), "file456.txt");
        assert_eq!(
            replace_regex("AFN_project_v1.rs", r"AFN", "CNP"),
            "CNP_project_v1.rs"
        );
        assert_eq!(
            replace_regex("test_file_2023.txt", r"\d{4}", "2024"),
            "test_file_2024.txt"
        );
        assert_eq!(
            replace_regex("CamelCase.txt", r"([A-Z])", "_$1"),
            "_Camel_Case.txt"
        );
        assert_eq!(
            replace_regex("invalid[regex.txt", r"[", "replacement"),
            "invalid[regex.txt"
        );
    }

    #[test]
    fn test_transform_replace() {
        let replace_transform = TransformType::Replace("AFN".to_string(), "CNP".to_string());
        assert_eq!(
            transform("AFN_project.rs", &replace_transform),
            "CNP_project.rs"
        );

        let regex_transform = TransformType::ReplaceRegex(r"\d+".to_string(), "XXX".to_string());
        assert_eq!(transform("file123.txt", &regex_transform), "fileXXX.txt");
    }
}
