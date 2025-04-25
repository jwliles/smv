//! Basic filename transformations
//!
//! This module provides the core transformation types for SMV.

use once_cell::sync::Lazy;
use regex::Regex;
use super::Transformation;

/// Transformation types available for filename conversion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformType {
    Clean,
    Snake,
    Kebab,
    Title,
    Camel,
    Pascal,
    Lower,
    Upper,
}

impl TransformType {
    /// Convert a string representation to TransformType
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
static UPPERCASE_FOLLOWED_BY_LOWERCASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([A-Z]+)([A-Z][a-z])").unwrap());
static LOWERCASE_FOLLOWED_BY_UPPERCASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([a-z\d])([A-Z])").unwrap());
static WORD_SEPARATORS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s_-]+").unwrap());

/// Transform a filename according to the specified transformation type
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
fn clean(name: &str) -> String {
    let trimmed = name.trim();
    let normalized_spaces = MULTIPLE_SPACES_RE.replace_all(trimmed, " ");
    let no_special_chars = SPECIAL_CHARS_RE.replace_all(&normalized_spaces, "");
    LEADING_TRAILING_SPECIALS_RE
        .replace_all(&no_special_chars, "")
        .to_string()
}

/// Convert a filename to snake_case
fn snake_case(name: &str) -> String {
    let with_slashes = name.replace("::", "/");
    let first_transform = UPPERCASE_FOLLOWED_BY_LOWERCASE_RE.replace_all(&with_slashes, "$1_$2");
    let second_transform =
        LOWERCASE_FOLLOWED_BY_UPPERCASE_RE.replace_all(&first_transform, "$1_$2");
    second_transform.replace('-', "_").to_lowercase()
}

/// Convert a filename to kebab-case
fn kebab_case(name: &str) -> String {
    let with_slashes = name.replace("::", "/");
    let first_transform = UPPERCASE_FOLLOWED_BY_LOWERCASE_RE.replace_all(&with_slashes, "$1-$2");
    let second_transform =
        LOWERCASE_FOLLOWED_BY_UPPERCASE_RE.replace_all(&first_transform, "$1-$2");
    second_transform.replace('_', "-").to_lowercase()
}

/// Convert a filename to Title Case
fn title_case(name: &str) -> String {
    WORD_SEPARATORS_RE
        .split(name)
        .filter(|s| !s.is_empty())
        .map(capitalize_first)
        .collect::<Vec<String>>()
        .join(" ")
}

/// Convert a filename to camelCase
fn camel_case(name: &str) -> String {
    let words = WORD_SEPARATORS_RE
        .split(name)
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in words.iter().skip(1) {
        result.push_str(&capitalize_first(word));
    }

    result
}

/// Convert a filename to PascalCase
fn pascal_case(name: &str) -> String {
    WORD_SEPARATORS_RE
        .split(name)
        .filter(|s| !s.is_empty())
        .map(capitalize_first)
        .collect::<Vec<String>>()
        .join("")
}

/// Helper function to capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// Implementation of transformations using the Transformation trait

/// Clean transformation implementation
pub struct CleanTransform;

impl Transformation for CleanTransform {
    fn transform(&self, input: &str) -> String {
        clean(input)
    }
    
    fn name(&self) -> &str {
        "clean"
    }
    
    fn description(&self) -> &str {
        "Remove special characters and normalize spaces"
    }
}

/// Snake case transformation implementation
pub struct SnakeCaseTransform;

impl Transformation for SnakeCaseTransform {
    fn transform(&self, input: &str) -> String {
        snake_case(input)
    }
    
    fn name(&self) -> &str {
        "snake"
    }
    
    fn description(&self) -> &str {
        "Convert to snake_case format"
    }
}

/// Kebab case transformation implementation
pub struct KebabCaseTransform;

impl Transformation for KebabCaseTransform {
    fn transform(&self, input: &str) -> String {
        kebab_case(input)
    }
    
    fn name(&self) -> &str {
        "kebab"
    }
    
    fn description(&self) -> &str {
        "Convert to kebab-case format"
    }
}

/// Title case transformation implementation
pub struct TitleCaseTransform;

impl Transformation for TitleCaseTransform {
    fn transform(&self, input: &str) -> String {
        title_case(input)
    }
    
    fn name(&self) -> &str {
        "title"
    }
    
    fn description(&self) -> &str {
        "Convert to Title Case format"
    }
}

/// Camel case transformation implementation
pub struct CamelCaseTransform;

impl Transformation for CamelCaseTransform {
    fn transform(&self, input: &str) -> String {
        camel_case(input)
    }
    
    fn name(&self) -> &str {
        "camel"
    }
    
    fn description(&self) -> &str {
        "Convert to camelCase format"
    }
}

/// Pascal case transformation implementation
pub struct PascalCaseTransform;

impl Transformation for PascalCaseTransform {
    fn transform(&self, input: &str) -> String {
        pascal_case(input)
    }
    
    fn name(&self) -> &str {
        "pascal"
    }
    
    fn description(&self) -> &str {
        "Convert to PascalCase format"
    }
}

/// Lowercase transformation implementation
pub struct LowercaseTransform;

impl Transformation for LowercaseTransform {
    fn transform(&self, input: &str) -> String {
        input.to_lowercase()
    }
    
    fn name(&self) -> &str {
        "lower"
    }
    
    fn description(&self) -> &str {
        "Convert to lowercase"
    }
}

/// Uppercase transformation implementation
pub struct UppercaseTransform;

impl Transformation for UppercaseTransform {
    fn transform(&self, input: &str) -> String {
        input.to_uppercase()
    }
    
    fn name(&self) -> &str {
        "upper"
    }
    
    fn description(&self) -> &str {
        "Convert to UPPERCASE"
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
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(kebab_case("HelloWorld"), "hello-world");
        assert_eq!(kebab_case("My_File.txt"), "my-file.txt");
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("hello_world"), "Hello World");
        assert_eq!(title_case("my-file.txt"), "My File Txt");
        assert_eq!(title_case("already Title Case"), "Already Title Case");
    }
    
    #[test]
    fn test_transformation_trait() {
        let clean = CleanTransform;
        assert_eq!(clean.transform("My File (1) !!"), "My File 1");
        assert_eq!(clean.name(), "clean");
        
        let snake = SnakeCaseTransform;
        assert_eq!(snake.transform("HelloWorld"), "hello_world");
        assert_eq!(snake.name(), "snake");
    }
}