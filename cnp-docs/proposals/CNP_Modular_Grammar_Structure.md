# CNP Modular Grammar Structure Proposal

**Version**: 1.0 DRAFT
**Date**: 2025-11-15
**Status**: Proposal - Under Review
**Companion Document**: CNP_Modular_Parser_Architecture.md

---

## Executive Summary

This document proposes a fundamental restructuring of CNP grammars from a hierarchical inheritance model (Base → Query/Action) to a **modular composition model** where grammars are assembled from independent, self-contained modules.

**Current Model:**
```
CNP_Base_Grammar (monolithic foundation)
    ├── CNP_Query_Grammar (inherits everything, adds more)
    └── CNP_Action_Grammar (inherits everything, adds more)
```

**Proposed Model:**
```
Grammar Modules (independent, composable):
    ├── Base Module (foundation primitives)
    ├── Filter Module (TYPE:, EXT:, NAME:, SIZE:, etc.)
    ├── Scope Module (tree, sys, disk, net)
    ├── Action Module (mv, cp, rm, rename, etc.)
    ├── Transform Module (CHANGE:INTO:, REGEX:INTO:)
    ├── Logic Module (AND, OR, NOT, WHERE:)
    ├── Route Module (TO:, INTO:, FORMAT:)
    └── Flag Module (-f, -d, -r, -h, -v, etc.)

Tools compose only what they need:
    RPT = Base + Filters + Scopes + Routes + Flags
    XFD = Base + Filters + Logic + Routes + Flags
    SMV = Base + Filters + Actions + Transforms + Logic + Routes + Flags
```

**Why This Change:**

1. **Eliminates Grammar Bloat**: Tools no longer inherit capabilities they don't use
2. **Enables Semantic Parsing**: Modules can be parsed independently, enabling position-independent command structures
3. **Clarifies Ownership**: Each keyword has one clear owner module
4. **Simplifies Testing**: Modules can be validated independently
5. **Supports Evolution**: New modules can be added without affecting existing tools
6. **Preserves Philosophy**: Maintains "caveman sentence" approach with minimal punctuation

---

## Table of Contents

