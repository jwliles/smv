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
    /// Converts to Sentence case (only first word capitalized)
    Sentence,
    /// Converts to Start Case (all words capitalized with spaces)
    Start,
    /// Converts to StudlyCaps (alternating case)
    Studly,
    /// Replace substring (find, replace)
    Replace(String, String),
    /// Replace using regex pattern (pattern, replacement)
    ReplaceRegex(String, String),
    /// Remove prefix from filename
    RemovePrefix(String),
    /// Split camelCase/PascalCase and convert to snake_case
    SplitSnake,
    /// Split camelCase/PascalCase and convert to kebab-case
    SplitKebab,
    /// Split camelCase/PascalCase and convert to Title Case
    SplitTitle,
    /// Split camelCase/PascalCase and convert to camelCase
    SplitCamel,
    /// Split camelCase/PascalCase and convert to PascalCase
    SplitPascal,
    /// Split camelCase/PascalCase and convert to lowercase
    SplitLower,
    /// Split camelCase/PascalCase and convert to uppercase
    SplitUpper,
    /// Split camelCase/PascalCase and convert to Sentence case
    SplitSentence,
    /// Split camelCase/PascalCase and convert to Start Case
    SplitStart,
    /// Split camelCase/PascalCase and convert to StudlyCaps
    SplitStudly,
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
            "sentence" => Some(TransformType::Sentence),
            "start" => Some(TransformType::Start),
            "studly" => Some(TransformType::Studly),
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

    /// Create a RemovePrefix transformation from prefix string
    pub fn remove_prefix(prefix: &str) -> Self {
        TransformType::RemovePrefix(prefix.to_string())
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
            TransformType::Sentence => "sentence".to_string(),
            TransformType::Start => "start".to_string(),
            TransformType::Studly => "studly".to_string(),
            TransformType::Replace(find, replace) => format!("replace({find} → {replace})"),
            TransformType::ReplaceRegex(pattern, replacement) => {
                format!("replace-regex({pattern} → {replacement})")
            }
            TransformType::RemovePrefix(prefix) => format!("remove-prefix({prefix})"),
            TransformType::SplitSnake => "split-snake".to_string(),
            TransformType::SplitKebab => "split-kebab".to_string(),
            TransformType::SplitTitle => "split-title".to_string(),
            TransformType::SplitCamel => "split-camel".to_string(),
            TransformType::SplitPascal => "split-pascal".to_string(),
            TransformType::SplitLower => "split-lower".to_string(),
            TransformType::SplitUpper => "split-upper".to_string(),
            TransformType::SplitSentence => "split-sentence".to_string(),
            TransformType::SplitStart => "split-start".to_string(),
            TransformType::SplitStudly => "split-studly".to_string(),
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
        TransformType::Sentence => sentence_case_preserve_extension(name),
        TransformType::Start => start_case_preserve_extension(name),
        TransformType::Studly => studly_caps_preserve_extension(name),
        TransformType::Replace(find, replace) => replace_substring(name, find, replace),
        TransformType::ReplaceRegex(pattern, replacement) => {
            replace_regex(name, pattern, replacement)
        }
        TransformType::RemovePrefix(prefix) => remove_prefix(name, prefix),
        TransformType::SplitSnake => split_and_transform(name, TransformType::Snake),
        TransformType::SplitKebab => split_and_transform(name, TransformType::Kebab),
        TransformType::SplitTitle => split_and_transform(name, TransformType::Title),
        TransformType::SplitCamel => split_and_transform(name, TransformType::Camel),
        TransformType::SplitPascal => split_and_transform(name, TransformType::Pascal),
        TransformType::SplitLower => split_and_transform(name, TransformType::Lower),
        TransformType::SplitUpper => split_and_transform(name, TransformType::Upper),
        TransformType::SplitSentence => split_and_transform(name, TransformType::Sentence),
        TransformType::SplitStart => split_and_transform(name, TransformType::Start),
        TransformType::SplitStudly => split_and_transform(name, TransformType::Studly),
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
            format!("{transformed_basename}.{transformed_extension}")
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
            format!("{transformed_basename}.{transformed_extension}")
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
    let tokens = tokenize(name, false);
    format_title(&tokens)
}

