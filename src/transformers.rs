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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Get string representation of the transform type
    ///
    /// This method returns the string representation of a TransformType.
    /// Useful for printing or serializing.
    ///
    /// # Returns
    /// A string representing the transformation type
    pub fn as_str(&self) -> &'static str {
        match self {
            TransformType::Clean => "clean",
            TransformType::Snake => "snake",
            TransformType::Kebab => "kebab",
            TransformType::Title => "title",
            TransformType::Camel => "camel",
            TransformType::Pascal => "pascal",
            TransformType::Lower => "lower",
            TransformType::Upper => "upper",
        }
    }
}

// Regular expressions used for transformations
static SPECIAL_CHARS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\w\s.-]").unwrap());
static MULTIPLE_SPACES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static LEADING_TRAILING_SPECIALS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[-\s.]+|[-\s.]+$").unwrap());
static WORD_SEPARATORS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s_-]+").unwrap());
static WORD_SEPARATORS_WITH_DOTS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s_.-]+").unwrap());

/// Transform a filename according to the specified transformation type
///
/// This is the main public function for transforming filenames. It dispatches
/// to the appropriate transformation function based on the specified transformation type.
///
/// # Arguments
/// * `name` - The filename string to transform
/// * `transform_type` - The type of transformation to apply
///
/// # Returns
/// A new string transformed according to the specified transformation type
pub fn transform(name: &str, transform_type: TransformType) -> String {
    match transform_type {
        TransformType::Clean => clean(name),
        TransformType::Snake => snake_case(name),
        TransformType::Kebab => kebab_case(name),
        TransformType::Title => title_case(name),
        TransformType::Camel => camel_case(name),
        TransformType::Pascal => pascal_case(name),
        TransformType::Lower => name.to_lowercase(),
        TransformType::Upper => name.to_uppercase(),
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

/// Tokenize a string into constituent words, handling all separators and camelCase
fn tokenize(name: &str, include_dots: bool) -> Vec<String> {
    let separator_regex = if include_dots {
        &WORD_SEPARATORS_WITH_DOTS_RE
    } else {
        &WORD_SEPARATORS_RE
    };
    
    separator_regex
        .split(name)
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
        if ch.is_uppercase() && i > 0 && (
            // Previous char is lowercase or digit
            chars[i-1].is_lowercase() || chars[i-1].is_ascii_digit() ||
            // Current char is followed by lowercase (handles XMLDocument -> XML Document)
            (i + 1 < chars.len() && chars[i+1].is_lowercase())
        ) {
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
    tokens.iter()
        .map(|token| capitalize_first(token))
        .collect::<Vec<String>>()
        .join("")
}

/// Format tokens as Title Case
fn format_title(tokens: &[String]) -> String {
    tokens.iter()
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
        assert_eq!(snake_case("My-File.txt"), "my_file.txt");
        assert_eq!(snake_case("already_snake"), "already_snake");
        assert_eq!(snake_case("Words With Spaces"), "words_with_spaces");
        assert_eq!(
            snake_case("Mix-of spaces_and-hyphens"),
            "mix_of_spaces_and_hyphens"
        );
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(kebab_case("HelloWorld"), "hello-world");
        assert_eq!(kebab_case("My_File.txt"), "my-file.txt");
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
        assert_eq!(kebab_case("Words With Spaces"), "words-with-spaces");
        assert_eq!(
            kebab_case("Mix-of spaces_and_underscores"),
            "mix-of-spaces-and-underscores"
        );
        // Test the specific bug case from BUGS.md
        assert_eq!(kebab_case("Dir Template.txt"), "dir-template.txt");
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
}