1. [Background and Motivation](#1-background-and-motivation)
2. [Design Principles](#2-design-principles)
3. [Module Specifications](#3-module-specifications)
4. [Grammar Composition Rules](#4-grammar-composition-rules)
5. [Tool-Specific Grammars](#5-tool-specific-grammars)
6. [Keyword Ownership Matrix](#6-keyword-ownership-matrix)
7. [Migration from Base Grammar v1.0](#7-migration-from-base-grammar-v10)
8. [Formal Grammar Notation](#8-formal-grammar-notation)
9. [Compatibility and Interoperability](#9-compatibility-and-interoperability)
10. [Trade-offs and Considerations](#10-trade-offs-and-considerations)
11. [Decision Framework](#11-decision-framework)

---

## 1. Background and Motivation

### 1.1 The Problem Cascade

A seemingly simple UX improvement exposed a cascade of architectural issues:

**Layer 1: UX Issue**
- `rpt files` should be renamed to `rpt tree` for clarity
- Should support both `-f` (files) and `-d` (dirs) flags

**Layer 2: Semantic Consistency**
- `rpt files TYPE:dir` feels contradictory
- If "files" is a scope, why does TYPE: override it?
- Should scope and type filters be independent?

**Layer 3: Execution Order**
- `-f` and `-d` are both flags AND type filters
- Type filters must execute BEFORE filesystem traversal
- Positional parsing makes this order fragile

**Layer 4: Architecture**
- Current Base Grammar mandates positional structure
- Query tools want declarative, position-independent parsing
- Action tools need imperative, order-dependent parsing
- Hierarchical grammar inheritance forces inappropriate constraints

### 1.2 Current Grammar Architecture Issues

**Issue 1: Inappropriate Inheritance**

Base Grammar defines: `<tool> <path> <FILTERS> <ROUTES> <flags>`

This positional structure makes sense for Action tools:
```bash
smv mv source.txt dest.txt CHANGE:foo:bar  # Order matters: action → targets → transform
```

But NOT for Query tools:
```bash
rpt tree . EXT:rs -f          # Position shouldn't matter
rpt tree -f . EXT:rs          # Same meaning, fails due to position
rpt -f tree EXT:rs .          # Same meaning, fails due to position
```

**Issue 2: Grammar Bloat**

Tools inherit capabilities they never use:
- RPT inherits Action Grammar keywords (never uses mv, cp, rm)
- Simple tools like DSC inherit complex filter logic
- Every tool carries delegation keywords even before delegation works

**Issue 3: Unclear Keyword Ownership**

Where does `TYPE:` belong?
- Base Grammar? (currently yes, as a universal filter)
- Query Grammar? (query tools use it)
- Filter Module? (it's a filter)
- Scope Module? (it affects traversal scope)

Current answer: "It's in Base Grammar, so everyone inherits it."
Better answer: "It's in Filter Module, and tools opt-in if they need filtering."

**Issue 4: Evolution Constraints**

Adding new capabilities requires:
1. Adding to Base Grammar (affects ALL tools)
2. Or adding to Query/Action Grammar (affects tool category)
3. No way to add capabilities to just SOME tools without creating sub-grammars

### 1.3 The Modular Vision

Instead of hierarchical inheritance, use **compositional assembly**:

```
┌─────────────────────────────────────────────────────────┐
│                    Grammar Modules                       │
│  (Independent, single-responsibility, composable)        │
└─────────────────────────────────────────────────────────┘
                            │
                            │ Tools compose what they need
                            ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│     RPT      │  │     XFD      │  │     SMV      │
│              │  │              │  │              │
│ Base         │  │ Base         │  │ Base         │
│ Filters      │  │ Filters      │  │ Filters      │
│ Scopes       │  │ Logic        │  │ Actions      │
│ Routes       │  │ Routes       │  │ Transforms   │
│ Flags        │  │ Flags        │  │ Logic        │
│              │  │              │  │ Routes       │
│              │  │              │  │ Flags        │
└──────────────┘  └──────────────┘  └──────────────┘
```

**Benefits:**

1. **Right-sized grammars**: Tools have exactly what they need
2. **Independent evolution**: Modules evolve without cross-contamination
3. **Clear ownership**: Each keyword belongs to exactly one module
4. **Testable isolation**: Module validation is straightforward
5. **Flexible parsing**: Modules can support semantic OR positional parsing as appropriate
6. **Gradual adoption**: Tools can migrate one module at a time

---

## 2. Design Principles

### 2.1 Core Principles

**Principle 1: Single Responsibility**
> Each grammar module has one clear purpose and owns one category of keywords.

Examples:
- Filter Module: Owns all filter keywords (TYPE:, EXT:, NAME:, SIZE:, MORE:, LESS:, etc.)
- Action Module: Owns all action verbs (mv, cp, rm, rename, delete, etc.)
- Logic Module: Owns logical operators (AND, OR, NOT, WHERE:, parentheses)

**Principle 2: Independence**
> Modules should have minimal dependencies on each other.

Examples:
- Filter Module does NOT depend on Logic Module (filters work without logic)
- Logic Module CAN depend on Filter Module (logic operates on filters)
- Route Module is independent (any tool can route output)

**Principle 3: Composition Over Inheritance**
> Tools assemble capabilities by composing modules, not inheriting from hierarchies.

Anti-pattern:
```rust
// Hierarchical inheritance
struct QueryGrammar extends BaseGrammar {
    // Inherits everything whether needed or not
}
```

Pattern:
```rust
// Compositional assembly
struct RptGrammar {
    base: BaseModule,
    filters: FilterModule,
    scopes: ScopeModule,
    routes: RouteModule,
    flags: FlagModule,
    // Logic module intentionally NOT included
}
```

**Principle 4: Preserve Philosophy**
> Maintain CNP's "caveman sentence" philosophy: content words, minimal punctuation, readable commands.

Examples:
```bash
# Good: Reads like simplified English
xfd . EXT:rs TYPE:file MORE:1KB

# Bad: Punctuation hell
find . -name "*.rs" -type f -size +1k

# Good: Semantic clarity
smv rename NAME:test CHANGE:test:prod

# Bad: Positional dependency
smv --rename --pattern="test" --replacement="prod" --filter="test"
```

**Principle 5: Explicit Dependencies**
> If one module needs another, declare it explicitly.

Examples:
```toml
[features]
logic = ["base"]              # Logic depends on Base
transforms = ["filters"]      # Transforms can filter results
query = ["filters", "logic"]  # Query composition requires both
```

**Principle 6: Gradual Migration**
> Modules can be adopted incrementally without breaking existing tools.

Migration path:
1. Phase 1: Extract modules, keep existing parsers working
2. Phase 2: Opt-in one tool to modular parsing (RPT pilot)
3. Phase 3: Migrate query tools (XFD, DSC, INX)
4. Phase 4: Migrate action tools (SMV, MKR, EDT)

### 2.2 Anti-Principles (What We Avoid)

**Anti-Principle 1: Universal Keywords**
> Avoid "this keyword is useful, so put it in Base Grammar for everyone."

Why: Creates bloat and false dependencies.

**Anti-Principle 2: Positional Mandates**
> Avoid "all tools must use this positional structure."

Why: Query tools want semantic parsing, Action tools want positional. One size doesn't fit all.

**Anti-Principle 3: Premature Abstraction**
> Avoid "let's make this generic for all future use cases."

Why: YAGNI. Build for known needs, refactor when new needs emerge.

**Anti-Principle 4: Hidden Dependencies**
> Avoid "this module secretly relies on that module's implementation details."

Why: Breaks independent testing and evolution.

---

## 3. Module Specifications

### 3.1 Base Module

**Purpose**: Foundation primitives that ALL CNP tools need.

**Responsibilities:**
- Tool identification
- Universal help/version flags
- Basic error types
- Tokenization primitives

**Keywords Owned:**
- None (Base provides infrastructure, not keywords)

**Flags Owned:**
- `-h` / `--help`: Universal help
- `-v` / `--version`: Universal version info

**Dependencies:**
- None (Base is the foundation)

**Formal Grammar:**
```ebnf
(* Base Module Grammar *)
tool           = tool_name , { base_flag } ;
tool_name      = "rpt" | "xfd" | "smv" | "dsc" | "inx" | "mkr" | "edt" ;
base_flag      = help_flag | version_flag ;
help_flag      = "-h" | "--help" ;
version_flag   = "-v" | "--version" ;
```

**Parsing Behavior:**
- Always parsed first, before all other modules
- If `-h` or `-v` detected, short-circuit all other parsing
- Provides tokenization for other modules to consume

**Example Usage:**
```bash
rpt -h               # Help processed by Base, exit
rpt --version        # Version processed by Base, exit
xfd -h               # Same behavior across all tools
```

**Module Interface:**
```rust
pub struct BaseModule {
    pub tool_name: String,
}

impl BaseModule {
    pub fn parse(args: &[String]) -> Result<(BaseModule, &[String])> {
        // Check for -h/-v first
        if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
            return Err(Error::ShowHelp);
        }
        if args.contains(&"-v".to_string()) || args.contains(&"--version".to_string()) {
            return Err(Error::ShowVersion);
        }

        // Extract tool name, return remaining args for other modules
        let tool_name = args[0].clone();
        Ok((BaseModule { tool_name }, &args[1..]))
    }
}
```

---

### 3.2 Filter Module

**Purpose**: File and directory filtering by attributes.

**Responsibilities:**
- Type filtering (file, dir, symlink, etc.)
- Extension matching
- Name pattern matching
- Size comparisons
- Time/date filtering
- Depth filtering

**Keywords Owned:**
```
TYPE:       - File type filter (file, dir, symlink, socket, pipe, block, char)
EXT:        - Extension filter (rs, md, txt, etc.)
NAME:       - Name pattern matching (supports glob)
SIZE:       - Exact size filter
MORE:       - Size greater than threshold
LESS:       - Size less than threshold
DEEPER:     - Depth greater than N
SHALLOWER:  - Depth less than N
NEWER:      - Modified after date/time
OLDER:      - Modified before date/time
HIDDEN:     - Include hidden files (true/false)
EXCL:       - Exclusion patterns
```

**Flags Owned:**
```
-f          - Alias for TYPE:file
-d          - Alias for TYPE:dir
```

**Dependencies:**
- `base`: Requires tokenization

**Formal Grammar:**
```ebnf
(* Filter Module Grammar *)
filter        = type_filter | ext_filter | name_filter | size_filter
              | time_filter | depth_filter | hidden_filter | excl_filter ;

type_filter   = "TYPE:" , file_type ;
file_type     = "file" | "dir" | "symlink" | "socket" | "pipe" | "block" | "char" ;

ext_filter    = "EXT:" , extension ;
extension     = identifier , { "," , identifier } ;

name_filter   = "NAME:" , pattern ;
pattern       = glob_pattern | regex_pattern ;

size_filter   = ( "SIZE:" | "MORE:" | "LESS:" ) , size_value ;
size_value    = integer , [ size_unit ] ;
size_unit     = "B" | "KB" | "MB" | "GB" ;

time_filter   = ( "NEWER:" | "OLDER:" ) , time_value ;
time_value    = iso_date | relative_time ;
iso_date      = digit , digit , digit , digit , "-" , digit , digit , "-" , digit , digit ;
relative_time = integer , time_unit ;
time_unit     = "s" | "m" | "h" | "d" | "w" ;

depth_filter  = ( "DEEPER:" | "SHALLOWER:" ) , integer ;

hidden_filter = "HIDDEN:" , boolean ;
boolean       = "true" | "false" | "yes" | "no" ;

excl_filter   = "EXCL:" , pattern ;

(* Flag aliases *)
file_flag     = "-f" ;  (* Equivalent to TYPE:file *)
dir_flag      = "-d" ;  (* Equivalent to TYPE:dir *)
```

**Parsing Behavior:**
- Position-independent (filters can appear anywhere in command)
- Multiple filters combine with AND logic (unless Logic Module overrides)
- Flag aliases (`-f`, `-d`) expand to keyword form during parsing

**Execution Semantics:**
- Type filters execute BEFORE traversal (affects which files are visited)
- Content filters execute AFTER traversal (filters results)
- Depth filters execute DURING traversal (prunes search)

**Example Usage:**
```bash
# Simple filters
xfd . TYPE:file EXT:rs
xfd . -f EXT:rs              # Same as above (-f is alias)

# Size filters
xfd . MORE:1MB LESS:10MB

# Time filters
xfd . NEWER:2024-01-01 OLDER:2024-12-31
xfd . NEWER:1d               # Last 24 hours

# Depth filters
rpt tree DEEPER:2 SHALLOWER:5

# Combined filters (AND logic without Logic Module)
xfd . TYPE:file EXT:rs MORE:1KB NEWER:1w
```

**Module Interface:**
```rust
pub struct FilterModule {
    pub type_filter: Option<FileType>,
    pub ext_filters: Vec<String>,
    pub name_patterns: Vec<Pattern>,
    pub size_constraints: Vec<SizeConstraint>,
    pub time_constraints: Vec<TimeConstraint>,
    pub depth_constraints: Vec<DepthConstraint>,
    pub include_hidden: bool,
    pub exclusions: Vec<Pattern>,
}

#[derive(Debug)]
pub enum FileType {
    File, Dir, Symlink, Socket, Pipe, Block, Char,
}

impl FilterModule {
    pub fn parse(tokens: &[Token]) -> Result<FilterModule> {
        let mut module = FilterModule::default();

        for token in tokens {
            match token {
                Token::Keyword("TYPE", value) => {
                    module.type_filter = Some(parse_file_type(value)?);
                }
                Token::Keyword("EXT", value) => {
                    module.ext_filters.extend(value.split(',').map(String::from));
                }
                Token::Flag("-f") => {
                    module.type_filter = Some(FileType::File);
                }
                Token::Flag("-d") => {
                    module.type_filter = Some(FileType::Dir);
                }
                // ... other filters
                _ => {} // Not a filter token, ignore
            }
        }

        Ok(module)
    }

    pub fn matches(&self, entry: &DirEntry) -> bool {
        // Apply all filter criteria
        if let Some(ref type_filter) = self.type_filter {
            if !self.matches_type(entry, type_filter) {
                return false;
            }
        }
        // ... check other filters
        true
    }
}
```

---

### 3.3 Scope Module

**Purpose**: Define the operational scope/domain for a command.

**Responsibilities:**
- Filesystem traversal scope (tree, files, dirs)
- System information scope (sys, env, net, disk, proc)
- Target path specification
- Multi-target selection

**Keywords Owned:**
```
tree        - Filesystem tree traversal
sys         - System information
env         - Environment variables
net         - Network information
disk        - Disk information
proc        - Process information
IN:         - Multi-target specification (IN:dir1,dir2,dir3)
```

**Target Specification:**
- Positional path argument (e.g., `.`, `/home/user`, `src/`)
- `IN:` for multiple targets

**Dependencies:**
- `base`: Requires tokenization

**Formal Grammar:**
```ebnf
(* Scope Module Grammar *)
scope         = fs_scope | sys_scope | target_spec ;

fs_scope      = "tree" | "files" | "dirs" ;

sys_scope     = "sys" | "env" | "net" | "disk" | "proc" ;

target_spec   = path | multi_target ;
path          = ( "." | ".." | "/" | relative_path | absolute_path ) ;
multi_target  = "IN:" , path , { "," , path } ;

relative_path = identifier , { "/" , identifier } ;
absolute_path = "/" , { identifier , "/" } , [ identifier ] ;
```

**Parsing Behavior:**
- Scopes are typically positional (first non-flag argument after tool name)
- If no scope specified, default behavior depends on tool:
  - RPT: defaults to `tree`
  - XFD: defaults to current directory query
  - SMV: requires explicit scope or target

**Smart Defaults:**
```bash
rpt                      # Defaults to: rpt tree .
rpt sys                  # Explicit sys scope
xfd EXT:rs               # Defaults to: xfd . EXT:rs
smv mv file.txt dest/    # Target specified, no scope needed
```

**Example Usage:**
```bash
# Filesystem scopes
rpt tree                 # Tree view of current directory
rpt tree /home/user      # Tree view of specific path
xfd . TYPE:file          # Explicit current directory

# System scopes
rpt sys                  # System information
rpt env                  # Environment variables
rpt net                  # Network interfaces
rpt disk                 # Disk usage

# Multi-target
xfd IN:src,tests,docs EXT:rs
```

**Module Interface:**
```rust
pub struct ScopeModule {
    pub scope_type: ScopeType,
    pub targets: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum ScopeType {
    // Filesystem scopes
    Tree,
    Files,
    Dirs,

    // System scopes
    Sys,
    Env,
    Net,
    Disk,
    Proc,
}

impl ScopeModule {
    pub fn parse(tokens: &[Token]) -> Result<ScopeModule> {
        let mut scope_type = None;
        let mut targets = Vec::new();

        for token in tokens {
            match token {
                Token::Identifier("tree") => scope_type = Some(ScopeType::Tree),
                Token::Identifier("sys") => scope_type = Some(ScopeType::Sys),
                Token::Keyword("IN", value) => {
                    targets = value.split(',').map(PathBuf::from).collect();
                }
                Token::Path(p) => {
                    targets.push(p.clone());
                }
                _ => {}
            }
        }

        // Apply smart defaults
        if scope_type.is_none() {
            scope_type = Some(ScopeType::Tree); // Default for RPT
        }
        if targets.is_empty() {
            targets.push(PathBuf::from(".")); // Default to current directory
        }

        Ok(ScopeModule {
            scope_type: scope_type.unwrap(),
            targets,
        })
    }
}
```

---

### 3.4 Action Module

**Purpose**: Imperative file operations (moving, copying, deleting, renaming, etc.).

**Responsibilities:**
- File/directory movement
- Copy operations
- Deletion
- Rename operations
- Archive operations
- Permission changes

**Keywords Owned:**
```
mv          - Move files/directories
cp          - Copy files/directories
rm          - Remove files/directories
rename      - Rename files/directories (bulk)
delete      - Delete with confirmation
archive     - Archive to zip/tar
extract     - Extract from archive
chmod       - Change permissions
chown       - Change ownership
```

**Dependencies:**
- `base`: Requires tokenization
- `filters` (optional): Actions can filter targets

**Formal Grammar:**
```ebnf
(* Action Module Grammar *)
action        = move_action | copy_action | remove_action | rename_action
              | archive_action | permission_action ;

move_action   = "mv" , source , destination ;
copy_action   = "cp" , source , destination ;
remove_action = "rm" , target , { target } ;
rename_action = "rename" , target_spec ;

source        = path | filter_pattern ;
destination   = path ;
target        = path | filter_pattern ;
target_spec   = path | filter_pattern ;

archive_action = ( "archive" | "extract" ) , path , [ archive_format ] ;
archive_format = "zip" | "tar" | "tar.gz" | "tar.bz2" ;

permission_action = ( "chmod" | "chown" ) , target , permission_spec ;
permission_spec   = octal_mode | symbolic_mode | owner_spec ;
```

**Parsing Behavior:**
- Actions are POSITIONAL (order matters for imperative commands)
- First action keyword defines the operation
- Subsequent arguments are targets/destinations in order
- Filters can refine targets but action structure is sequential

**Execution Semantics:**
- Actions execute in order specified
- Side effects are immediate (unless `--dry-run` flag)
- Preview mode available for safety

**Example Usage:**
```bash
# Simple actions
smv mv file.txt dest/
smv cp src/ dest/
smv rm temp.txt

# Actions with filters
smv mv . EXT:log /var/log/
smv rm . TYPE:file OLDER:30d

# Bulk rename
smv rename . NAME:test CHANGE:test:prod

# Archive
smv archive . INTO:backup.zip
smv extract backup.tar.gz
```

**Module Interface:**
```rust
pub struct ActionModule {
    pub action: ActionType,
    pub sources: Vec<PathBuf>,
    pub destination: Option<PathBuf>,
    pub options: ActionOptions,
}

#[derive(Debug)]
pub enum ActionType {
    Move,
    Copy,
    Remove,
    Rename,
    Archive,
    Extract,
    Chmod,
    Chown,
}

impl ActionModule {
    pub fn parse(tokens: &[Token]) -> Result<ActionModule> {
        let mut iter = tokens.iter().peekable();

        // First token should be action keyword
        let action = match iter.next() {
            Some(Token::Identifier("mv")) => ActionType::Move,
            Some(Token::Identifier("cp")) => ActionType::Copy,
            Some(Token::Identifier("rm")) => ActionType::Remove,
            // ... other actions
            _ => return Err(Error::NoActionSpecified),
        };

        // Parse remaining tokens positionally
        let sources = Vec::new();
        let destination = None;
        // ... positional parsing logic

        Ok(ActionModule { action, sources, destination, options: Default::default() })
    }

    pub fn execute(&self, filters: Option<&FilterModule>) -> Result<ActionResult> {
        // Apply filters to refine targets
        let targets = if let Some(filters) = filters {
            self.sources.iter()
                .filter(|s| filters.matches(s))
                .collect()
        } else {
            self.sources.clone()
        };

        // Execute action on targets
        match self.action {
            ActionType::Move => self.execute_move(targets),
            ActionType::Copy => self.execute_copy(targets),
            // ... other actions
        }
    }
}
```

---

### 3.5 Transform Module

**Purpose**: Content transformation during actions (find-replace, regex, case conversion).

**Responsibilities:**
- String replacement in filenames
- Regex-based transformations
- Case conversions
- Prefix/suffix additions
- Sequence numbering

**Keywords Owned:**
```
CHANGE:INTO:    - Simple find-replace (CHANGE:old:INTO:new)
REGEX:INTO:     - Regex-based replacement (REGEX:pattern:INTO:replacement)
CASE:           - Case conversion (CASE:upper, CASE:lower, CASE:title)
PREFIX:         - Add prefix to names
SUFFIX:         - Add suffix to names
NUMBER:         - Add sequence numbers (NUMBER:01, NUMBER:001)
```

**Dependencies:**
- `base`: Requires tokenization
- `filters`: Transforms typically apply to filtered results

**Formal Grammar:**
```ebnf
(* Transform Module Grammar *)
transform     = change_transform | regex_transform | case_transform
              | affix_transform | number_transform ;

change_transform = "CHANGE:" , find_pattern , ":INTO:" , replace_pattern ;
find_pattern     = text ;
replace_pattern  = text ;

regex_transform  = "REGEX:" , regex_pattern , ":INTO:" , regex_replacement ;
regex_pattern    = text ;  (* Valid regex syntax *)
regex_replacement = text ;  (* With capture group references *)

case_transform   = "CASE:" , case_type ;
case_type        = "upper" | "lower" | "title" | "snake" | "camel" | "pascal" ;

affix_transform  = prefix_transform | suffix_transform ;
prefix_transform = "PREFIX:" , text ;
suffix_transform = "SUFFIX:" , text ;

number_transform = "NUMBER:" , number_format ;
number_format    = digit , { digit } ;  (* e.g., "01" for 2-digit padding *)
```

**Parsing Behavior:**
- Position-independent within command
- Multiple transforms can be chained
- Execution order: CHANGE → REGEX → CASE → AFFIX → NUMBER

**Execution Semantics:**
- Transforms create new names without side effects (until action executes)
- Preview mode shows transformations before applying
- Collisions detected and reported

**Example Usage:**
```bash
# Simple replacement
smv rename . NAME:test CHANGE:test:INTO:prod

# Regex transformation
smv rename . EXT:txt REGEX:^log_(.+):INTO:archive_$1

# Case conversion
smv rename . EXT:md CASE:title

# Add prefix
smv rename . TYPE:file PREFIX:backup_

# Sequence numbering
smv rename . EXT:jpg NUMBER:001

# Chained transforms
smv rename . EXT:txt CHANGE:log:INTO:archive CASE:lower PREFIX:old_ NUMBER:01
```

**Module Interface:**
```rust
pub struct TransformModule {
    pub transforms: Vec<Transform>,
}

#[derive(Debug)]
pub enum Transform {
    Change { find: String, replace: String },
    Regex { pattern: regex::Regex, replacement: String },
    Case(CaseType),
    Prefix(String),
    Suffix(String),
    Number { format: String, start: usize },
}

#[derive(Debug)]
pub enum CaseType {
    Upper, Lower, Title, Snake, Camel, Pascal,
}

impl TransformModule {
    pub fn parse(tokens: &[Token]) -> Result<TransformModule> {
        let mut transforms = Vec::new();

        for token in tokens {
            match token {
                Token::Keyword("CHANGE", value) => {
                    if let Some((find, replace)) = value.split_once(":INTO:") {
                        transforms.push(Transform::Change {
                            find: find.to_string(),
                            replace: replace.to_string(),
                        });
                    }
                }
                Token::Keyword("REGEX", value) => {
                    if let Some((pattern, replacement)) = value.split_once(":INTO:") {
                        let regex = regex::Regex::new(pattern)?;
                        transforms.push(Transform::Regex {
                            pattern: regex,
                            replacement: replacement.to_string(),
                        });
                    }
                }
                Token::Keyword("CASE", value) => {
                    transforms.push(Transform::Case(parse_case_type(value)?));
                }
                // ... other transforms
                _ => {}
            }
        }

        Ok(TransformModule { transforms })
    }

    pub fn apply(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Apply transforms in order
        for transform in &self.transforms {
            result = match transform {
                Transform::Change { find, replace } => result.replace(find, replace),
                Transform::Regex { pattern, replacement } => {
                    pattern.replace(&result, replacement).to_string()
                }
                Transform::Case(case_type) => apply_case(&result, case_type),
                Transform::Prefix(prefix) => format!("{}{}", prefix, result),
                Transform::Suffix(suffix) => format!("{}{}", result, suffix),
                Transform::Number { format, start } => {
                    format!("{}_{:0width$}", result, start, width = format.len())
                }
            };
        }

        result
    }
}
```

---

### 3.6 Logic Module

**Purpose**: Boolean logic for complex queries (AND, OR, NOT, grouping).

**Responsibilities:**
- Combine multiple filters with AND/OR logic
- Negate filters with NOT
- Group expressions with parentheses
- Short-circuit evaluation

**Keywords Owned:**
```
AND         - Logical AND (both conditions must be true)
OR          - Logical OR (either condition must be true)
NOT         - Logical NOT (invert condition)
WHERE:      - Complex filter expressions (WHERE:(EXT:rs OR EXT:md))
( )         - Grouping parentheses
```

**Dependencies:**
- `base`: Requires tokenization
- `filters`: Logic operates on filter expressions

**Formal Grammar:**
```ebnf
(* Logic Module Grammar *)
logic_expr    = or_expr ;

or_expr       = and_expr , { "OR" , and_expr } ;

and_expr      = not_expr , { "AND" , not_expr } ;

not_expr      = [ "NOT" ] , primary_expr ;

primary_expr  = filter | grouped_expr | where_expr ;

grouped_expr  = "(" , logic_expr , ")" ;

where_expr    = "WHERE:" , "(" , logic_expr , ")" ;

filter        = (* Any filter from Filter Module *) ;
```

**Parsing Behavior:**
- Position-independent
- Standard operator precedence: NOT > AND > OR
- Parentheses override precedence
- `WHERE:` is syntactic sugar for complex expressions

**Execution Semantics:**
- Lazy evaluation (short-circuit when possible)
- Filters evaluated in dependency order (type filters before content filters)

**Example Usage:**
```bash
# Simple AND (implicit)
xfd . EXT:rs TYPE:file

# Explicit OR
xfd . WHERE:(EXT:rs OR EXT:md)

# Complex logic
xfd . WHERE:(EXT:rs OR EXT:md) AND TYPE:file AND MORE:1KB

# Negation
xfd . TYPE:file NOT NAME:test

# Grouped expressions
xfd . WHERE:((EXT:rs OR EXT:md) AND NOT NAME:test)

# Multi-condition
xfd . WHERE:(TYPE:file AND (EXT:rs OR EXT:toml) AND MORE:1KB AND NEWER:1w)
```

**Module Interface:**
```rust
pub struct LogicModule {
    pub root: LogicExpr,
}

#[derive(Debug)]
pub enum LogicExpr {
    And(Vec<LogicExpr>),
    Or(Vec<LogicExpr>),
    Not(Box<LogicExpr>),
    Filter(FilterExpr),
}

impl LogicModule {
    pub fn parse(tokens: &[Token], filters: &FilterModule) -> Result<LogicModule> {
        // Parse logic expression tree
        let root = parse_logic_expr(tokens, filters)?;
        Ok(LogicModule { root })
    }

    pub fn evaluate(&self, entry: &DirEntry) -> bool {
        evaluate_expr(&self.root, entry)
    }
}

fn evaluate_expr(expr: &LogicExpr, entry: &DirEntry) -> bool {
    match expr {
        LogicExpr::And(exprs) => {
            exprs.iter().all(|e| evaluate_expr(e, entry))
        }
        LogicExpr::Or(exprs) => {
            exprs.iter().any(|e| evaluate_expr(e, entry))
        }
        LogicExpr::Not(inner) => {
            !evaluate_expr(inner, entry)
        }
        LogicExpr::Filter(filter) => {
            filter.matches(entry)
        }
    }
}
```

---

### 3.7 Route Module

**Purpose**: Output routing and delegation (TO:, INTO:, FORMAT:).

**Responsibilities:**
- Delegate to another CNP tool
- Write output to file
- Transform output format

**Keywords Owned:**
```
TO:         - Delegate to another tool (TO:smv, TO:rpt)
INTO:       - Write to file (INTO:results.txt)
FORMAT:     - Output format (FORMAT:json, FORMAT:csv, FORMAT:cnp)
```

**Dependencies:**
- `base`: Requires tokenization

**Formal Grammar:**
```ebnf
(* Route Module Grammar *)
route         = to_route | into_route | format_route ;

to_route      = "TO:" , tool_name ;
tool_name     = "rpt" | "xfd" | "smv" | "dsc" | "inx" | "mkr" | "edt" ;

into_route    = "INTO:" , file_path ;
file_path     = path ;

format_route  = "FORMAT:" , format_type ;
format_type   = "json" | "csv" | "table" | "tree" | "cnp" | "toml" | "yaml" ;
```

**Parsing Behavior:**
- Position-independent
- Multiple routes processed in order: FORMAT → TO → INTO
- Routes are terminal operations (last step in execution)

**Execution Semantics:**
1. Tool executes primary operation
2. Apply FORMAT transformation
3. Execute TO delegation (spawn new process with output)
4. Write to INTO destination

**Example Usage:**
```bash
# Delegation
xfd . EXT:log TO:smv

# File output
rpt tree INTO:report.txt

# Format transformation
rpt sys FORMAT:json

# Combined routing
xfd . EXT:rs FORMAT:json INTO:results.json

# Delegation chain
xfd . EXT:log FORMAT:cnp TO:smv
```

**Module Interface:**
```rust
pub struct RouteModule {
    pub delegation: Option<String>,
    pub output_file: Option<PathBuf>,
    pub format: Option<OutputFormat>,
}

#[derive(Debug)]
pub enum OutputFormat {
    Json, Csv, Table, Tree, Cnp, Toml, Yaml,
}

impl RouteModule {
    pub fn parse(tokens: &[Token]) -> Result<RouteModule> {
        let mut delegation = None;
        let mut output_file = None;
        let mut format = None;

        for token in tokens {
            match token {
                Token::Keyword("TO", value) => {
                    delegation = Some(value.to_string());
                }
                Token::Keyword("INTO", value) => {
                    output_file = Some(PathBuf::from(value));
                }
                Token::Keyword("FORMAT", value) => {
                    format = Some(parse_format(value)?);
                }
                _ => {}
            }
        }

        Ok(RouteModule { delegation, output_file, format })
    }

    pub fn apply(&self, output: String) -> Result<()> {
        let mut result = output;

        // Step 1: Apply format transformation
        if let Some(ref fmt) = self.format {
            result = self.format_output(&result, fmt)?;
        }

        // Step 2: Delegate to another tool
        if let Some(ref tool) = self.delegation {
            result = self.delegate(&result, tool)?;
        }

        // Step 3: Write to file
        if let Some(ref path) = self.output_file {
            std::fs::write(path, result)?;
        } else {
            println!("{}", result);
        }

        Ok(())
    }
}
```

---

### 3.8 Flag Module

**Purpose**: Tool-specific and grammar-specific flags.

**Responsibilities:**
- Recursive traversal flags
- Preview/dry-run flags
- Verbosity flags
- Tool-specific behaviors

**Flags Owned:**
```
-r          - Recursive traversal
-p          - Preview mode (dry-run)
-y          - Yes to all prompts
-n          - No to all prompts
-q          - Quiet mode
-V          - Verbose mode
-i          - Interactive mode
--dry-run   - Alias for -p
--force     - Force operation without confirmation
```

**Dependencies:**
- `base`: Requires tokenization

**Formal Grammar:**
```ebnf
(* Flag Module Grammar *)
flag          = short_flag | long_flag ;

short_flag    = "-" , flag_char ;
flag_char     = "r" | "p" | "y" | "n" | "q" | "V" | "i" ;

long_flag     = "--" , flag_name ;
flag_name     = "dry-run" | "force" | "recursive" | "preview" | "quiet" | "verbose" | "interactive" ;
```

**Parsing Behavior:**
- Position-independent
- Flags are boolean switches (presence = true)
- Long flags are aliases for short flags where applicable

**Execution Semantics:**
- Flags modify tool behavior globally
- `-p` / `--dry-run`: Show what would happen, don't execute
- `-r`: Enable recursive traversal
- `-V`: Increase output verbosity

**Example Usage:**
```bash
# Recursive traversal
rpt tree -r

# Preview mode
smv rm . EXT:log -p

# Force without confirmation
smv rm . TYPE:file --force

# Verbose output
rpt sys -V

# Combined flags
xfd . EXT:rs -r -V
```

**Module Interface:**
```rust
pub struct FlagModule {
    pub recursive: bool,
    pub preview: bool,
    pub yes_to_all: bool,
    pub no_to_all: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub interactive: bool,
    pub force: bool,
}

impl FlagModule {
    pub fn parse(tokens: &[Token]) -> Result<FlagModule> {
        let mut flags = FlagModule::default();

        for token in tokens {
            match token {
                Token::Flag("-r") | Token::Flag("--recursive") => {
                    flags.recursive = true;
                }
                Token::Flag("-p") | Token::Flag("--dry-run") | Token::Flag("--preview") => {
                    flags.preview = true;
                }
                Token::Flag("-y") => flags.yes_to_all = true,
                Token::Flag("-n") => flags.no_to_all = true,
                Token::Flag("-q") | Token::Flag("--quiet") => flags.quiet = true,
                Token::Flag("-V") | Token::Flag("--verbose") => flags.verbose = true,
                Token::Flag("-i") | Token::Flag("--interactive") => flags.interactive = true,
                Token::Flag("--force") => flags.force = true,
                _ => {}
            }
        }

        Ok(flags)
    }
}

impl Default for FlagModule {
    fn default() -> Self {
        FlagModule {
            recursive: false,
            preview: false,
            yes_to_all: false,
            no_to_all: false,
            quiet: false,
            verbose: false,
            interactive: false,
            force: false,
        }
    }
}
```

---

## 4. Grammar Composition Rules

### 4.1 Composition Syntax

Tools declare which modules they use via Cargo features:

```toml
# rpt/Cargo.toml
[dependencies]
cnp-parse = { path = "../cnp-parse", features = ["filters", "scopes", "routes", "flags"] }

# xfd/Cargo.toml
[dependencies]
cnp-parse = { path = "../cnp-parse", features = ["filters", "logic", "routes", "flags"] }

# smv/Cargo.toml
[dependencies]
cnp-parse = { path = "../cnp-parse", features = ["filters", "actions", "transforms", "logic", "routes", "flags"] }
```

### 4.2 Dependency Rules

**Rule 1: Base is Universal**
> All tools MUST include Base Module (implicit dependency).

**Rule 2: Declare Direct Dependencies**
> If a tool uses a module's keywords, it must declare that module.

Example:
```toml
# If tool uses EXT:, NAME:, TYPE:
features = ["filters"]

# If tool uses AND, OR, WHERE:
features = ["logic"]
```

**Rule 3: Module Dependencies Auto-Include**
> If Module A depends on Module B, including A automatically includes B.

Example:
```toml
# Including "logic" automatically includes "filters"
# because Logic operates on Filters
features = ["logic"]  # Implicitly includes "filters"
```

**Rule 4: No Circular Dependencies**
> Modules cannot depend on each other cyclically.

Valid:
```
Base ← Filters ← Logic
```

Invalid:
```
Logic ← Filters ← Logic  # CIRCULAR, NOT ALLOWED
```

**Rule 5: Optional Dependencies**
> Modules can optionally enhance other modules without requiring them.

Example:
```rust
// Transforms CAN use filters to refine targets
// But filters are NOT required for transforms to work

impl TransformModule {
    pub fn apply(&self, targets: Vec<PathBuf>, filters: Option<&FilterModule>) -> Vec<PathBuf> {
        if let Some(f) = filters {
            targets.into_iter().filter(|t| f.matches(t)).collect()
        } else {
            targets
        }
    }
}
```

### 4.3 Composition Patterns

**Pattern 1: Query Tool**
```toml
# Minimal query: search + filter
features = ["filters", "routes"]

# Advanced query: search + filter + logic
features = ["filters", "logic", "routes"]

# Example: xfd (Extended Find)
features = ["filters", "logic", "routes", "flags"]
```

**Pattern 2: Action Tool**
```toml
# Minimal action: operate
features = ["actions", "routes"]

# Action with filtering: operate on filtered targets
features = ["actions", "filters", "routes"]

# Full action: operate + filter + transform + logic
features = ["actions", "filters", "transforms", "logic", "routes", "flags"]

# Example: smv (Smart Move)
features = ["actions", "filters", "transforms", "logic", "routes", "flags"]
```

**Pattern 3: Reporting Tool**
```toml
# Report with scope selection
features = ["scopes", "routes"]

# Report with filters
features = ["scopes", "filters", "routes", "flags"]

# Example: rpt (Report)
features = ["scopes", "filters", "routes", "flags"]
```

**Pattern 4: Hybrid Tool**
```toml
# Query + Action hybrid
features = ["filters", "actions", "logic", "routes", "flags"]

# Example: Hypothetical classifier
features = ["filters", "logic", "actions", "transforms", "routes", "flags"]
```

### 4.4 Validation Rules

**Validation 1: Keyword Conflicts**
> No two modules can own the same keyword.

Example:
```
TYPE: is owned by Filter Module
└─ No other module can define TYPE:
```

**Validation 2: Flag Conflicts**
> Flags must be unique within a tool's composed grammar.

Example:
```
-f (Filter: TYPE:file)
-f (Flag: force)         # CONFLICT - NOT ALLOWED IN SAME TOOL
```

**Validation 3: Completeness**
> All keywords used in a command must have a corresponding module included.

Example:
```bash
# Command uses EXT:, WHERE:, INTO:
xfd . EXT:rs WHERE:(NAME:test) INTO:results.txt

# Required modules:
features = ["filters", "logic", "routes"]
```

**Validation 4: Execution Order**
> Modules must declare execution phase if order matters.

Example:
```rust
// Execution phases
Phase::TypeFilter   // Before traversal
Phase::Traversal    // During filesystem walk
Phase::ContentFilter // After traversal
Phase::Logic        // After all filters
Phase::Transform    // Before action
Phase::Action       // Execute operation
Phase::Route        // Terminal routing
```

---

## 5. Tool-Specific Grammars

### 5.1 RPT Grammar (Report)

**Purpose**: Generate reports about filesystem, system, disks, network.

**Modules Composed:**
```toml
features = ["filters", "scopes", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
rpt_command   = "rpt" , [ scope ] , [ target ] , { filter } , { route } , { flag } ;

scope         = "tree" | "sys" | "env" | "net" | "disk" | "proc" ;
target        = path ;
filter        = (* From Filter Module *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -r, -V, -q *) ;
```

**Example Commands:**
```bash
# Default: tree view of current directory
rpt

# Explicit scope and target
rpt tree /home/user

# With filters
rpt tree . TYPE:file EXT:rs

# System report with JSON output
rpt sys FORMAT:json

# Filtered tree with routing
rpt tree . MORE:1MB FORMAT:cnp INTO:large_files.cnp

# Recursive with flags
rpt tree -r -V
```

**Parsing Approach:**
- Semantic parsing (position-independent for filters/routes/flags)
- Scope is positionally first (if present)
- Target is positional second (defaults to `.`)

---

### 5.2 XFD Grammar (Extended Find)

**Purpose**: Advanced file searching with complex logic.

**Modules Composed:**
```toml
features = ["filters", "logic", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
xfd_command   = "xfd" , [ target ] , { filter } , [ logic_expr ] , { route } , { flag } ;

target        = path | "IN:" , path , { "," , path } ;
filter        = (* From Filter Module *) ;
logic_expr    = (* From Logic Module *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -r, -V, -q *) ;
```

**Example Commands:**
```bash
# Simple search
xfd . EXT:rs

# Complex logic
xfd . WHERE:((EXT:rs OR EXT:toml) AND MORE:1KB)

# Multiple targets
xfd IN:src,tests,docs TYPE:file EXT:rs

# Negation
xfd . TYPE:file NOT NAME:test

# With routing
xfd . EXT:log FORMAT:json TO:smv
```

**Parsing Approach:**
- Fully semantic (position-independent)
- Target can be first or after filters
- Logic expressions parsed into AST

---

### 5.3 SMV Grammar (Smart Move)

**Purpose**: Intelligent file operations with transformations.

**Modules Composed:**
```toml
features = ["actions", "filters", "transforms", "logic", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
smv_command   = "smv" , action , [ scope ] , [ target ] , { filter } , { transform } ,
                [ logic_expr ] , { route } , { flag } ;

action        = (* From Action Module *) ;
scope         = (* From Scope Module *) ;
target        = path | destination ;
filter        = (* From Filter Module *) ;
transform     = (* From Transform Module *) ;
logic_expr    = (* From Logic Module *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -p, -r, -y, -n, --force *) ;
```

**Example Commands:**
```bash
# Simple move
smv mv file.txt dest/

# Move with filters
smv mv . EXT:log /var/log/

# Bulk rename with transform
smv rename . NAME:test CHANGE:test:INTO:prod

# Complex action with preview
smv rm . WHERE:(TYPE:file AND OLDER:30d) -p

# Transform with regex
smv rename . EXT:txt REGEX:^log_(.+):INTO:archive_$1

# Delegation
smv mv . EXT:log TO:rpt
```

**Parsing Approach:**
- Hybrid: Action is positional (first), rest is semantic
- Action keyword defines operation type
- Filters/transforms/logic are position-independent
- Targets may be positional depending on action

---

### 5.4 DSC Grammar (Discover)

**Purpose**: Lightweight file discovery (simpler than XFD).

**Modules Composed:**
```toml
features = ["filters", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
dsc_command   = "dsc" , [ target ] , { filter } , { route } , { flag } ;

target        = path ;
filter        = (* From Filter Module: subset only *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -r only *) ;
```

**Example Commands:**
```bash
# Simple discovery
dsc . EXT:rs

# With filters
dsc . TYPE:file MORE:1KB

# Recursive
dsc . EXT:md -r

# Output routing
dsc . TYPE:file INTO:files.txt
```

**Parsing Approach:**
- Semantic parsing
- Subset of filters (no complex logic)
- Minimal flags

---

### 5.5 INX Grammar (Index)

**Purpose**: Build and query file indexes.

**Modules Composed:**
```toml
features = ["filters", "logic", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
inx_command   = "inx" , [ subcommand ] , [ target ] , { filter } , [ logic_expr ] ,
                { route } , { flag } ;

subcommand    = "build" | "query" | "update" | "clear" ;
target        = path ;
filter        = (* From Filter Module *) ;
logic_expr    = (* From Logic Module *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module *) ;
```

**Example Commands:**
```bash
# Build index
inx build .

# Query index
inx query EXT:rs WHERE:(MORE:1KB)

# Update index
inx update .

# Clear index
inx clear
```

---

### 5.6 MKR Grammar (Maker)

**Purpose**: Create files/directories with templates.

**Modules Composed:**
```toml
features = ["actions", "transforms", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
mkr_command   = "mkr" , action , target , { transform } , { route } , { flag } ;

action        = "file" | "dir" | "template" ;
target        = path ;
transform     = (* From Transform Module: for name generation *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -p, --force *) ;
```

**Example Commands:**
```bash
# Create file
mkr file new.txt

# Create directory
mkr dir src/components

# Create from template
mkr template rust-lib my-project

# Bulk create with numbering
mkr file test.txt NUMBER:001
```

---

### 5.7 EDT Grammar (Editor)

**Purpose**: Batch text editing operations.

**Modules Composed:**
```toml
features = ["filters", "transforms", "routes", "flags"]
```

**Complete Grammar:**
```ebnf
edt_command   = "edt" , [ target ] , { filter } , { transform } , { route } , { flag } ;

target        = path ;
filter        = (* From Filter Module *) ;
transform     = (* From Transform Module: for content transforms *) ;
route         = (* From Route Module *) ;
flag          = (* From Flag Module: -p, -r, --force *) ;
```

**Example Commands:**
```bash
# Replace in files
edt . EXT:txt CHANGE:foo:INTO:bar

# Regex replacement
edt . EXT:rs REGEX:println!\((.+)\):INTO:log::info!($1)

# Preview changes
edt . EXT:md CHANGE:TODO:INTO:DONE -p
```

---

## 6. Keyword Ownership Matrix

This matrix defines which module owns which keywords/flags:

| Keyword/Flag | Module | Category | Parsing |
|--------------|--------|----------|---------|
| `-h`, `--help` | Base | Meta | Universal |
| `-v`, `--version` | Base | Meta | Universal |
| `TYPE:` | Filter | Filter | Semantic |
| `EXT:` | Filter | Filter | Semantic |
| `NAME:` | Filter | Filter | Semantic |
| `SIZE:` | Filter | Filter | Semantic |
| `MORE:` | Filter | Filter | Semantic |
| `LESS:` | Filter | Filter | Semantic |
| `NEWER:` | Filter | Filter | Semantic |
| `OLDER:` | Filter | Filter | Semantic |
| `DEEPER:` | Filter | Filter | Semantic |
| `SHALLOWER:` | Filter | Filter | Semantic |
| `HIDDEN:` | Filter | Filter | Semantic |
| `EXCL:` | Filter | Filter | Semantic |
| `-f` | Filter | Flag Alias | Semantic |
| `-d` | Filter | Flag Alias | Semantic |
| `tree` | Scope | Scope | Positional |
| `sys` | Scope | Scope | Positional |
| `env` | Scope | Scope | Positional |
| `net` | Scope | Scope | Positional |
| `disk` | Scope | Scope | Positional |
| `proc` | Scope | Scope | Positional |
| `IN:` | Scope | Multi-target | Semantic |
| `mv` | Action | Action | Positional |
| `cp` | Action | Action | Positional |
| `rm` | Action | Action | Positional |
| `rename` | Action | Action | Positional |
| `delete` | Action | Action | Positional |
| `archive` | Action | Action | Positional |
| `extract` | Action | Action | Positional |
| `chmod` | Action | Action | Positional |
| `chown` | Action | Action | Positional |
| `CHANGE:INTO:` | Transform | Transform | Semantic |
| `REGEX:INTO:` | Transform | Transform | Semantic |
| `CASE:` | Transform | Transform | Semantic |
| `PREFIX:` | Transform | Transform | Semantic |
| `SUFFIX:` | Transform | Transform | Semantic |
| `NUMBER:` | Transform | Transform | Semantic |
| `AND` | Logic | Logic | Semantic |
| `OR` | Logic | Logic | Semantic |
| `NOT` | Logic | Logic | Semantic |
| `WHERE:` | Logic | Logic | Semantic |
| `(`, `)` | Logic | Grouping | Semantic |
| `TO:` | Route | Delegation | Semantic |
| `INTO:` | Route | Output | Semantic |
| `FORMAT:` | Route | Transform | Semantic |
| `-r`, `--recursive` | Flag | Behavior | Semantic |
| `-p`, `--preview` | Flag | Behavior | Semantic |
| `-y` | Flag | Behavior | Semantic |
| `-n` | Flag | Behavior | Semantic |
| `-q`, `--quiet` | Flag | Output | Semantic |
| `-V`, `--verbose` | Flag | Output | Semantic |
| `-i`, `--interactive` | Flag | Behavior | Semantic |
| `--force` | Flag | Behavior | Semantic |
| `--dry-run` | Flag | Behavior | Semantic |

**Notes:**

1. **Positional vs Semantic**:
   - Positional: Must appear in specific position
   - Semantic: Position-independent

2. **Flag Aliases**:
   - `-f` expands to `TYPE:file`
   - `-d` expands to `TYPE:dir`

3. **Conflicts**:
   - No keyword appears in multiple modules
   - Tools combining modules must check for flag conflicts

4. **Reserved for Future Use**:
   - `-H`, `-V` (uppercase variants reserved by Base)
   - `GROUP:`, `SORT:`, `LIMIT:` (aggregation module, not yet implemented)

---

## 7. Migration from Base Grammar v1.0

### 7.1 Current State (Base Grammar v1.0)

**Structure:**
```
CNP_Base_Grammar v1.0
    ├── Universal base flags (-h, -v)
    ├── Universal route keywords (TO:, INTO:, FORMAT:)
    ├── Universal filter keywords (TYPE:, EXT:, NAME:, etc.)
    └── Positional structure: <tool> <path> <FILTERS> <ROUTES> <flags>

CNP_Query_Grammar (extends Base)
    └── Adds semantic parsing, smart defaults, phase-based execution

CNP_Action_Grammar (extends Base)
    └── Adds action verbs, transform keywords, imperative structure
```

**Issues:**
1. Positional mandate conflicts with semantic parsing goals
2. "Universal" keywords create bloat in tools that don't need them
3. Hierarchical inheritance limits flexibility
4. No way to opt-out of capabilities

### 7.2 Migration Strategy

**Phase 1: Extract Modules (No Breaking Changes)**

1. Create `cnp-parse` crate with modular structure
2. Extract modules as independent units
3. Current tools continue using Clap
4. Modules tested in isolation

**Phase 2: Pilot Tool (RPT)**

1. RPT migrates to modular parser
2. Composes: Base + Filters + Scopes + Routes + Flags
3. Enables semantic parsing for RPT
4. Validates module composition approach

**Phase 3: Query Tools**

1. XFD migrates (adds Logic module)
2. DSC migrates (minimal composition)
3. INX migrates (adds Logic module)
4. Validates semantic parsing across query tools

**Phase 4: Action Tools**

1. MKR migrates (Actions + Transforms)
2. EDT migrates (Filters + Transforms)
3. SMV migrates (full composition)
4. Validates hybrid parsing (positional actions + semantic filters)

**Phase 5: Deprecate Base Grammar v1.0**

1. Update Base Grammar to reference modular structure
2. Mark positional mandate as deprecated
3. Update all tool documentation
4. Release modular grammar as v2.0

### 7.3 Compatibility Plan

**Backward Compatibility:**

Tools can support BOTH old and new grammar during transition:

```rust
// Example: RPT during transition
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Try new modular parser first
    match parse_modular(&args) {
        Ok(cmd) => execute_command(cmd),
        Err(_) => {
            // Fallback to old Clap parser
            parse_clap_fallback(&args)
        }
    }
}
```

**Migration Checklist for Each Tool:**

- [ ] Identify which modules needed
- [ ] Add `cnp-parse` dependency with features
- [ ] Implement modular parsing
- [ ] Test semantic parsing
- [ ] Enable fallback to old parser (if desired)
- [ ] Update documentation
- [ ] Update tests
- [ ] Announce migration in release notes

### 7.4 Breaking Changes

**For Tool Developers:**

1. **Change**: Base Grammar no longer mandates positional structure
   - **Impact**: Tools can choose semantic or positional parsing
   - **Migration**: Update parser to use modules instead of Base Grammar structure

2. **Change**: Keywords no longer universally inherited
   - **Impact**: Tools must explicitly include modules for keywords they use
   - **Migration**: Declare feature dependencies in Cargo.toml

3. **Change**: Execution order based on semantic phases, not position
   - **Impact**: Commands parsed by meaning, not left-to-right
   - **Migration**: Update any code that assumes positional parsing

**For End Users:**

1. **Change**: Semantic tools allow position-independent arguments
   - **Impact**: `rpt -f EXT:rs` and `rpt EXT:rs -f` both work
   - **Migration**: None required (old syntax still works)

2. **Change**: Some tools may have different default behaviors
   - **Impact**: Example: `rpt` becomes `rpt tree .` explicitly
   - **Migration**: Update scripts to use explicit syntax

### 7.5 Rollback Plan

If modular approach fails, rollback strategy:

**Step 1: Identify Issues**
- Performance problems?
- Parsing complexity too high?
- User confusion?

**Step 2: Freeze Modular Development**
- Stop migrating new tools
- Keep piloted tools on modular parser
- Revert non-piloted tools to Clap

**Step 3: Document Lessons**
- What went wrong?
- What assumptions were incorrect?
- What constraints were missed?

**Step 4: Evaluate Alternatives**
- Can Clap be extended to support semantic parsing?
- Are there other parsing libraries?
- Is hybrid approach viable (Clap + custom for specific tools)?

---

## 8. Formal Grammar Notation

### 8.1 EBNF (Extended Backus-Naur Form)

All module grammars use EBNF notation:

**Notation:**
```ebnf
(* This is a comment *)

non_terminal  = production ;        (* Definition *)
| = Alternation (or)
, = Concatenation (and)
[ ... ] = Optional (zero or one)
{ ... } = Repetition (zero or more)
( ... ) = Grouping
"..." = Literal string
```

**Example:**
```ebnf
filter        = type_filter | ext_filter ;
type_filter   = "TYPE:" , file_type ;
file_type     = "file" | "dir" ;
ext_filter    = "EXT:" , extension , { "," , extension } ;
extension     = identifier ;
```

### 8.2 Complete Modular Grammar

Combining all modules:

```ebnf
(* CNP Modular Grammar - Complete *)

(* Top Level *)
cnp_command   = tool_name , { module_element } ;
tool_name     = "rpt" | "xfd" | "smv" | "dsc" | "inx" | "mkr" | "edt" ;

module_element = base_element | filter_element | scope_element | action_element
               | transform_element | logic_element | route_element | flag_element ;

(* Base Module *)
base_element  = help_flag | version_flag ;
help_flag     = "-h" | "--help" ;
version_flag  = "-v" | "--version" ;

(* Filter Module *)
filter_element = type_filter | ext_filter | name_filter | size_filter
               | time_filter | depth_filter | hidden_filter | excl_filter
               | filter_flag ;

type_filter    = "TYPE:" , file_type ;
file_type      = "file" | "dir" | "symlink" | "socket" | "pipe" | "block" | "char" ;

ext_filter     = "EXT:" , extension_list ;
extension_list = identifier , { "," , identifier } ;

name_filter    = "NAME:" , pattern ;
pattern        = glob_pattern | regex_pattern ;

size_filter    = ( "SIZE:" | "MORE:" | "LESS:" ) , size_value ;
size_value     = integer , [ size_unit ] ;
size_unit      = "B" | "KB" | "MB" | "GB" ;

time_filter    = ( "NEWER:" | "OLDER:" ) , time_value ;
time_value     = iso_date | relative_time ;

depth_filter   = ( "DEEPER:" | "SHALLOWER:" ) , integer ;

hidden_filter  = "HIDDEN:" , boolean ;

excl_filter    = "EXCL:" , pattern ;

filter_flag    = "-f" | "-d" ;

(* Scope Module *)
scope_element  = fs_scope | sys_scope | target_spec ;
fs_scope       = "tree" | "files" | "dirs" ;
sys_scope      = "sys" | "env" | "net" | "disk" | "proc" ;
target_spec    = path | multi_target ;
multi_target   = "IN:" , path , { "," , path } ;

(* Action Module *)
action_element = move_action | copy_action | remove_action | rename_action
               | archive_action | permission_action ;

move_action    = "mv" , source , destination ;
copy_action    = "cp" , source , destination ;
remove_action  = "rm" , target_list ;
rename_action  = "rename" , target_spec ;
archive_action = ( "archive" | "extract" ) , path , [ archive_format ] ;
permission_action = ( "chmod" | "chown" ) , target , permission_spec ;

(* Transform Module *)
transform_element = change_transform | regex_transform | case_transform
                  | affix_transform | number_transform ;

change_transform  = "CHANGE:" , text , ":INTO:" , text ;
regex_transform   = "REGEX:" , regex_pattern , ":INTO:" , replacement ;
case_transform    = "CASE:" , case_type ;
affix_transform   = ( "PREFIX:" | "SUFFIX:" ) , text ;
number_transform  = "NUMBER:" , number_format ;

(* Logic Module *)
logic_element  = logic_expr ;
logic_expr     = or_expr ;
or_expr        = and_expr , { "OR" , and_expr } ;
and_expr       = not_expr , { "AND" , not_expr } ;
not_expr       = [ "NOT" ] , primary_expr ;
primary_expr   = filter_element | grouped_expr | where_expr ;
grouped_expr   = "(" , logic_expr , ")" ;
where_expr     = "WHERE:" , "(" , logic_expr , ")" ;

(* Route Module *)
route_element  = to_route | into_route | format_route ;
to_route       = "TO:" , tool_name ;
into_route     = "INTO:" , file_path ;
format_route   = "FORMAT:" , format_type ;
format_type    = "json" | "csv" | "table" | "tree" | "cnp" | "toml" | "yaml" ;

(* Flag Module *)
flag_element   = short_flag | long_flag ;
short_flag     = "-" , flag_char ;
flag_char      = "r" | "p" | "y" | "n" | "q" | "V" | "i" ;
long_flag      = "--" , flag_name ;
flag_name      = "dry-run" | "force" | "recursive" | "preview" | "quiet"
               | "verbose" | "interactive" ;

(* Primitives *)
identifier     = letter , { letter | digit | "_" | "-" } ;
integer        = digit , { digit } ;
digit          = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
letter         = "a" | "b" | "c" (* ... *) | "z" | "A" | "B" | "C" (* ... *) | "Z" ;
text           = { letter | digit | symbol } ;
boolean        = "true" | "false" | "yes" | "no" ;
path           = ( "." | ".." | "/" | identifier ) , { "/" , identifier } ;
iso_date       = digit , digit , digit , digit , "-" , digit , digit , "-" , digit , digit ;
relative_time  = integer , time_unit ;
```

### 8.3 Grammar Production Rules

**Rule 1: Tokens are space-delimited**
```
Input:  "rpt tree . EXT:rs TYPE:file"
Tokens: ["rpt", "tree", ".", "EXT:rs", "TYPE:file"]
```

**Rule 2: Keywords use UPPERCASE:value syntax**
```
Valid:   EXT:rs, TYPE:file, NAME:config
Invalid: ext:rs, Type:file, name:Config
```

**Rule 3: Multi-value keywords use comma separation**
```
Valid:   EXT:rs,toml,md
Valid:   IN:src,tests,docs
Invalid: EXT:rs toml md  (space-delimited treated as separate tokens)
```

**Rule 4: Nested keywords use colon chains**
```
Valid:   CHANGE:old:INTO:new
Valid:   REGEX:pattern:INTO:replacement
Invalid: CHANGE:old INTO new  (missing connector)
```

**Rule 5: Flags are boolean (presence = true)**
```
-f means TYPE:file
-d means TYPE:dir
-r means recursive enabled
```

---

## 9. Compatibility and Interoperability

### 9.1 Cross-Tool Compatibility

**Scenario 1: Delegation (TO:)**

When delegating between tools, output format must be compatible:

```bash
# xfd finds files, delegates to smv for action
xfd . EXT:log TO:smv

# Behind the scenes:
# 1. xfd executes query: finds all .log files
# 2. xfd formats output as paths (one per line)
# 3. xfd spawns: smv <reads from stdin>
# 4. smv receives paths, performs default action
```

**Compatibility Requirements:**
- xfd must output in format smv can parse
- Standard format: newline-delimited paths
- Alternative: CNP file format (`.cnp`)

**Scenario 2: CNP File Format**

Tools can exchange data via `.cnp` files:

```bash
# xfd exports to CNP
xfd . EXT:rs FORMAT:cnp INTO:rust_files.cnp

# smv imports from CNP
smv --from-cnp=rust_files.cnp mv /backup/
```

**CNP Format Benefits:**
- Preserves metadata (size, time, perms)
- Supports complex queries
- Enables offline processing

### 9.2 Format Compatibility Matrix

| Source Tool | Output Format | Compatible Tools |
|-------------|---------------|------------------|
| XFD | Newline paths | SMV, RPT, DSC |
| XFD | JSON | RPT (with parse), External tools |
| XFD | CNP | All CNP tools |
| RPT | Tree (text) | Human readable only |
| RPT | JSON | External tools, parsing scripts |
| RPT | CNP | All CNP tools |
| SMV | Action log | Human readable, RPT can summarize |
| SMV | CNP | All CNP tools |

### 9.3 Grammar Version Compatibility

**Version Numbering:**
- `v1.0`: Current Base Grammar (hierarchical, positional)
- `v2.0`: Modular Grammar (compositional, semantic)

**Transition Period:**
- Tools specify which grammar version they support
- Parsers can support multiple versions during transition
- Users can force specific version via flag: `--grammar-version=2.0`

**Compatibility Check:**
```bash
# Check tool grammar version
rpt --grammar-version
# Output: Grammar: CNP Modular v2.0 (Modules: base, filters, scopes, routes, flags)

# Force old grammar (if supported)
rpt --grammar-version=1.0 tree .
```

### 9.4 Future Module Additions

**Planned Modules (Not Yet Implemented):**

1. **Aggregation Module**: GROUP:, SORT:, LIMIT:
2. **Hash Module**: HASH:, DUPLICATE:, CHECKSUM:
3. **Content Module**: CONTAINS:, MATCHES:, ENCODING:
4. **Network Module**: HOST:, PORT:, PROTOCOL: (for `rpt net`)
5. **Process Module**: PID:, USER:, CPU:, MEM: (for `rpt proc`)

**Adding New Modules:**

Process:
1. Propose module in `cnp-docs/proposals/`
2. Define keywords and ownership
3. Implement in `cnp-parse` crate
4. Add feature flag to Cargo.toml
5. Update tools that need module
6. Document in grammar specs

**Backward Compatibility:**
- New modules are opt-in via features
- Existing tools unaffected
- Tools can adopt new modules incrementally

---

## 10. Trade-offs and Considerations

### 10.1 Benefits

**Benefit 1: Right-Sized Grammars**
- Tools include only what they need
- No bloat from unused capabilities
- Clearer tool responsibilities

**Benefit 2: Semantic Parsing**
- Position-independent arguments
- Commands read like sentences
- Reduced cognitive load for users

**Benefit 3: Independent Evolution**
- Modules evolve separately
- No cascading changes across tools
- Easier to maintain

**Benefit 4: Clear Ownership**
- Every keyword has one owner
- No ambiguity about where features belong
- Easier to debug parsing issues

**Benefit 5: Flexible Composition**
- New tools compose existing modules
- Hybrid tools combine multiple capabilities
- Gradual adoption of new features

**Benefit 6: Testability**
- Modules tested in isolation
- Grammar validation straightforward
- Easier to catch conflicts early

### 10.2 Costs

**Cost 1: Implementation Effort**
- Estimate: 8+ weeks to implement fully
- Every tool needs migration
- Extensive testing required

**Cost 2: Complexity**
- More moving parts (modules vs monolithic)
- Need to understand composition rules
- Steeper learning curve for contributors

**Cost 3: Breaking Changes**
- Tools must migrate to new structure
- Some commands may behave differently
- Documentation updates extensive

**Cost 4: Performance Overhead**
- Module composition adds indirection
- Semantic parsing more complex than positional
- May be slower for simple commands

**Cost 5: Potential Over-Engineering**
- Risk of premature abstraction
- May not need this much flexibility
- Could be solving problems that don't exist yet

### 10.3 Risks

**Risk 1: Adoption Failure**
- Contributors may not understand modular approach
- Tools may resist migration
- Users may prefer old simplicity

**Mitigation:**
- Excellent documentation
- Clear examples
- Gradual migration with fallbacks

**Risk 2: Hidden Dependencies**
- Modules may have unexpected coupling
- Composition may not be as clean as hoped
- Conflicts may emerge later

**Mitigation:**
- Thorough testing during pilot
- Grammar independence exercise
- Clear dependency declarations

**Risk 3: Performance Issues**
- Semantic parsing may be too slow
- Module composition overhead significant
- Commands feel sluggish

**Mitigation:**
- Benchmark early in pilot
- Optimize hot paths
- Use caching where appropriate

**Risk 4: User Confusion**
- Semantic parsing may be surprising
- Position independence may violate expectations
- Error messages may be unclear

**Mitigation:**
- Clear error messages
- Good help text
- Examples in documentation

**Risk 5: Scope Creep**
- Temptation to keep adding modules
- Over-abstraction paralysis
- Never ship because it's not "perfect"

**Mitigation:**
- Stick to MVP module set
- Ship early, iterate based on feedback
- Resist adding modules until clear need

### 10.4 Alternatives Considered

**Alternative 1: Keep Clap, Accept Limitations**

Pros:
- No implementation cost
- Familiar tool
- Stable and well-tested

Cons:
- Can't solve execution order issue
- Can't enable semantic parsing
- Stuck with positional structure

**Alternative 2: Extend Clap with Custom Parser**

Pros:
- Leverage Clap for basics
- Custom code only where needed
- Gradual transition

Cons:
- Fighting Clap's design
- Maintenance burden (custom code + Clap updates)
- May be more complex than full custom parser

**Alternative 3: Use Another Parser Library**

Options explored:
- `pico-args`: Too minimal, no semantic support
- `lexopt`: Still positional-focused
- `argh`: Derive-based, not flexible enough
- `structopt`: Deprecated in favor of Clap v3

Conclusion: No existing library supports CNP's semantic + positional hybrid needs.

**Alternative 4: Hybrid Per-Tool**

Approach: Let each tool choose its parser (Clap or custom).

Pros:
- Maximum flexibility per tool
- No forced migration
- Learn from experimentation

Cons:
- Inconsistent UX across tools
- No shared parsing infrastructure
- Can't delegate cleanly between tools

**Alternative 5: Modular Grammar with Clap**

Approach: Use modular grammar design, but implement with Clap.

Pros:
- Benefit from Clap's maturity
- Modular grammar still conceptually clean
- Less implementation work

Cons:
- Clap doesn't support semantic parsing well
- Would need significant Clap customization
- May not be possible given Clap's architecture

---

## 11. Decision Framework

### 11.1 Questions to Answer

Before committing to modular grammar, answer these questions:

**Question 1: Is semantic parsing worth the cost?**
- Does position-independence significantly improve UX?
- Are users actually confused by positional requirements?
- Will "reads like a sentence" benefit justify effort?

**Question 2: Is modular composition actually cleaner?**
- Does grammar independence hold under real usage?
- Are modules truly independent or tightly coupled?
- Does composition simplify or complicate development?

**Question 3: Can we achieve goals with less effort?**
- Can Clap be extended to solve execution order issue?
- Can we fix RPT's bypass logic without rewriting parser?
- Is a hybrid approach (some tools custom, some Clap) viable?

**Question 4: Will this pay off long-term?**
- Are we building more tools that need this flexibility?
- Will CNP ecosystem grow enough to justify investment?
- Or is this over-engineering for current needs?

**Question 5: What's the migration path?**
- Can tools migrate incrementally?
- Can users opt-in to new grammar gradually?
- What's the rollback plan if it doesn't work?

### 11.2 Decision Criteria

**Proceed with Modular Grammar if:**
- ✅ Semantic parsing is high-value for UX
- ✅ Module independence validated in pilot
- ✅ Performance acceptable in benchmarks
- ✅ Team understands and supports approach
- ✅ Clear migration path with rollback option
- ✅ Benefits justify 8+ week investment

**Do NOT proceed if:**
- ❌ Current tools work well enough as-is
- ❌ Module coupling emerges during pilot
- ❌ Performance unacceptable
- ❌ Team doesn't understand or support
- ❌ Simpler solution available
- ❌ Cost exceeds expected benefit

### 11.3 Validation Approach

**Phase 1: Pilot (Weeks 1-3)**
1. Build Base, Filter, Scope, Route, Flag modules
2. Migrate RPT to use modules
3. Compare before/after: parsing speed, binary size, UX
4. Identify unexpected coupling or issues

**Decision Point 1:** Proceed to Phase 2 only if pilot successful.

**Phase 2: Query Tools (Weeks 4-6)**
1. Add Logic module
2. Migrate XFD (complex logic) and DSC (simple)
3. Validate semantic parsing across different complexity levels
4. Get user feedback on position-independence

**Decision Point 2:** Proceed to Phase 3 only if query tools validate approach.

**Phase 3: Action Tools (Weeks 7-9)**
1. Add Action and Transform modules
2. Migrate MKR (simple actions) and EDT (filters + transforms)
3. Validate hybrid parsing (positional actions + semantic filters)
4. Test SMV complexity (full composition)

**Decision Point 3:** Commit to full rollout or rollback.

**Phase 4: Finalization (Weeks 10-12)**
1. Complete all tool migrations
2. Update all documentation
3. Deprecate Base Grammar v1.0
4. Release modular grammar as v2.0

### 11.4 Success Metrics

**Metric 1: Parsing Performance**
- Target: Modular parser ≤ 2x slower than Clap (acceptable trade-off)
- Measure: Benchmark 1000 command parses, compare times

**Metric 2: Binary Size**
- Target: No more than 10% increase in tool binary size
- Measure: Compare RPT before/after modular migration

**Metric 3: User Satisfaction**
- Target: 80%+ prefer semantic parsing after trying both
- Measure: Survey users, collect feedback

**Metric 4: Developer Productivity**
- Target: New tools easier to build with modular approach
- Measure: Time to build hypothetical new tool (before/after)

**Metric 5: Bug Reduction**
- Target: Fewer parsing-related bugs reported
- Measure: Compare issue count 3 months before/after

**Metric 6: Code Maintainability**
- Target: Modules have clear boundaries, easy to modify independently
- Measure: Developer feedback, code review observations

### 11.5 Final Recommendation

**Recommendation: Proceed with Pilot**

**Rationale:**
1. Current bypass logic in RPT is fragile and confusing
2. Execution order issue is real and affects UX
3. Hierarchical grammar is showing strain as tools evolve
4. Modular approach aligns with "caveman sentence" philosophy
5. Pilot provides low-risk validation before full commitment

**Scope:**
- Build modular parser infrastructure
- Migrate RPT as pilot
- Validate module independence
- Gather performance data
- Make informed decision before broader rollout

**Timeline:**
- Weeks 1-3: Pilot (RPT migration)
- Week 3: Decision point (proceed or rollback)
- If proceeding: Weeks 4-12 for full migration

**Exit Criteria:**
- Pilot successful: Continue to Phase 2
- Pilot problematic: Rollback, keep Clap, solve RPT issue differently
- No sunken cost fallacy: Willing to abandon if it doesn't work

---

## Appendix A: Grammar Independence Exercise Results

During architectural planning, we validated module independence:

**Test 1: Can Filters work without Logic?**
✅ Yes. Simple AND combination of filters works without Logic module.

**Test 2: Can Actions work without Filters?**
✅ Yes. Actions can operate on explicit paths without filtering.

**Test 3: Can Transforms work without Actions?**
✅ Yes. Transforms can generate new names without executing actions (preview mode).

**Test 4: Can Routes work without any other module?**
✅ Yes. Routes can redirect output regardless of what generated it.

**Test 5: Do any modules have hidden dependencies?**
⚠️ Partial. Logic module semantically depends on Filters (logic operates on filter expressions), but can be implemented independently if it accepts generic predicates.

**Conclusion:** Modules are sufficiently independent for composition approach.

---

## Appendix B: Example Tool Implementations

### B.1 RPT with Modular Grammar

```rust
// rpt/src/main.rs
use cnp_parse::{BaseModule, FilterModule, ScopeModule, RouteModule, FlagModule, tokenize};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Tokenize input
    let tokens = tokenize(&args[1..])?;

    // Parse modules (order doesn't matter for semantic parsing)
    let base = BaseModule::parse(&tokens)?;
    let scope = ScopeModule::parse(&tokens)?;
    let filters = FilterModule::parse(&tokens)?;
    let routes = RouteModule::parse(&tokens)?;
    let flags = FlagModule::parse(&tokens)?;

    // Build command
    let cmd = RptCommand { scope, filters, routes, flags };

    // Execute
    let output = execute_rpt(cmd)?;

    // Apply routing
    routes.apply(output)?;

    Ok(())
}
```

### B.2 XFD with Logic Module

```rust
// xfd/src/main.rs
use cnp_parse::{BaseModule, FilterModule, LogicModule, RouteModule, FlagModule, tokenize};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let tokens = tokenize(&args[1..])?;

    // Parse modules
    let filters = FilterModule::parse(&tokens)?;
    let logic = LogicModule::parse(&tokens, &filters)?; // Logic depends on filters
    let routes = RouteModule::parse(&tokens)?;
    let flags = FlagModule::parse(&tokens)?;

    // Execute query
    let results = if logic.has_expressions() {
        // Complex logic: use logic module to evaluate
        execute_with_logic(&logic, &flags)?
    } else {
        // Simple filters: use filter module directly
        execute_with_filters(&filters, &flags)?
    };

    // Format and route output
    let output = format_results(results)?;
    routes.apply(output)?;

    Ok(())
}
```

### B.3 SMV with Full Composition

```rust
// smv/src/main.rs
use cnp_parse::{
    BaseModule, FilterModule, ActionModule, TransformModule,
    LogicModule, RouteModule, FlagModule, tokenize
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let tokens = tokenize(&args[1..])?;

    // Parse modules
    let action = ActionModule::parse(&tokens)?; // Positional (first)
    let filters = FilterModule::parse(&tokens)?; // Semantic
    let transforms = TransformModule::parse(&tokens)?; // Semantic
    let logic = LogicModule::parse(&tokens, &filters)?; // Semantic
    let routes = RouteModule::parse(&tokens)?; // Semantic
    let flags = FlagModule::parse(&tokens)?; // Semantic

    // Build targets (action + filters + logic)
    let targets = resolve_targets(&action, &filters, &logic)?;

    // Apply transforms to get new names
    let transformed = transforms.apply_to_targets(targets)?;

    // Preview or execute
    if flags.preview {
        preview_action(&action, &transformed)?;
    } else {
        execute_action(&action, &transformed)?;
    }

    // Route output
    let output = format_action_log(&action)?;
    routes.apply(output)?;

    Ok(())
}
```

---

## Appendix C: Glossary

**Base Grammar**: Original CNP grammar specification (v1.0) defining hierarchical inheritance.

**Module**: Self-contained grammar component owning specific keywords and parsing logic.

**Composition**: Assembling tool grammar from independent modules (vs inheritance).

**Semantic Parsing**: Position-independent parsing based on keyword meaning.

**Positional Parsing**: Order-dependent parsing based on argument position.

**Hybrid Parsing**: Combination of positional (for actions) and semantic (for filters/routes).

**Delegation**: Passing output from one CNP tool to another via `TO:` keyword.

**Grammar Independence**: Property where modules can function without depending on others.

**Execution Phase**: Semantic stage in command execution (type filter → traversal → content filter → logic → action → route).

**Caveman Sentence**: CNP philosophy of using content words with minimal punctuation (inspired by simplified speech).

**Flag Alias**: Short flag that expands to keyword form (`-f` → `TYPE:file`).

**Universal Keyword**: Keyword previously mandated in Base Grammar for all tools (now optional via modules).

**Feature Flag**: Cargo.toml feature enabling specific grammar module inclusion.

**Keyword Ownership**: Principle that each keyword belongs to exactly one module.

**Route**: Output redirection keyword (`TO:`, `INTO:`, `FORMAT:`).

**Transform**: Content/name modification keyword (`CHANGE:INTO:`, `REGEX:INTO:`, etc.).

---

## Document Metadata

**Version**: 1.0 DRAFT
**Authors**: CNP Development Team
**Date**: 2025-11-15
**Status**: Proposal - Pending Review
**Related Documents**:
- `CNP_Modular_Parser_Architecture.md` (companion implementation doc)
- `CNP_Base_Grammar.md` (v1.0 - current standard)
- `CNP_Query_Grammar_v2_DRAFT.md` (semantic vision)
- `CNP_Action_Grammar.md` (action tool standard)

**Feedback**:
Please provide feedback on this proposal by:
1. Opening issue in `cnp-docs` repo
2. Commenting on relevant sections
3. Proposing alternative designs
4. Validating assumptions with real usage

**Next Steps**:
1. Team review and discussion
2. Pilot implementation (RPT migration)
3. Validate module independence
4. Decision: proceed or rollback
