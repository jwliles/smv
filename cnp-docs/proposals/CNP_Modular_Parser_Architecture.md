# CNP Modular Parser Architecture

**Status:** PROPOSAL
**Date:** 2025-11-15
**Author:** CNP Development Team
**Purpose:** Define a compositional parser architecture for CNP ecosystem tools

---

## Executive Summary

This document proposes a fundamental shift from **hierarchical grammar inheritance** to **compositional grammar modules**. Instead of monolithic Query and Action grammars, CNP will provide fine-grained grammar modules that tools compose based on their needs.

**Key Change:**
```
FROM: Tools inherit from broad grammars (Query or Action)
TO:   Tools compose from narrow modules (Filters + Actions + Logic + ...)
```

**Benefits:**
- Tools only include parsing logic they need
- Grammar evolution affects only relevant tools
- Clear ownership boundaries
- Easier testing and validation
- Enables tools that don't fit Query/Action dichotomy

**Cost:**
- Requires reworking all existing tools
- New mental model for grammar composition
- More complex initial setup

---

## Table of Contents

1. [Background: The Problem](#background-the-problem)
2. [Current Architecture (Hierarchical)](#current-architecture-hierarchical)
3. [Proposed Architecture (Modular)](#proposed-architecture-modular)
4. [Grammar Module Specifications](#grammar-module-specifications)
5. [Tool Composition Examples](#tool-composition-examples)
6. [Implementation Strategy](#implementation-strategy)
7. [Migration Path](#migration-path)
8. [Trade-off Analysis](#trade-off-analysis)
9. [Decision Framework](#decision-framework)

---

## Background: The Problem

### The Cascade of Issues

CNP's development revealed a cascade of interconnected problems:

**Layer 1: UX Issue**
```bash
rpt files .              # Subcommand implies "files only"
rpt files . TYPE:dir     # But can show dirs? Contradictory!
```
Resolution: Rename `files` → `tree` (neutral name)

**Layer 2: Semantic Consistency**
```bash
rpt tree . TYPE:file     # Verbose
rpt tree . -f            # Shorthand - but now -f is a flag AND a filter
```
Issue: Type constraints (`-f`/`-d`) are semantically filters, but syntactically flags.

**Layer 3: Execution Order**
```bash
rpt tree /large/directory -f EXT:rs

# Clap processes flags AFTER positional args:
1. Traverse /large/directory (processes files AND directories)
2. Apply -f flag (discard directories)  ← Wasted work!
```
Issue: Type constraints must execute BEFORE traversal, but flags execute AFTER.

**Layer 4: Parser Architecture**

Clap's positional model can't reorder execution. To fix Layer 3, we need semantic/phase-based parsing. But semantic parsing requires rethinking the entire grammar architecture.

### The Fundamental Question

**If we're rebuilding parsing, should we also fix the grammar structure?**

Current grammars (Query and Action) are monolithic:
- **Query Grammar** includes filters, scopes, logical operators, flags (too broad)
- **Action Grammar** includes actions, transforms, targets (too broad)
- **SMV needs BOTH** Query filters and Action operations (dependencies unclear)

This proposal addresses the grammar structure problem while building the semantic parser.

---

## Current Architecture (Hierarchical)

### Grammar Hierarchy

```
CNP Base Grammar v1.0
    ├─→ CNP Query Grammar v1.0
    │   └─→ Used by: rpt, xfd, dsc, inx
    │
    └─→ CNP Action Grammar v1.0
        └─→ Used by: smv, edt, mkr
```

### Base Grammar Contents

**Universal Elements:**
- Routes: `TO:`, `INTO:`, `FORMAT:`
- Meta flags: `-h`, `-v`
- Universal filters: `NAME:`, `EXT:`, `TYPE:`, `SIZE:`, etc.

### Query Grammar Contents

**Extends Base with:**
- Scopes: `tree`, `sys`, `disk`, `net`
- Query flags: `-f`, `-d`, `-r`
- Logical operators: `AND`, `OR`, `NOT`
- Semantic groups: `FOR:notes`, `FOR:media`

### Action Grammar Contents

**Extends Base with:**
- Actions: `snake`, `kebab`, `mv`, `cp`, `rm`
- Transformations: `CHANGE:INTO:`, `REGEX:INTO:`
- Action flags: `-p`, `-f`, `-i`, `-u`
- Source/destination semantics

### Problems with Current Architecture

**1. Monolithic Grammars**

Query Grammar is a kitchen sink:
```
Filters + Scopes + Logic + Flags + Semantic Groups
```

Problem: RPT needs filters and scopes, but not logic. XFD needs everything. No way to opt out of unused features.

**2. Unclear Dependencies**

SMV uses Query filters in Action commands:
```bash
smv rm . EXT:log TYPE:file -r
         ^^^^^^^^ ^^^^^^^^^^
         Query filters in Action context
```

Question: Does Action Grammar depend on Query Grammar? Or do both depend on some shared filter grammar?

**3. Grammar Scope Creep**

Adding features requires deciding "Is this Query or Action?"
```
Where does SORT: belong?
- Query (sort results before output)?
- Action (organize files)?
- Both (duplicated)?
```

**4. Testing Boundaries**

Can't test filters independently of scopes. Can't test actions independently of transforms. Modules aren't isolated.

**5. Tool Coupling**

Tools inherit entire grammars even if they only need parts:
```rust
// RPT needs filters + scopes, not logic
// But must include all of Query Grammar
use cnp_grammar::query::*;  // Brings everything
```

---

## Proposed Architecture (Modular)

### Core Principle

**Grammars are composed, not inherited.**

```
CNP Base (Substrate)
    │
    ├─→ Filter Module
    ├─→ Scope Module
    ├─→ Action Module
    ├─→ Transform Module
    ├─→ Logic Module
    ├─→ Route Module
    └─→ Flag Module

Tools compose needed modules:
    RPT = Base + Filter + Scope + Route + Flag
    XFD = Base + Filter + Scope + Logic + Route + Flag
    SMV = Base + Filter + Action + Transform + Route + Flag
```

### Key Changes

**FROM:**
```
Query Grammar = Filters + Scopes + Logic + Flags
Action Grammar = Actions + Transforms + Flags

Problem: Monolithic, all-or-nothing
```

**TO:**
```
Filter Module = Just filters (TYPE:, EXT:, SIZE:, etc.)
Scope Module = Just scopes (tree, sys, disk)
Action Module = Just actions (snake, mv, cp)
Transform Module = Just transforms (CHANGE:INTO:)
Logic Module = Just logical operators (AND, OR, NOT)
...

Solution: Granular, compose what you need
```

### Benefits

**1. Precise Dependencies**
```toml
# rpt/Cargo.toml
[dependencies]
cnp-parse = { features = ["filters", "scopes"] }
# Gets: Only filters + scopes
# Skips: Actions, transforms, logic (not compiled in)
```

**2. Independent Evolution**
```
Change Filter Module:
  → Affects: RPT, XFD, SMV (all use filters)
  → Doesn't affect: Scope, Action modules

Change Logic Module:
  → Affects: XFD, DSC
  → Doesn't affect: RPT, SMV
```

**3. Clear Ownership**
```
TYPE:file filter   → filters module
snake action       → actions module
tree scope         → scopes module
AND operator       → logic module
```

**4. Flexible Composition**
```
New tool: "File classifier" (passive analysis)
  Compose: filters + routes
  Skip: scopes, actions, transforms

New tool: "Batch renamer" (action without filters)
  Compose: actions + transforms
  Skip: filters, scopes
```

**5. Testing Isolation**
```rust
// Test filters without scopes
#[test]
fn filter_parsing() {
    use cnp_parse::filters;
    assert_eq!(filters::parse("TYPE:file"), Filter::Type(File));
}

// Test actions without filters
#[test]
fn action_parsing() {
    use cnp_parse::actions;
    assert_eq!(actions::parse("snake"), Action::Snake);
}
```

---

## Grammar Module Specifications

### Base Module (Always Required)

**Purpose:** Tokenization substrate - recognizes patterns but doesn't interpret them.

**Provides:**
```rust
pub enum Token {
    Keyword { key: String, value: String },  // UPPERCASE:value
    Flag(String),                             // -flag
    Path(PathBuf),                           // ./path or /absolute
    Bare(String),                            // word
    QuotedString(String),                    // "string with spaces"
    Grouping(char),                          // ( or )
    LogicalOp(String),                       // AND, OR, NOT
}

pub fn tokenize(input: &str) -> Vec<Token>;
```

**Responsibilities:**
- Split input into tokens
- Recognize keyword pattern (`UPPERCASE:value`)
- Identify flags (starts with `-`)
- Handle quoted strings
- Preserve grouping characters
- Identify potential logical operators

**Does NOT:**
- Validate keyword names
- Interpret flag meanings
- Resolve paths
- Parse expressions

**Feature flag:** `base` (always included)

---

### Filter Module

**Purpose:** Parse and validate filter keywords used for item selection.

**Keywords:**
```
TYPE:file|dir|symlink|other
EXT:extension[,extension...]
NAME:pattern
SIZE:>value | SIZE:<value
MORE:size_value
LESS:size_value
DEPTH:N
HIDDEN:true|false
EXCL:pattern[,pattern...]
MODIFIED:>date | MODIFIED:<date
ACCESSED:>date | ACCESSED:<date
CREATED:>date | CREATED:<date
```

**Provides:**
```rust
#[cfg(feature = "filters")]
pub mod filters {
    pub enum Filter {
        Type(TypeFilter),
        Ext(Vec<String>),
        Name(String),
        Size(SizeOp, u64),
        More(u64),
        Less(u64),
        Depth(usize),
        Hidden(bool),
        Excl(Vec<String>),
        Modified(TimeOp, DateTime),
        Accessed(TimeOp, DateTime),
        Created(TimeOp, DateTime),
    }

    pub enum TypeFilter { File, Dir, Symlink, Other, Both }
    pub enum SizeOp { GreaterThan, LessThan, Equal }
    pub enum TimeOp { After, Before }

    pub fn parse(token: &Token) -> Result<Filter, ParseError>;
    pub fn parse_all(tokens: &[Token]) -> Vec<Filter>;
}
```

**Used by:**
- Query tools: RPT, XFD, DSC, INX (filter query results)
- Action tools: SMV, EDT (filter action targets)

**Dependencies:**
- `base` (for Token type)

**Feature flag:** `filters`

**Example:**
```rust
use cnp_parse::{tokenize, filters};

let tokens = tokenize("TYPE:file EXT:rs SIZE:>1MB");
let filters = filters::parse_all(&tokens);
// Returns: [Filter::Type(File), Filter::Ext(["rs"]), Filter::Size(GT, 1048576)]
```

---

### Scope Module

**Purpose:** Parse query scope identifiers.

**Keywords:**
```
tree   - Filesystem hierarchy view
sys    - System information
disk   - Disk usage and filesystem stats
net    - Network interfaces and connectivity
env    - Environment variables
stats  - Directory statistics
```

**Provides:**
```rust
#[cfg(feature = "scopes")]
pub mod scopes {
    pub enum Scope {
        Tree,
        Sys,
        Disk,
        Net,
        Env,
        Stats,
    }

    pub fn parse(token: &Token) -> Result<Scope, ParseError>;
    pub fn find_scope(tokens: &[Token]) -> Option<Scope>;
}
```

**Used by:**
- Query tools: RPT, XFD, DSC (define query domain)

**Not used by:**
- Action tools (actions don't have scopes)

**Dependencies:**
- `base` (for Token type)

**Feature flag:** `scopes`

**Example:**
```rust
use cnp_parse::{tokenize, scopes};

let tokens = tokenize("tree . TYPE:file");
let scope = scopes::find_scope(&tokens);
// Returns: Some(Scope::Tree)
```

---

### Action Module

**Purpose:** Parse imperative action verbs.

**Keywords:**
```
# Transformations
snake      - Convert to snake_case
kebab      - Convert to kebab-case
pascal     - Convert to PascalCase
camel      - Convert to camelCase
title      - Convert to Title Case
upper      - Convert to UPPERCASE
lower      - Convert to lowercase
clean      - Remove special characters

# File operations
mv         - Move files
cp         - Copy files
rm         - Remove files

# Organization
organize   - Group files by criteria
flatten    - Flatten directory structure
sort       - Sort files by criteria
group      - Group files into directories
```

**Provides:**
```rust
#[cfg(feature = "actions")]
pub mod actions {
    pub enum Action {
        // Transformations
        Snake,
        Kebab,
        Pascal,
        Camel,
        Title,
        Upper,
        Lower,
        Clean,

        // Operations
        Move,
        Copy,
        Remove,

        // Organization
        Organize,
        Flatten,
        Sort,
        Group,
    }

    pub fn parse(token: &Token) -> Result<Action, ParseError>;
    pub fn find_action(tokens: &[Token]) -> Option<Action>;
}
```

**Used by:**
- Action tools: SMV, EDT, MKR

**Not used by:**
- Query tools (queries don't perform actions)

**Dependencies:**
- `base` (for Token type)

**Feature flag:** `actions`

**Example:**
```rust
use cnp_parse::{tokenize, actions};

let tokens = tokenize("snake . EXT:md");
let action = actions::find_action(&tokens);
// Returns: Some(Action::Snake)
```

---

### Transform Module

**Purpose:** Parse multi-token transformation directives.

**Keywords:**
```
CHANGE:"old_value" INTO:"new_value"
REGEX:"pattern" INTO:"replacement"
```

**Provides:**
```rust
#[cfg(feature = "transforms")]
pub mod transforms {
    pub enum Transform {
        Change { from: String, to: String },
        Regex { pattern: String, replacement: String },
    }

    pub fn parse(tokens: &[Token]) -> Result<Transform, ParseError>;
    pub fn find_transform(tokens: &[Token]) -> Option<Transform>;
}
```

**Used by:**
- SMV (filename transformations)
- EDT (content transformations)

**Dependencies:**
- `base` (for Token type)
- Optionally `filters` (transformations often apply to filtered targets)

**Feature flag:** `transforms`

**Example:**
```rust
use cnp_parse::{tokenize, transforms};

let tokens = tokenize(r#"CHANGE:"IMG_" INTO:"photo_" . EXT:jpg"#);
let transform = transforms::find_transform(&tokens);
// Returns: Some(Transform::Change { from: "IMG_", to: "photo_" })
```

---

### Logic Module

**Purpose:** Parse logical operators and build expression trees.

**Keywords:**
```
AND    - Logical conjunction
OR     - Logical disjunction
NOT    - Logical negation
(...)  - Grouping/precedence
```

**Provides:**
```rust
#[cfg(feature = "logic")]
pub mod logic {
    pub enum Expression<T> {
        Term(T),                              // Single filter/condition
        And(Box<Expression<T>>, Box<Expression<T>>),
        Or(Box<Expression<T>>, Box<Expression<T>>),
        Not(Box<Expression<T>>),
        Group(Box<Expression<T>>),            // Parenthesized
    }

    pub fn parse_expression<T, F>(
        tokens: &[Token],
        term_parser: F
    ) -> Result<Expression<T>, ParseError>
    where
        F: Fn(&Token) -> Result<T, ParseError>;
}
```

**Used by:**
- XFD (complex queries: `(TYPE:file AND EXT:rs) OR NAME:config`)
- DSC (semantic groups: `FOR:notes OR FOR:configs`)
- Potentially SMV (complex filter expressions)

**Not used by:**
- RPT (simple filters only, no logic needed)

**Dependencies:**
- `base` (for Token type)
- Generic over term type (works with any parseable element)

**Feature flag:** `logic`

**Example:**
```rust
use cnp_parse::{tokenize, filters, logic};

let tokens = tokenize("(TYPE:file AND EXT:rs) OR NAME:config");
let expr = logic::parse_expression(&tokens, |t| filters::parse(t));
// Returns: Or(
//   And(Term(Type(File)), Term(Ext("rs"))),
//   Term(Name("config"))
// )
```

---

### Route Module

**Purpose:** Parse universal output routing directives.

**Keywords:**
```
TO:tool       - Delegate to another CNP tool
INTO:file     - Write output to file
FORMAT:type   - Specify output format (json, csv, yaml, table, tree, cnp)
```

**Provides:**
```rust
#[cfg(feature = "routes")]
pub mod routes {
    pub enum Route {
        To(String),
        Into(PathBuf),
        Format(OutputFormat),
    }

    pub enum OutputFormat {
        Json,
        Csv,
        Yaml,
        Table,
        Tree,
        Cnp,
        Plain,
    }

    pub fn parse(token: &Token) -> Result<Route, ParseError>;
    pub fn parse_all(tokens: &[Token]) -> Vec<Route>;
}
```

**Used by:**
- ALL tools (routes are universal)

**Dependencies:**
- `base` (for Token type)

**Feature flag:** `routes` (typically in default features)

**Example:**
```rust
use cnp_parse::{tokenize, routes};

let tokens = tokenize("tree . FORMAT:json INTO:output.json");
let routes = routes::parse_all(&tokens);
// Returns: [Format(Json), Into("output.json")]
```

---

### Flag Module

**Purpose:** Parse and categorize flag arguments.

**Flags:**
```
# Meta (universal)
-h, --help
-v, --version

# Query-specific
-f, --files-only       (alias for TYPE:file)
-d, --dirs-only        (alias for TYPE:dir)
-r, --recursive

# Action-specific
-p, --preview
-f, --force
-i, --interactive
-u, --undo
-n, --no-clobber
```

**Provides:**
```rust
#[cfg(feature = "flags")]
pub mod flags {
    pub enum Flag {
        Meta(MetaFlag),
        Query(QueryFlag),
        Action(ActionFlag),
    }

    pub enum MetaFlag {
        Help,
        Version,
    }

    pub enum QueryFlag {
        FilesOnly,
        DirsOnly,
        Recursive,
    }

    pub enum ActionFlag {
        Preview,
        Force,
        Interactive,
        Undo,
        NoClobber,
    }

    pub fn parse(token: &Token) -> Result<Flag, ParseError>;
    pub fn parse_all(tokens: &[Token]) -> Vec<Flag>;
}
```

**Used by:**
- ALL tools (but different subsets)

**Dependencies:**
- `base` (for Token type)

**Feature flag:** `flags` (typically in default features)

**Note on aliases:** In semantic parsing, `-f` becomes an alias for `TYPE:file`. The flag module handles the syntactic recognition; the semantic resolver converts it to the appropriate filter.

**Example:**
```rust
use cnp_parse::{tokenize, flags};

let tokens = tokenize("tree . -f -r");
let flags = flags::parse_all(&tokens);
// Returns: [Query(FilesOnly), Query(Recursive)]
```

---

## Tool Composition Examples

### Example 1: RPT (Simple Query Tool)

**Needs:**
- Filters: to select files/directories
- Scopes: to define query domain (tree, sys, disk)
- Routes: to output results
- Flags: for shortcuts and meta operations

**Cargo.toml:**
```toml
[package]
name = "rpt"

[dependencies]
cnp-parse = {
    version = "0.1.0",
    features = ["filters", "scopes", "routes", "flags"]
}
```

**Parsing code:**
```rust
use cnp_parse::{tokenize, filters, scopes, routes, flags};

pub struct RptCommand {
    pub scope: Scope,
    pub filters: Vec<Filter>,
    pub routes: Vec<Route>,
    pub flags: Vec<Flag>,
    pub path: Option<PathBuf>,
}

pub fn parse_rpt(input: &str) -> Result<RptCommand> {
    let tokens = tokenize(input);

    // Each module extracts what it recognizes
    let scope = scopes::find_scope(&tokens)
        .ok_or("No scope specified")?;
    let filters = filters::parse_all(&tokens);
    let routes = routes::parse_all(&tokens);
    let flags = flags::parse_all(&tokens);

    // Extract path (any token that's a valid path)
    let path = tokens.iter()
        .find_map(|t| match t {
            Token::Path(p) => Some(p.clone()),
            Token::Bare(".") => Some(PathBuf::from(".")),
            _ => None
        });

    Ok(RptCommand { scope, filters, routes, flags, path })
}
```

**Binary size impact:**
- Includes: base, filters, scopes, routes, flags (~15KB)
- Excludes: actions, transforms, logic (~10KB saved)

---

### Example 2: XFD (Complex Query Tool with Logic)

**Needs:**
- Filters: to select files
- Scopes: to define query domain
- Logic: for complex expressions (`(A AND B) OR C`)
- Routes: to output results
- Flags: for shortcuts

**Cargo.toml:**
```toml
[package]
name = "xfd"

[dependencies]
cnp-parse = {
    version = "0.1.0",
    features = ["filters", "scopes", "logic", "routes", "flags"]
}
```

**Parsing code:**
```rust
use cnp_parse::{tokenize, filters, scopes, logic, routes, flags};

pub struct XfdCommand {
    pub scope: Scope,
    pub expression: Expression<Filter>,  // Logical expression tree
    pub routes: Vec<Route>,
    pub flags: Vec<Flag>,
    pub path: Option<PathBuf>,
}

pub fn parse_xfd(input: &str) -> Result<XfdCommand> {
    let tokens = tokenize(input);

    let scope = scopes::find_scope(&tokens)
        .unwrap_or(Scope::Tree);

    // Logic module builds expression tree using filter parser
    let expression = logic::parse_expression(&tokens, |t| {
        filters::parse(t)
    })?;

    let routes = routes::parse_all(&tokens);
    let flags = flags::parse_all(&tokens);

    let path = extract_path(&tokens);

    Ok(XfdCommand { scope, expression, routes, flags, path })
}
```

**Example query:**
```bash
xfd . (TYPE:file AND EXT:rs) OR (TYPE:dir AND NAME:config)
```

**Binary size impact:**
- Includes: base, filters, scopes, logic, routes, flags (~20KB)
- Excludes: actions, transforms (~8KB saved)

---

### Example 3: SMV (Action Tool with Filters)

**Needs:**
- Actions: to specify operation (snake, mv, cp, etc.)
- Transforms: for CHANGE:INTO: operations
- Filters: to select action targets
- Routes: to output results
- Flags: for preview, force, undo, etc.

**Cargo.toml:**
```toml
[package]
name = "smv"

[dependencies]
cnp-parse = {
    version = "0.1.0",
    features = ["actions", "transforms", "filters", "routes", "flags"]
}
```

**Parsing code:**
```rust
use cnp_parse::{tokenize, actions, transforms, filters, routes, flags};

pub struct SmvCommand {
    pub action: Action,
    pub transform: Option<Transform>,
    pub filters: Vec<Filter>,
    pub routes: Vec<Route>,
    pub flags: Vec<Flag>,
    pub targets: Vec<PathBuf>,
}

pub fn parse_smv(input: &str) -> Result<SmvCommand> {
    let tokens = tokenize(input);

    let action = actions::find_action(&tokens)
        .ok_or("No action specified")?;

    let transform = transforms::find_transform(&tokens);
    let filters = filters::parse_all(&tokens);
    let routes = routes::parse_all(&tokens);
    let flags = flags::parse_all(&tokens);

    let targets = extract_paths(&tokens);

    Ok(SmvCommand { action, transform, filters, routes, flags, targets })
}
```

**Example commands:**
```bash
# Transform with filters
smv snake . EXT:md TYPE:file -p

# Transform with explicit change
smv CHANGE:"IMG_" INTO:"photo_" . EXT:jpg -r

# Move with filters
smv mv src/ dest/ EXT:rs -f
```

**Binary size impact:**
- Includes: base, actions, transforms, filters, routes, flags (~22KB)
- Excludes: scopes, logic (~6KB saved)

---

### Example 4: Hypothetical "File Classifier" (Read-only Analysis)

**Needs:**
- Filters: to select files
- Routes: to output classification results

**Doesn't need:**
- Scopes (no query domains, just current directory)
- Actions (passive analysis, no mutations)
- Transforms (no modifications)
- Logic (simple filters only)

**Cargo.toml:**
```toml
[package]
name = "cnp-classifier"

[dependencies]
cnp-parse = {
    version = "0.1.0",
    features = ["filters", "routes"]
}
# Minimal: just filters + routes
```

**Parsing code:**
```rust
use cnp_parse::{tokenize, filters, routes};

pub struct ClassifierCommand {
    pub filters: Vec<Filter>,
    pub routes: Vec<Route>,
    pub path: PathBuf,
}

pub fn parse_classifier(input: &str) -> Result<ClassifierCommand> {
    let tokens = tokenize(input);

    let filters = filters::parse_all(&tokens);
    let routes = routes::parse_all(&tokens);
    let path = extract_path(&tokens).unwrap_or_else(|| PathBuf::from("."));

    Ok(ClassifierCommand { filters, routes, path })
}
```

**Binary size impact:**
- Includes: base, filters, routes (~10KB)
- Excludes: scopes, actions, transforms, logic (~15KB saved)

**This tool wouldn't fit Query or Action grammars, but composes easily with modules.**

---

## Implementation Strategy

### Phase 1: Foundation (Week 1)

**Goal:** Build base tokenization and test with all existing commands.

**Tasks:**
1. Create `cnp-parse` crate with feature structure
2. Implement `base.rs` module:
   - Token enum with all types
   - `tokenize()` function with full pattern recognition
   - Quoted string handling
   - Parenthesis tracking
   - Logical operator recognition
3. Write comprehensive tokenization tests
4. Test tokenizer with commands from ALL tools (RPT, XFD, SMV, etc.)

**Success criteria:**
- Tokenizer handles all existing command patterns
- No tool-specific assumptions in base
- All tests pass

**Deliverable:** Working tokenizer that all modules will build upon.

---

### Phase 2: Core Modules (Week 2)

**Goal:** Implement most commonly used modules.

**Tasks:**
1. Implement `filters.rs`:
   - All filter types (TYPE, EXT, SIZE, etc.)
   - Value parsing (sizes, dates, etc.)
   - Validation
   - Tests with RPT and SMV commands
2. Implement `routes.rs`:
   - TO, INTO, FORMAT parsing
   - Output format enum
   - Tests with all tools
3. Implement `flags.rs`:
   - Meta, Query, Action flag categories
   - Tests with all tools

**Success criteria:**
- Filters work in both Query and Action contexts
- Routes parse correctly
- Flags categorize correctly

**Deliverable:** Core modules that most tools need.

---

### Phase 3: Specialized Modules (Week 3)

**Goal:** Implement domain-specific modules.

**Tasks:**
1. Implement `scopes.rs`:
   - Scope enum (Tree, Sys, Disk, etc.)
   - Scope detection
   - Tests with RPT, XFD commands
2. Implement `actions.rs`:
   - Action enum (Snake, Mv, Cp, etc.)
   - Action detection
   - Tests with SMV commands
3. Implement `transforms.rs`:
   - Multi-token parsing (CHANGE:INTO:)
   - Quoted string handling
   - Tests with SMV commands

**Success criteria:**
- Scopes work in Query tools
- Actions work in Action tools
- Transforms handle complex syntax

**Deliverable:** Domain-specific modules.

---

### Phase 4: Advanced Features (Week 4)

**Goal:** Implement logic module and integrate everything.

**Tasks:**
1. Implement `logic.rs`:
   - Expression tree building
   - Precedence handling
   - Parenthesis matching
   - Generic over term type
   - Tests with XFD complex queries
2. Integration testing:
   - Test all module combinations
   - Test tool composition patterns
   - Performance benchmarks
3. Documentation:
   - API docs for each module
   - Examples for each composition pattern
   - Migration guide

**Success criteria:**
- Logic module handles complex expressions
- All tools can compose successfully
- Documentation is comprehensive

**Deliverable:** Complete parser library with all modules.

---

### Phase 5: Tool Migration (Weeks 5-8)

**Goal:** Migrate existing tools to use modular parser.

**Week 5: RPT**
- Compose: filters + scopes + routes + flags
- Replace Clap with cnp-parse
- Test all existing commands
- Validate phase-based execution

**Week 6: XFD**
- Compose: filters + scopes + logic + routes + flags
- Replace existing parser
- Test complex queries
- Validate expression trees

**Week 7: SMV**
- Compose: actions + transforms + filters + routes + flags
- Replace Clap with cnp-parse
- Test all action types
- Validate filter integration

**Week 8: Remaining Tools**
- DSC, INX, EDT, MKR
- Compose appropriate modules
- Test and validate

**Success criteria:**
- All tools work with new parser
- No functionality regressions
- Binary sizes are smaller or comparable

---

## Migration Path

### Backward Compatibility Strategy

**Grammar specs remain valid:**
- Base Grammar v1.0 → Maps to base + routes + flags modules
- Query Grammar v1.0 → Maps to filters + scopes + logic modules
- Action Grammar v1.0 → Maps to actions + transforms modules

**Users don't see breaking changes:**
```bash
# These commands still work
rpt tree . TYPE:file EXT:rs
smv snake . EXT:md -r
xfd . (TYPE:file AND EXT:rs)

# Parser is internal implementation detail
```

### Migration Steps for Each Tool

**Step 1: Add cnp-parse dependency**
```toml
[dependencies]
cnp-parse = { version = "0.1.0", features = [...] }
```

**Step 2: Create parallel parser**
```rust
// Keep old parser working
mod old_parser { /* existing Clap code */ }

// Add new parser in parallel
mod new_parser {
    use cnp_parse::*;
    // New parsing logic
}

// Feature flag to switch
#[cfg(feature = "new-parser")]
use new_parser::parse;
#[cfg(not(feature = "new-parser"))]
use old_parser::parse;
```

**Step 3: Test equivalence**
```rust
#[cfg(test)]
mod migration_tests {
    #[test]
    fn old_and_new_equivalent() {
        let input = "tree . TYPE:file EXT:rs";
        let old = old_parser::parse(input);
        let new = new_parser::parse(input);
        assert_eq!(old, new);
    }
}
```

**Step 4: Enable by default**
```toml
[features]
default = ["new-parser"]
```

**Step 5: Remove old parser**
```rust
// Delete old_parser module
// Remove feature flag
```

### Rollback Plan

If migration reveals issues:

1. **Disable new parser** (flip feature flag back)
2. **Identify issue** (what command pattern broke?)
3. **Fix in cnp-parse** (add test case, fix module)
4. **Re-enable** once fix is validated

### User Communication

**Blog post / Release notes:**
```markdown
# RPT v0.2.0 - Internal Parser Modernization

## What's New

RPT now uses CNP's modular parser architecture. This is an internal
change that enables future features while maintaining compatibility.

## What's Changed

All existing commands work exactly as before. No breaking changes.

## What's Improved

- Smaller binary size (~10% reduction)
- Better error messages
- Foundation for upcoming features

## What's Next

Future releases will enable position-independent syntax, allowing
commands like these to work identically:

    rpt tree . TYPE:file EXT:rs
    rpt TYPE:file tree EXT:rs
    rpt EXT:rs TYPE:file tree

Stay tuned!
```

---

## Trade-off Analysis

### Benefits

**1. Modularity**
```
✅ Tools only include what they need
✅ Binary sizes reduced
✅ Compile times improved
✅ Clear module boundaries
```

**2. Maintainability**
```
✅ Change filters without affecting actions
✅ Test modules independently
✅ Clear ownership
✅ Easier onboarding (focused modules)
```

**3. Flexibility**
```
✅ Tools compose freely
✅ New tools don't need to fit Query/Action
✅ Grammar evolution is localized
✅ Experimentation is safer
```

**4. Correctness**
```
✅ Can't accidentally use wrong module
✅ Compile-time guarantees
✅ Explicit dependencies
```

### Costs

**1. Complexity**
```
❌ More modules to understand
❌ Composition requires thought
❌ Not obvious which modules to use
❌ More files to navigate
```

**2. Migration Effort**
```
❌ Every tool needs rework
❌ Testing burden increases
❌ Potential for bugs during migration
❌ 8+ weeks of work
```

**3. Documentation**
```
❌ Need to document each module
❌ Need to document compositions
❌ Examples for common patterns
❌ Migration guides
```

**4. Learning Curve**
```
❌ New contributors need to learn composition
❌ Not as familiar as Clap
❌ More decisions to make
```

### Comparison Matrix

| Aspect | Current (Clap + Hierarchical) | Proposed (Modular) |
|--------|-------------------------------|-------------------|
| **Binary size** | Full Clap (~200KB) + full grammar | Only needed modules (~20-30KB) |
| **Compile time** | Clap derive macros (slow) | Direct parsing (faster) |
| **Flexibility** | Inherit entire grammar | Compose needed modules |
| **Testing** | Integrated tests only | Module + integration tests |
| **Evolution** | Change affects all tools | Change affects subset |
| **Learning curve** | Clap docs (good) | Custom docs (need to write) |
| **Type safety** | Clap validates at runtime | Features validate at compile time |
| **Position independence** | Hard to achieve | Natural fit |

### Risk Assessment

**Low Risk:**
- Base tokenization (straightforward, testable)
- Filter module (well-defined, used everywhere)
- Routes module (stable, universal)

**Medium Risk:**
- Logic module (expression trees are complex)
- Transform module (multi-token parsing)
- Migration testing (need comprehensive coverage)

**High Risk:**
- Flag semantics (overlap between Query/Action)
- Module composition guidelines (could be confusing)
- Breaking changes during migration (hidden incompatibilities)

---

## Decision Framework

### Questions to Answer

**1. Commitment**

Are you willing to:
- Invest 8+ weeks in migration?
- Rework all existing tools?
- Write comprehensive documentation?
- Support both parsers during transition?

**2. Philosophy**

Do you believe:
- Modularity is worth complexity?
- Composition is better than inheritance?
- Tool-specific optimization matters?
- CNP should own its parsing?

**3. Future**

Do you expect:
- Many more tools in the ecosystem?
- Grammar to keep evolving?
- Tools that don't fit Query/Action?
- Need for position-independent parsing?

**4. Risk Tolerance**

Are you comfortable with:
- Custom parser (no Clap safety net)?
- Migration risk (potential bugs)?
- Maintenance burden (ongoing support)?
- Learning curve for contributors?

### Decision Matrix

| Answer Pattern | Recommendation |
|----------------|----------------|
| **Mostly YES** | Proceed with modular architecture |
| **Mixed** | Pilot with RPT first, evaluate, then decide |
| **Mostly NO** | Keep hierarchical grammars, improve incrementally |

### Pilot Approach (Recommended)

Instead of committing fully, pilot the approach:

**Week 1-2:** Build base + filters + scopes modules
**Week 3:** Migrate RPT only
**Week 4:** Evaluate:
- Is parsing cleaner?
- Are errors better?
- Is testing easier?
- Is binary smaller?

**Decision point:** If pilot succeeds, continue. If not, revert RPT and reassess.

---

## Next Steps

### If Proceeding

1. **Review this document** with stakeholders
2. **Decide on pilot vs full migration**
3. **Create cnp-parse crate** with feature structure
4. **Write grammar module specs** (more detailed than this doc)
5. **Begin Phase 1 implementation** (base tokenization)

### If Not Proceeding

1. **Document decision rationale**
2. **Identify specific concerns**
3. **Propose alternative solutions** (improve Clap usage?)
4. **Revisit in 6 months** when more tools exist

### If Piloting

1. **Implement base + filters + scopes** (2 weeks)
2. **Migrate RPT** (1 week)
3. **Evaluate results** (1 week)
4. **Decide**: continue, iterate, or abandon

---

## Conclusion

The modular parser architecture represents a fundamental shift in how CNP tools are built. Instead of inheriting broad grammars, tools compose narrow modules based on their specific needs.

**Core trade-off:**
- **Complexity** (more modules, composition required)
- **For flexibility** (precise dependencies, independent evolution)

**This architecture makes sense if:**
- CNP ecosystem will have many tools
- Grammars will continue evolving
- Binary size and compile time matter
- Position-independent parsing is valuable

**This architecture may not be worth it if:**
- Only a few tools will ever exist
- Grammars are stable and settled
- Clap's limitations are acceptable
- Migration risk outweighs benefits

**Recommendation:** Pilot with RPT. Build base + filters + scopes, migrate one tool, evaluate results. Make full decision based on evidence, not speculation.

---

**Document Status:** PROPOSAL
**Next Review:** After implementation of Phase 1 (base tokenization)
**Decision Deadline:** 2025-12-01
**Contact:** CNP Development Team