/// Convert a filename to Title Case for filenames (no spaces)
fn title_case_filename(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_title_filename(&tokens)
}

/// Convert a filename to Title Case while preserving the file extension
fn title_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, title_case_filename)
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

/// Convert a filename to Sentence case for filenames
fn sentence_case_filename(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_sentence(&tokens)
}

/// Convert a filename to Sentence case while preserving the file extension
fn sentence_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, sentence_case_filename)
}

/// Convert a filename to Start Case for filenames
fn start_case_filename(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_start(&tokens)
}

/// Convert a filename to Start Case while preserving the file extension
fn start_case_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, start_case_filename)
}

/// Convert a filename to StudlyCaps for filenames
fn studly_caps_filename(name: &str) -> String {
    let tokens = tokenize(name, false);
    format_studly(&tokens)
}

/// Convert a filename to StudlyCaps while preserving the file extension
fn studly_caps_preserve_extension(name: &str) -> String {
    preserve_extension_transform(name, studly_caps_filename)
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
        .flat_map(split_camel_case_word)
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
            && !current_word.is_empty()
        {
            result.push(current_word.clone());
            current_word.clear();
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

/// Format tokens as Title Case (with spaces for regular text)
fn format_title(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Format tokens as Title Case for filenames (without spaces)
fn format_title_filename(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join("")
}

/// Format tokens as Sentence case (only first word capitalized, rest lowercase)
fn format_sentence(tokens: &[String]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let mut result = capitalize_first(&tokens[0]);
    for token in &tokens[1..] {
        result.push_str(&token.to_lowercase());
    }
    result
}

/// Format tokens as Start Case (all words capitalized with spaces)
fn format_start(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Format tokens as StudlyCaps (alternating case)
fn format_studly(tokens: &[String]) -> String {
    let full_text = tokens.join("");
    let mut result = String::new();
    let mut letter_count = 0;

    for ch in full_text.chars() {
        if ch.is_alphabetic() {
            if letter_count % 2 == 0 {
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            } else {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
            }
            letter_count += 1;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Helper function to capitalize the first letter of a string
///
/// # Arguments
/// * `s` - The string to capitalize
///
/// # Returns
/// A new string with the first letter capitalized and the rest lowercased
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
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
            eprintln!("Warning: Invalid regex pattern '{pattern}', returning original string");
            name.to_string()
        }
    }
}

/// Remove prefix from a filename
///
/// This function removes a specified prefix from the beginning of a filename.
/// If the filename doesn't start with the prefix, it returns the original filename unchanged.
/// This is particularly useful for batch operations on files with common prefixes.
///
/// # Arguments
/// * `name` - The filename string to transform
/// * `prefix` - The prefix string to remove from the beginning
///
/// # Returns
/// A new string with the prefix removed, or the original string if it doesn't start with the prefix
fn remove_prefix(name: &str, prefix: &str) -> String {
    if let Some(stripped) = name.strip_prefix(prefix) {
        stripped.to_string()
    } else {
        name.to_string()
    }
}

/// Split camelCase/PascalCase text at word boundaries
fn split_camel_case_boundaries(text: &str) -> Vec<String> {
    // Use regex to find word boundaries in camelCase/PascalCase
    static CAMEL_SPLIT_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([a-z])([A-Z])|([A-Z]+)([A-Z][a-z])").unwrap());

    // Insert spaces at word boundaries
    let spaced = CAMEL_SPLIT_RE.replace_all(text, "$1$3 $2$4");

    // Split on spaces and filter empty strings
    spaced.split_whitespace().map(|s| s.to_string()).collect()
}

/// Split camelCase/PascalCase and apply transformation with extension preservation
fn split_and_transform(name: &str, transform_type: TransformType) -> String {
    if let Some(dot_pos) = name.rfind('.') {
        if dot_pos > 0 && dot_pos < name.len() - 1 {
            // File has an extension
            let (basename, extension) = name.split_at(dot_pos);
            let transformed_basename = split_and_transform_basename(basename, transform_type);
            let transformed_extension = extension[1..].to_lowercase(); // Remove the dot and lowercase extension
            format!("{transformed_basename}.{transformed_extension}")
        } else {
            // Dot at beginning or end, treat as regular filename
            split_and_transform_basename(name, transform_type)
        }
    } else {
        // No extension
        split_and_transform_basename(name, transform_type)
    }
}

/// Split camelCase/PascalCase basename and apply transformation
fn split_and_transform_basename(basename: &str, transform_type: TransformType) -> String {
    // Split at camelCase/PascalCase boundaries
    let words = split_camel_case_boundaries(basename);

    // If no boundaries found, fall back to regular transformation
    if words.len() <= 1 {
        return transform(basename, &transform_type);
    }

    // Join words with appropriate separators and apply transformation
    let joined = match transform_type {
        TransformType::Snake => words.join("_").to_lowercase(),
        TransformType::Kebab => words.join("-").to_lowercase(),
        TransformType::Lower => words.join("").to_lowercase(),
        TransformType::Upper => words.join("").to_uppercase(),
        _ => {
            // For other transformations, join with spaces and apply transformation
            let joined_with_spaces = words.join(" ");
            transform(&joined_with_spaces, &transform_type)
        }
    };

    joined
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

    #[test]
    fn test_remove_prefix() {
        assert_eq!(remove_prefix("prefix_file.txt", "prefix_"), "file.txt");
        assert_eq!(remove_prefix("IMG_1234.jpg", "IMG_"), "1234.jpg");
        assert_eq!(remove_prefix("DSC_9876.png", "DSC_"), "9876.png");
        assert_eq!(remove_prefix("no_match.txt", "prefix_"), "no_match.txt");
        assert_eq!(remove_prefix("prefix_", "prefix_"), "");
        assert_eq!(remove_prefix("", "prefix_"), "");
        assert_eq!(remove_prefix("file.txt", ""), "file.txt");
    }

    #[test]
    fn test_sentence_case() {
        assert_eq!(sentence_case_filename("hello_world"), "Helloworld");
        assert_eq!(sentence_case_filename("HELLO_WORLD"), "Helloworld");
        assert_eq!(sentence_case_filename("HelloWorld"), "Helloworld");
        assert_eq!(
            sentence_case_filename("multiple words here"),
            "Multiplewordshere"
        );
    }

    #[test]
    fn test_start_case() {
        assert_eq!(start_case_filename("hello_world"), "Hello World");
        assert_eq!(start_case_filename("HELLO_WORLD"), "Hello World");
        assert_eq!(start_case_filename("HelloWorld"), "Hello World");
        assert_eq!(
            start_case_filename("multiple words here"),
            "Multiple Words Here"
        );
    }

    #[test]
    fn test_studly_caps() {
        assert_eq!(studly_caps_filename("hello_world"), "hElLoWoRlD");
        assert_eq!(studly_caps_filename("HELLO_WORLD"), "hElLoWoRlD");
        assert_eq!(studly_caps_filename("HelloWorld"), "hElLoWoRlD");
        assert_eq!(studly_caps_filename("abc"), "aBc");
        assert_eq!(studly_caps_filename("a"), "a");
    }

    #[test]
    fn test_transform_remove_prefix() {
        let remove_prefix_transform = TransformType::RemovePrefix("IMG_".to_string());
        assert_eq!(
            transform("IMG_1234.jpg", &remove_prefix_transform),
            "1234.jpg"
        );
        assert_eq!(
            transform("no_prefix.jpg", &remove_prefix_transform),
            "no_prefix.jpg"
        );
    }

    #[test]
    fn test_split_camel_case_boundaries() {
        assert_eq!(
            split_camel_case_boundaries("featureWishList"),
            vec!["feature", "Wish", "List"]
        );
        assert_eq!(
            split_camel_case_boundaries("XMLDocument"),
            vec!["XML", "Document"]
        );
        assert_eq!(
            split_camel_case_boundaries("apiEndpoint"),
            vec!["api", "Endpoint"]
        );
        assert_eq!(
            split_camel_case_boundaries("HelloWorld"),
            vec!["Hello", "World"]
        );
        assert_eq!(
            split_camel_case_boundaries("camelCase"),
            vec!["camel", "Case"]
        );
        assert_eq!(split_camel_case_boundaries("lowercase"), vec!["lowercase"]);
        assert_eq!(split_camel_case_boundaries("UPPERCASE"), vec!["UPPERCASE"]);
    }

    #[test]
    fn test_split_transformations() {
        // Test split snake
        assert_eq!(
            transform("featureWishList.md", &TransformType::SplitSnake),
            "feature_wish_list.md"
        );
        assert_eq!(
            transform("XMLDocument.xml", &TransformType::SplitSnake),
            "xml_document.xml"
        );
        assert_eq!(
            transform("apiEndpoint.ts", &TransformType::SplitSnake),
            "api_endpoint.ts"
        );

        // Test split kebab
        assert_eq!(
            transform("FeatureWishList.txt", &TransformType::SplitKebab),
            "feature-wish-list.txt"
        );
        assert_eq!(
            transform("UserSettings.json", &TransformType::SplitKebab),
            "user-settings.json"
        );

        // Test split title
        assert_eq!(
            transform("myFeatureList.md", &TransformType::SplitTitle),
            "MyFeatureList.md"
        );
        assert_eq!(
            transform("userSettings.js", &TransformType::SplitTitle),
            "UserSettings.js"
        );

        // Test split camel
        assert_eq!(
            transform("UserSettings.json", &TransformType::SplitCamel),
            "userSettings.json"
        );
        assert_eq!(
            transform("FeatureList.py", &TransformType::SplitCamel),
            "featureList.py"
        );

        // Test split pascal
        assert_eq!(
            transform("userSettings.js", &TransformType::SplitPascal),
            "UserSettings.js"
        );
        assert_eq!(
            transform("apiClient.rb", &TransformType::SplitPascal),
            "ApiClient.rb"
        );

        // Test split lower
        assert_eq!(
            transform("XMLDocument.xml", &TransformType::SplitLower),
            "xmldocument.xml"
        );
        assert_eq!(
            transform("dataProcessor.cpp", &TransformType::SplitLower),
            "dataprocessor.cpp"
        );

        // Test split upper
        assert_eq!(
            transform("dataProcessor.cpp", &TransformType::SplitUpper),
            "DATAPROCESSOR.cpp"
        );
        assert_eq!(
            transform("apiClient.js", &TransformType::SplitUpper),
            "APICLIENT.js"
        );

        // Test split sentence
        assert_eq!(
            transform("HelloWorld.py", &TransformType::SplitSentence),
            "Helloworld.py"
        );
        assert_eq!(
            transform("featureList.md", &TransformType::SplitSentence),
            "Featurelist.md"
        );

        // Test split start
        assert_eq!(
            transform("todoList.md", &TransformType::SplitStart),
            "Todo List.md"
        );
        assert_eq!(
            transform("userProfile.html", &TransformType::SplitStart),
            "User Profile.html"
        );

        // Test split studly
        assert_eq!(
            transform("HelloWorld.rb", &TransformType::SplitStudly),
            "hElLoWoRlD.rb"
        );
        assert_eq!(
            transform("apiClient.js", &TransformType::SplitStudly),
            "aPiClIeNt.js"
        );
    }

    #[test]
    fn test_split_no_boundaries() {
        // Files without camelCase boundaries should fall back to regular transformation
        assert_eq!(
            transform("lowercase.txt", &TransformType::SplitSnake),
            "lowercase.txt"
        );
        assert_eq!(
            transform("UPPERCASE.txt", &TransformType::SplitKebab),
            "uppercase.txt"
        );
        assert_eq!(
            transform("already-kebab.txt", &TransformType::SplitSnake),
            "already_kebab.txt"
        );
    }
}
