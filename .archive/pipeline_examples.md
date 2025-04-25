# SMV Transformation Pipeline Examples

This document explains the pipeline feature with practical examples.

## Pipeline Concept

The pipeline feature allows chaining multiple transformations in sequence. Instead of applying transformations one after another, you can combine them in a single command.

## Basic Syntax

```
smv --pipe "transform1,transform2,transform3" files
```

Or using the short form:

```
smv -p "transform1,transform2,transform3" files
```

Each transformation is separated by commas. Some transformations can take parameters using colons:

```
smv --pipe "transform1,transform2:param1:param2" files
```

## Examples

### Example 1: Clean and Convert to snake_case

```bash
# Without pipeline
smv --clean file.txt      # Step 1: Clean the filename
smv --snake file.txt      # Step 2: Convert to snake_case

# With pipeline
smv --pipeline "clean,snake" file.txt
```

**Input:** `My File (1)!!.txt`
**Output:** `my_file_1.txt`

### Example 2: Multiple Replacements

```bash
smv --pipe "clean,rep:space:_,upper" "My Document.txt"
```

This pipeline will:
1. Clean the filename (remove special characters)
2. Replace all spaces with underscores
3. Convert to uppercase

**Input:** `My Document.txt`
**Output:** `MY_DOCUMENT.txt`

### Example 3: Complex Filename Formatting

```bash
smv -p "snake,rep:_:-,title" "user_profile_data.json"
```

This pipeline will:
1. Convert to snake_case (if not already)
2. Replace underscores with hyphens
3. Convert to Title Case

**Input:** `user_profile_data.json`
**Output:** `User-Profile-Data.json`

### Example 4: Character Set Transformations

```bash
smv -p "chars:digits:X,camel" "file123.txt"
```

This pipeline will:
1. Replace all digits with 'X'
2. Convert to camelCase

**Input:** `file123.txt`
**Output:** `fileXXX.txt`

## Parameterized Transformations

Some transformations accept parameters:

| Transformation | Parameter Format | Example | Shorthand |
|----------------|-----------------|---------|-----------|
| `rep` | `old:new` | `rep:space:_` | `-R space:_` |
| `regex` | `pattern:replacement` | `regex:^file:doc` | `-X ^file:doc` |
| `chars` | `charset:replacement` | `chars:digits:#` | `-C digits:#` |

Available character sets:
- `digits`: All numeric digits (0-9)
- `lowercase`: Lowercase letters (a-z)
- `uppercase`: Uppercase letters (A-Z)
- `vowels`: Vowel characters (aeiou)
- `consonants`: Consonant characters
- `whitespace`: Space characters
- `punctuation`: Punctuation characters

## Saving Pipelines as Custom Transformations

You can save frequently used pipelines as custom transformations:

```bash
# Save a pipeline
smv --save "my-format:clean,snake,rep:space:_"

# Use the saved transformation
smv -t my-format file.txt
```

## Pipeline Visualization

In interactive mode, you can preview the pipeline steps:

```
smv -i
> preview pipe "clean,snake,upper" "My File.txt"

Pipeline preview:
Input: "My File.txt"
Step 1 (clean): "My File.txt"
Step 2 (snake): "my_file.txt"
Step 3 (upper): "MY_FILE.TXT"
Final: "MY_FILE.TXT"
```

This allows you to see the effect of each transformation step.