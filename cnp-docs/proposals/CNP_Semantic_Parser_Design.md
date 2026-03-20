# CNP Semantic Parser Design Document

**Version:** 1.0  
**Date:** 2025-11-15  
**Status:** Design Proposal  
**Author:** CNP Development Team

---

## Executive Summary

This document proposes a semantic, position-independent parsing system for the CNP (Canopy) CLI ecosystem as an alternative to traditional positional parsing frameworks like Clap. The semantic parser enables users to express commands in natural order without memorizing positional buckets, while providing phase-based execution that optimizes performance and enables advanced features like transparent tool delegation.

**Key Decision:** Build `cnp-parse` as a lightweight, reusable crate that provides semantic parsing primitives for CNP tools, starting with `rpt` as the pilot implementation.

---

## Table of Contents

1. [Background and Motivation](#background-and-motivation)
2. [Problem Statement](#problem-statement)
3. [Semantic vs Positional Parsing](#semantic-vs-positional-parsing)
4. [Design Goals](#design-goals)
5. [Architecture Overview](#architecture-overview)
6. [Implementation Strategy](#implementation-strategy)
7. [Migration Path](#migration-path)
8. [Risk Analysis](#risk-analysis)
9. [Success Criteria](#success-criteria)
10. [References and Prior Art](#references-and-prior-art)

---

## Background and Motivation

### The CNP Vision

CNP is a modular CLI ecosystem designed to replace and extend traditional Unix tooling with:
- Structured, declarative grammar across all tools
- Composable workflows via file formats (`.cnp`, `.cnd`)
- Transparent delegation between tools
- Index-backed architecture for performance

### Current State

CNP tools currently use positional parsing (via Clap) with the v1 grammar structure:

```bash
<tool> <path> <filters> <routes> <flags>
```

This creates artificial constraints that don't match the problem domain and increase cognitive load for users.

### Triggering Event

During implementation of `rpt tree`, adding `-f` (files-only) and `-d` (dirs-only) filters exposed a fundamental limitation:

```bash
# Type filters need to execute DURING traversal, not after
rpt tree /large/directory EXT:rs SIZE:>1MB -f

# But v1 grammar says flags come last
# This means wasting CPU/time on directories we'll discard
```

The `-f`/`-d` flags need to be **semantic type constraints** that affect execution phase, not post-processing flags. This revealed that positional parsing creates execution order problems.

---

## Problem Statement

### Core Issues with Positional Parsing

1. **Cognitive Overhead**
   - Users must remember bucket positions: `<path> <filters> <routes> <flags>`
   - Complex commands require mental translation from intent to syntax
   - Error messages reference positions rather than semantics

2. **Execution Constraints**
   - Processing order tied to input order
   - Cannot optimize execution based on keyword semantics
   - Flags can't influence early execution phases

3. **Extensibility Limits**
   - Adding new keyword types requires new positional slots
   - Grammar evolution breaks existing mental models
   - Tool-specific extensions create inconsistency

4. **Delegation Complexity**
   - Positional parsing makes keyword routing awkward
   - Tools must parse, re-serialize, and delegate
   - Error handling across tool boundaries is complex

### Concrete Example: The Type Filter Problem

```bash
# User intent: "Show me Rust files in a tree"
# Current positional requirement:
rpt tree . TYPE:file EXT:rs FORMAT:tree
       ^   ^         ^       ^
     scope path    filter   route

# Problem: TYPE:file should execute during traversal, but it's in filter position
# Solution with semantic: keywords execute in dependency order, not input order
rpt tree TYPE:file EXT:rs  # Works regardless of order
```

---

## Semantic vs Positional Parsing

### Positional Parsing (Clap-style)

**Model:**
```
command := tool position1 position2 ... positionN [flags]
```

**Mental Model:** "What goes where?"

**Example:**
```bash
xfd ./src EXT:rs SIZE:>1MB -p
    ^     ^      ^         ^
  path  filter  filter   flag
```

**Characteristics:**
- ✅ Well-understood paradigm
- ✅ Framework support (Clap, structopt)
- ✅ Familiar to Unix users
- ❌ Bucket memorization required
- ❌ Execution order = input order
- ❌ Extensions require new positions

### Semantic Parsing (Proposed)

**Model:**
```
command := tool keyword1 keyword2 ... keywordN
execution := resolve_phases(keywords)
```

**Mental Model:** "What do I want?"

**Example:**
```bash
rpt tree TYPE:file EXT:rs    # Any order works
rpt TYPE:file tree EXT:rs    # Same result
rpt EXT:rs TYPE:file tree    # Also same
```

**Characteristics:**
- ✅ Position-independent
- ✅ Lower cognitive load
- ✅ Execution optimized by semantics
- ✅ Natural command composition
- ❌ Requires custom parser
- ❌ Less familiar paradigm

---

## Design Goals

### Primary Goals

1. **Position Independence**
   - Keywords work in any order
   - No bucket memorization required
   - Execution order determined by semantics, not syntax

2. **Phase-Based Execution**
   - Keywords resolve to execution phases
   - Phases execute in dependency order
   - Early phases can optimize later phases

3. **Grammar Consistency**
   - Shared keyword definitions across tools
   - Consistent delegation protocol
   - Uniform error handling

4. **Performance Optimization**
   - Type constraints applied during traversal
   - Delegation to specialized tools (INX, CMP)
   - Query planning for complex filters

5. **Developer Experience**
   - Simple to add new keywords
   - Clear extension points
   - Reusable across tools

### Non-Goals

1. **Natural Language Processing**
   - Not trying to parse free-form text
   - Keywords remain structured (UPPERCASE, operators)
   - No ML or fuzzy matching

2. **Full Tool Migration**
   - Not requiring all tools migrate immediately
   - Positional and semantic can coexist
   - Migration on tool-by-tool basis

3. **Perfect Backward Compatibility**
   - V2 grammar may break v1 commands
   - Tools still in development (pre-1.0)
   - Users expect evolution

---

## Architecture Overview

### High-Level Structure

```
┌─────────────────────────────────────────────────┐
│                  User Input                      │
│   "rpt tree . TYPE:file EXT:rs FORMAT:json"     │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│                   Lexer                          │
│  Tokenize: [tree, ., TYPE:file, EXT:rs, ...]    │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│                   Parser                         │
│  Build AST: Command { scope, filters, routes }  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│              Semantic Resolver                   │
│  Map keywords to execution phases                │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│              Phase Executor                      │
│  Execute phases in dependency order              │
└─────────────────────────────────────────────────┘
```

### Core Components

#### 1. Lexer

**Responsibility:** Convert input string to tokens

```rust
pub struct Token {
    kind: TokenKind,
    value: String,
    position: usize,
}

pub enum TokenKind {
    Keyword,      // UPPERCASE: NAME:, EXT:, TYPE:
    Operator,     // >, <, ==, !=
    Value,        // strings, numbers, paths
    Route,        // TO:, INTO:, FORMAT:
    Flag,         // -f, -d, -r, etc.
    Scope,        // tree, sys, disk, net
}

impl Lexer {
    pub fn tokenize(input: &str) -> Result<Vec<Token>> {
        // Tokenization logic
    }
}
```

**Key Features:**
- Recognize UPPERCASE keywords
- Identify operators and values
- Preserve original positions for error reporting
- Handle quoted strings and escape sequences

#### 2. Parser

**Responsibility:** Build Abstract Syntax Tree from tokens

```rust
pub struct Command {
    tool: String,
    scope: Option<Scope>,
    path: Option<PathBuf>,
    filters: Vec<Filter>,
    routes: Vec<Route>,
    flags: Vec<Flag>,
}

pub enum Filter {
    Type(TypeFilter),
    Extension(ExtFilter),
    Size(SizeFilter),
    Name(NameFilter),
    Modified(TimeFilter),
    Custom(CustomFilter),
}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Result<Command> {
        // Build AST from tokens
        // Position-independent - collect all keywords
        // Validate combinations
    }
}
```

**Key Features:**
- Position-independent keyword collection
- Syntax validation
- Conflict detection (e.g., `-f` and `-d` mutually exclusive)
- Helpful error messages

#### 3. Semantic Resolver

**Responsibility:** Map keywords to execution phases

```rust
pub enum Phase {
    PathResolution,     // Resolve . or ~/path
    TypeConstraints,    // Apply TYPE:file or -f/-d
    IndexQuery,         // Delegate SIZE:, MODIFIED: to INX
    Traversal,          // Walk filesystem with constraints
    Filtering,          // Apply EXT:, NAME: during traversal
    Aggregation,        // Apply GROUP:, SORT:, LIMIT:
    Routing,            // Execute TO:, INTO:, FORMAT:
}

pub struct PhaseResolver;

impl PhaseResolver {
    pub fn resolve(cmd: Command) -> Vec<ExecutionPhase> {
        // Map keywords to phases
        // Order phases by dependency
        // Optimize execution plan
    }
}
```

**Key Features:**
- Dependency-based phase ordering
- Query optimization (index-backed vs filesystem)
- Delegation routing (SIZE: → INX, HASH: → CMP)
- Smart defaults (path defaults to `.`)

#### 4. Phase Executor

**Responsibility:** Execute phases in order, maintaining state

```rust
pub struct PhaseExecutor {
    context: ExecutionContext,
    results: Vec<Entry>,
}

pub struct ExecutionContext {
    path: PathBuf,
    type_constraint: Option<TypeFilter>,
    index_results: Option<Vec<IndexEntry>>,
    // ... state shared across phases
}

impl PhaseExecutor {
    pub fn execute(&mut self, phases: Vec<ExecutionPhase>) -> Result<Output> {
        for phase in phases {
            self.execute_phase(phase)?;
        }
        Ok(self.finalize())
    }
    
    fn execute_phase(&mut self, phase: ExecutionPhase) -> Result<()> {
        match phase {
            Phase::TypeConstraints(filter) => {
                self.context.type_constraint = Some(filter);
            }
            Phase::IndexQuery(filters) => {
                self.context.index_results = self.query_index(filters)?;
            }
            Phase::Traversal => {
                self.results = self.traverse_with_constraints()?;
            }
            // ...
        }
        Ok(())
    }
}
```

**Key Features:**
- Stateful execution (phases share context)
- Early termination on errors
- Progress reporting for long operations
- Delegation to external tools

---

## Implementation Strategy

### Phase 1: Core Parser Library (3-4 weeks)

**Week 1: Lexer and Token Model**

```rust
// cnp-parse/src/lexer.rs
pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self { /* ... */ }
    pub fn tokenize(&mut self) -> Result<Vec<Token>> { /* ... */ }
    
    fn next_token(&mut self) -> Option<Token> { /* ... */ }
    fn is_keyword(&self, s: &str) -> bool { /* ... */ }
    fn is_operator(&self, s: &str) -> bool { /* ... */ }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tokenize_basic() {
        let tokens = Lexer::new("tree . TYPE:file".into()).tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Scope);
        assert_eq!(tokens[1].value, ".");
        assert_eq!(tokens[2].kind, TokenKind::Keyword);
    }
    
    #[test]
    fn test_tokenize_complex() {
        let tokens = Lexer::new("tree . TYPE:file EXT:rs SIZE:>1MB FORMAT:json".into())
            .tokenize()
            .unwrap();
        assert_eq!(tokens.len(), 6);
    }
}
```

**Week 2: Parser and AST**

```rust
// cnp-parse/src/parser.rs
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { /* ... */ }
    pub fn parse(&mut self) -> Result<Command> { /* ... */ }
    
    fn parse_filter(&mut self, token: Token) -> Result<Filter> { /* ... */ }
    fn parse_route(&mut self, token: Token) -> Result<Route> { /* ... */ }
    fn validate_command(&self, cmd: &Command) -> Result<()> { /* ... */ }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_position_independent() {
        let cmd1 = parse_str("tree . TYPE:file EXT:rs").unwrap();
        let cmd2 = parse_str("tree TYPE:file . EXT:rs").unwrap();
        let cmd3 = parse_str("TYPE:file tree EXT:rs .").unwrap();
        
        // All should resolve to same logical command
        assert_eq!(cmd1.filters, cmd2.filters);
        assert_eq!(cmd2.filters, cmd3.filters);
    }
    
    #[test]
    fn test_mutually_exclusive_flags() {
        let result = parse_str("tree -f -d");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mutually exclusive"));
    }
}
```

**Week 3: Semantic Resolver and Phase Model**

```rust
// cnp-parse/src/resolver.rs
pub struct PhaseResolver;

impl PhaseResolver {
    pub fn resolve(cmd: Command) -> Vec<ExecutionPhase> {
        let mut phases = vec![];
        
        // Always resolve path first
        phases.push(ExecutionPhase::PathResolution(cmd.path));
        
        // Type constraints before traversal
        if let Some(type_filter) = Self::extract_type_filter(&cmd) {
            phases.push(ExecutionPhase::TypeConstraints(type_filter));
        }
        
        // Index queries for metadata filters
        let index_filters = Self::extract_index_filters(&cmd);
        if !index_filters.is_empty() {
            phases.push(ExecutionPhase::IndexQuery(index_filters));
        }
        
        // Traversal with constraints
        phases.push(ExecutionPhase::Traversal);
        
        // Filtering during traversal
        let traversal_filters = Self::extract_traversal_filters(&cmd);
        if !traversal_filters.is_empty() {
            phases.push(ExecutionPhase::Filtering(traversal_filters));
        }
        
        // Aggregation
        if let Some(agg) = Self::extract_aggregation(&cmd) {
            phases.push(ExecutionPhase::Aggregation(agg));
        }
        
        // Routes last
        for route in cmd.routes {
            phases.push(ExecutionPhase::Routing(route));
        }
        
        phases
    }
    
    fn extract_type_filter(cmd: &Command) -> Option<TypeFilter> {
        // Check for TYPE: keyword or -f/-d flags
    }
    
    fn extract_index_filters(cmd: &Command) -> Vec<Filter> {
        // SIZE:, MODIFIED:, HASH:, etc. delegate to INX
    }
    
    fn extract_traversal_filters(cmd: &Command) -> Vec<Filter> {
        // NAME:, EXT:, etc. apply during traversal
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_phase_ordering() {
        let cmd = parse_str("tree . TYPE:file EXT:rs SIZE:>1MB").unwrap();
        let phases = PhaseResolver::resolve(cmd);
        
        // Verify correct ordering
        assert!(matches!(phases[0], ExecutionPhase::PathResolution(_)));
        assert!(matches!(phases[1], ExecutionPhase::TypeConstraints(_)));
        assert!(matches!(phases[2], ExecutionPhase::IndexQuery(_)));
        assert!(matches!(phases[3], ExecutionPhase::Traversal));
    }
}
```

**Week 4: Integration and Testing**

```rust
// cnp-parse/src/lib.rs
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod executor;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, Command, Filter, Route};
pub use resolver::{PhaseResolver, ExecutionPhase};
pub use executor::{PhaseExecutor, ExecutionContext};

// Convenience API
pub fn parse(input: &str) -> Result<Command> {
    let tokens = Lexer::new(input.to_string()).tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

pub fn parse_and_resolve(input: &str) -> Result<Vec<ExecutionPhase>> {
    let cmd = parse(input)?;
    Ok(PhaseResolver::resolve(cmd))
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_end_to_end() {
        let phases = parse_and_resolve("tree . TYPE:file EXT:rs SIZE:>1MB").unwrap();
        assert!(phases.len() >= 4);
    }
    
    #[test]
    fn test_position_independence() {
        let p1 = parse_and_resolve("tree . TYPE:file EXT:rs").unwrap();
        let p2 = parse_and_resolve("tree TYPE:file . EXT:rs").unwrap();
        let p3 = parse_and_resolve("TYPE:file tree EXT:rs .").unwrap();
        
        // All should produce equivalent phase sequences
        assert_eq!(p1.len(), p2.len());
        assert_eq!(p2.len(), p3.len());
    }
}
```

### Phase 2: RPT Integration (1-2 weeks)

**Week 5: Replace Clap in RPT**

```rust
// rpt/src/cli.rs
use cnp_parse::{parse_and_resolve, ExecutionPhase};

pub fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let input = args.join(" ");
    
    // Parse with semantic parser
    let phases = parse_and_resolve(&input)?;
    
    // Execute phases
    let mut executor = PhaseExecutor::new();
    executor.execute(phases)?;
    
    Ok(())
}
```

**Week 6: Testing and Refinement**

- Comprehensive test suite
- Error message quality
- Performance benchmarks
- Documentation

### Phase 3: Evaluation and Iteration (2-3 weeks)

**Week 7-8: Internal Testing**
- Dog-food the semantic parser
- Identify rough edges
- Refine error messages
- Optimize performance

**Week 9: User Feedback**
- Release rpt with semantic parser
- Gather feedback on UX
- Document common patterns
- Identify pain points

### Phase 4: Ecosystem Expansion (Ongoing)

- Evaluate other tools for semantic migration
- Standardize keyword definitions
- Build comprehensive grammar docs
- Create migration guides

---

## Migration Path

### Coexistence Strategy

**Semantic and positional can coexist:**

```bash
# Positional tools (v1 grammar)
xfd ./src EXT:rs SIZE:>1MB -p
smv snake . EXT:md -pr

# Semantic tools (v2 grammar)
rpt tree TYPE:file EXT:rs
rpt EXT:rs tree TYPE:file    # Same result
```

**Users learn one, benefit from both:**
- Users who learn positional style write semantic commands in positional order
- Semantic parser accepts positional order (it's a valid ordering)
- No cognitive dissonance until they try semantic order with positional tools

### Migration Triggers

**A tool should migrate to semantic when:**

1. **Execution order problems arise**
   - Like the `-f`/`-d` type filter issue in rpt
   - Flags need to affect early execution phases

2. **Complex command composition is needed**
   - Many filter combinations
   - Rich delegation protocol
   - Query optimization opportunities

3. **User feedback indicates confusion**
   - Frequent "wrong position" errors
   - Documentation requests about ordering
   - Feature requests for flexibility

**Tools can stay positional if:**

1. **Simple command structure**
   - Few arguments
   - Clear positional semantics
   - No execution order dependencies

2. **Well-established patterns**
   - Users comfortable with current syntax
   - No complaints about ordering
   - Minimal complexity

### Tool-by-Tool Assessment

| Tool | Migration Priority | Rationale |
|------|-------------------|-----------|
| `rpt` | **High** | Triggering tool; complex filtering; aggregation |
| `xfd` | **Medium** | Complex queries; delegation to INX; many filters |
| `dsc` | **Medium** | Simpler than xfd but similar patterns |
| `smv` | **Low** | Action grammar; clear positional semantics |
| `edt` | **Low** | Content editing; different mental model |
| `mkr` | **Low** | Template creation; simple arguments |
| `inx` | **Medium** | Backend tool; semantic queries make sense |
| `say` | **Medium** | Matching engine; flexible syntax desirable |

---

## Risk Analysis

### Technical Risks

**Risk: Parser Complexity**
- *Description:* Custom parser harder to maintain than framework
- *Likelihood:* Medium
- *Impact:* Medium
- *Mitigation:* 
  - Comprehensive test suite
  - Clear architecture documentation
  - Modular design for easy debugging
  - Reference implementations from established parsers

**Risk: Performance Overhead**
- *Description:* Semantic parsing slower than positional
- *Likelihood:* Low
- *Impact:* Low
- *Mitigation:*
  - Parsing is tiny fraction of total runtime
  - Execution optimization more important
  - Benchmark against Clap baseline
  - Optimize hot paths

**Risk: Edge Cases and Bugs**
- *Description:* Unforeseen parsing corner cases
- *Likelihood:* Medium
- *Impact:* Medium
- *Mitigation:*
  - Extensive test coverage
  - Fuzzing for edge cases
  - Clear error handling
  - Progressive rollout for feedback

### User Experience Risks

**Risk: Paradigm Confusion**
- *Description:* Users confused by semantic vs positional
- *Likelihood:* Low
- *Impact:* Medium
- *Mitigation:*
  - Semantic accepts positional ordering
  - Clear documentation
  - Examples in both styles
  - Helpful error messages

**Risk: Documentation Burden**
- *Description:* Must document both paradigms during transition
- *Likelihood:* High
- *Impact:* Low
- *Mitigation:*
  - Clear "v1 vs v2" sections in docs
  - Tool-specific notes on which model used
  - Migration guides
  - Unified keyword reference

**Risk: Breaking Changes**
- *Description:* v2 grammar breaks v1 commands
- *Likelihood:* Medium
- *Impact:* Low (pre-1.0)
- *Mitigation:*
  - Tools still pre-1.0, breaking changes expected
  - Clear versioning and release notes
  - Backward compatibility where possible
  - Deprecation warnings before removal

### Project Risks

**Risk: Scope Creep**
- *Description:* Parser becomes overly complex, delays delivery
- *Likelihood:* Medium
- *Impact:* High
- *Mitigation:*
  - Strict scope definition
  - MVP first, enhancements later
  - Regular progress reviews
  - Timebox implementation phases

**Risk: Tool Fragmentation**
- *Description:* Some tools semantic, some positional, inconsistent UX
- *Likelihood:* Medium
- *Impact:* Medium
- *Mitigation:*
  - Clear migration criteria
  - Grammar consistency even across paradigms
  - Shared keyword definitions
  - Regular ecosystem review

---

## Success Criteria

### Phase 1: Parser Implementation

**Must Have:**
- ✅ Lexer tokenizes all current keyword types
- ✅ Parser builds correct AST for rpt commands
- ✅ Semantic resolver produces correct phase ordering
- ✅ Position-independent parsing works (any keyword order)
- ✅ 90%+ test coverage
- ✅ Clear error messages with helpful suggestions

**Nice to Have:**
- ⭐ Performance within 10% of Clap parsing
- ⭐ Fuzzing test suite for edge cases
- ⭐ Detailed architecture documentation
- ⭐ Example parsers for other tools

### Phase 2: RPT Integration

**Must Have:**
- ✅ RPT works with semantic parser
- ✅ All existing rpt functionality preserved
- ✅ Type filters execute during traversal (not after)
- ✅ No regression in performance
- ✅ Documentation updated for v2 grammar

**Nice to Have:**
- ⭐ Improved error messages vs Clap version
- ⭐ New features enabled by semantic model
- ⭐ User testimonials on ease of use

### Phase 3: User Validation

**Must Have:**
- ✅ 80%+ user satisfaction with semantic syntax
- ✅ No increase in support questions
- ✅ Users successfully compose complex commands
- ✅ Error messages are actionable

**Nice to Have:**
- ⭐ Users discover new workflow patterns
- ⭐ Positive feedback on learning curve
- ⭐ Community contributions to parser

### Long-Term Success

**Must Have:**
- ✅ Parser is maintainable (clear code, good tests)
- ✅ New tools adopt semantic model
- ✅ Grammar evolution is smooth
- ✅ CNP ecosystem remains consistent

**Nice to Have:**
- ⭐ Parser becomes reference implementation for other projects
- ⭐ Community tools use cnp-parse
- ⭐ CNP recognized for superior CLI UX

---

## References and Prior Art

### Existing CLI Parsing Libraries

**Clap (Rust)**
- Positional, flag-based parsing
- Extensive feature set
- Well-documented, widely used
- Inspiration for error messages and help text

**Cobra (Go)**
- Positional command/subcommand model
- Flag parsing
- Used by Kubernetes, Hugo, Docker
- Good hierarchical command structure

**Click (Python)**
- Decorator-based CLI building
- Positional and keyword arguments
- Excellent documentation generation
- Inspiration for composability

**Yargs (JavaScript)**
- Chainable configuration
- Positional and option parsing
- Good middleware model
- Inspiration for command building

### Semantic/Natural Language CLI

**SQL**
- Position-independent clauses (SELECT, WHERE, ORDER BY can be rearranged in some dialects)
- Keyword-driven syntax
- Declarative model
- Inspiration for keyword-based approach

**Kubernetes kubectl**
- Verb-noun model with flags
- Position-independent flags
- Extensive filtering syntax
- Inspiration for filter composition

**jq**
- Functional pipeline syntax
- Position matters but composition is flexible
- Filter chaining
- Inspiration for filter design

### Query Languages

**GraphQL**
- Declarative queries
- Field selection syntax
- Flexible argument ordering
- Inspiration for filter/field selection

**MongoDB Query Language**
- JSON-based query objects
- Operator-driven filtering
- Composable filters
- Inspiration for filter operators

### Academic References

**Parser Design:**
- "Crafting Interpreters" by Robert Nystrom
- "Modern Compiler Implementation" by Andrew Appel
- "Parsing Techniques" by Dick Grune and Ceriel Jacobs

**CLI Design:**
- "The Art of Unix Programming" by Eric Raymond
- "Command Line Interface Guidelines" (CLI Guidelines Working Group)
- "The Unix Philosophy" by Mike Gancarz

### CNP-Specific Context

**Internal Documents:**
- CNP_Base_Grammar.md - Universal grammar foundation
- CNP_Query_Grammar.md - Declarative query syntax
- CNP_Query_Grammar_v2_DRAFT.md - Semantic proposal
- CNP_Keyword_Routing.md - Delegation protocol

---

## Appendices

### Appendix A: Grammar Comparison

**V1 Positional:**
```
rpt <scope> <path> <filters> <routes> <flags>
    ^       ^      ^         ^        ^
    bucket1 bucket2 bucket3  bucket4  bucket5

Required order, mental buckets
```

**V2 Semantic:**
```
rpt <keyword> <keyword> <keyword> ...
    ^         ^         ^
    any order, resolved by semantics

No buckets, just keywords
```

### Appendix B: Execution Phase Examples

**Example 1: Simple Query**
```bash
Input: rpt tree . TYPE:file EXT:rs

Phases:
1. PathResolution(.)
2. TypeConstraints(File)
3. Traversal
4. Filtering(EXT:rs)
```

**Example 2: Complex Query with Delegation**
```bash
Input: rpt tree . TYPE:file SIZE:>1MB EXT:rs FORMAT:json

Phases:
1. PathResolution(.)
2. TypeConstraints(File)
3. IndexQuery([SIZE:>1MB])      ← Delegate to INX
4. Traversal                     ← Use INX results
5. Filtering(EXT:rs)
6. Routing(FORMAT:json)
```

**Example 3: Aggregation**
```bash
Input: rpt tree . GROUP:ext SORT:size LIMIT:10

Phases:
1. PathResolution(.)
2. Traversal
3. Aggregation(GROUP:ext)
4. Aggregation(SORT:size)
5. Aggregation(LIMIT:10)
```

### Appendix C: Error Message Examples

**Position Error (Current with Clap):**
```
Error: unexpected argument 'EXT:rs' found

Usage: rpt tree <PATH> [FILTERS...] [FLAGS...]

For more information, try '--help'.
```

**Semantic Error (Proposed):**
```
Error: Unknown keyword 'ETX:rs'
  
  Did you mean 'EXT:rs'?

  Valid extension filter syntax:
    EXT:rs
    EXT:md,txt
    EXT:*

For more information, try '--help'.
```

**Conflict Error (Semantic):**
```
Error: Conflicting type filters

  Cannot use both -f (files only) and -d (directories only)

  Choose one:
    rpt tree . -f    # Files only
    rpt tree . -d    # Directories only

For more information, try '--help'.
```

---

## Conclusion

The semantic parser represents a fundamental shift in how CNP tools are invoked, moving from positional bucket memorization to intent-driven keyword composition. This shift:

1. **Reduces cognitive load** - users think about what they want, not where to put it
2. **Enables optimization** - execution phases ordered by dependencies, not input
3. **Improves extensibility** - new keywords don't require new positions
4. **Maintains consistency** - shared grammar across growing tool ecosystem

The investment (3-4 weeks initial implementation) is justified by long-term benefits to UX, maintainability, and CNP's identity as a modern CLI ecosystem.

**Next Steps:**
1. Review and approve this design
2. Begin Phase 1 implementation (cnp-parse core)
3. Integrate with rpt as pilot
4. Evaluate results and iterate

---

**Document Status:** Approved for implementation  
**Implementation Start:** [Date TBD]  
**Review Date:** [3 months after pilot launch]
