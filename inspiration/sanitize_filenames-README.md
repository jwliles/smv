# File and Directory Renaming Script

## Overview

This Python script recursively renames files and directories within a given directory to adhere to certain naming
conventions. It aims to standardize file and folder names by converting them to lowercase, replacing special characters,
and ensuring compatibility with Windows naming restrictions.

## Features

- **Naming Conventions**:
    - Converts all names to lowercase, except for `README.md` which is kept in uppercase.
    - Skips renaming of files and directories in CamelCase or PascalCase.
- **Special Character Handling**:
    - Replaces `&` with `and`.
    - Removes quotes, apostrophes, and certain punctuation (`.,!?()`) from names.
    - Replaces colons, hyphens, en dashes, and em dashes with underscores (`_`).
    - Replaces Windows-incompatible characters (`<>:"/\|?*`) with underscores.
    - Collapses multiple underscores into a single underscore and trims leading or trailing underscores.
- **Reserved Windows Names**: Appends an underscore (`_`) to any file or directory name that matches a reserved Windows
  name (e.g., `CON`, `PRN`, `AUX`, `COM1`).
- **Lowercase Extensions**: Converts all file extensions to lowercase.

## Usage

1. **Run the Script**: Use the script with a target directory as a command-line argument, or without arguments to
   process the current directory:
   ```bash
   python3 rename_files_and_directories.py <target_directory>
   ```
    - If `<target_directory>` is omitted, the script uses the current working directory.

2. **Process**: The script will:
    - Recursively traverse through all non-hidden files and directories.
    - Rename files and directories according to the specified conventions.
    - Skip any CamelCase or PascalCase names.

## Requirements

- **Python 3**: The script is written in Python 3.
- **File System Access**: The script needs read and write permissions to rename files and directories within the
  specified target directory.

## Customization

- **Excluding Certain Names**: The script currently preserves CamelCase and PascalCase names. If you want to rename
  these as well, modify the `is_camel_or_pascal_case` function.
- **Character Replacement Rules**: The character replacement logic can be customized in the `rename_item` function,
  where the script replaces characters like `&` and `<>:"/\|?*`.

## Notes

- **Handling Reserved Windows Names**: The script uses a list of reserved names (`CON`, `PRN`, `AUX`, `NUL`, etc.) to
  append an underscore if a name matches any reserved word.
- **Hidden Files and Directories**: The script ignores hidden files and directories (names starting with `.`).

## Example

Before running the script:

```
example &file!.txt
CamelCaseFile.md
CON.txt
some-folder/
```

After running the script:

```
example_andfile.txt
CamelCaseFile.md
CON_.txt
some_folder/
```

## License

This script is released under [The Unlicense](https://unlicense.org/), making it public domain and free to use without
any restrictions.