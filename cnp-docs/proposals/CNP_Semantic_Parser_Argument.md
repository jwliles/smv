# The Case for Semantic Parsing in CNP

**Purpose:** Argument document for choosing semantic over positional parsing  
**Audience:** CNP development team, stakeholders, future contributors  
**Date:** 2025-11-15

---

## Executive Summary

CNP should adopt semantic, position-independent parsing as the foundation for its CLI grammar. This document presents the argument for why semantic parsing is the right technical and strategic choice for CNP, despite requiring custom parser development instead of using established frameworks like Clap.

**Bottom Line:** The 3-4 week investment in building a semantic parser will pay for itself in improved UX, easier maintenance, and CNP's competitive positioning in the CLI tool ecosystem.

---

## The Problem: Why Positional Parsing Fails CNP

### 1. Cognitive Load is Too High

**With Positional Parsing (v1):**
```bash
rpt <scope> <path> <filters> <routes> <flags>
```

Users must remember:
- What is a scope vs a filter vs a route vs a flag?
- Which bucket does each keyword belong to?
- What order must buckets appear in?
- Which combinations are valid?

**Example of mental burden:**
```bash
# User wants: files, rust extension, as tree, in JSON
# Must translate to: scope path filter route
rpt tree . EXT:rs FORMAT:json
    ^    ^ ^       ^
    1    2 3       4
```

If they forget the order:
```bash
rpt . tree EXT:rs FORMAT:json  # ❌ Error: unexpected argument
rpt tree EXT:rs . FORMAT:json  # ❌ Error: path expected after scope
```

**Cognitive load increases with complexity:**
- Simple command: "I need to remember 4 buckets"
- Complex command: "I need to remember 4 buckets AND their sub-rules"
- Every new feature: "Which bucket does this go in?"

### 2. Execution Order is Inflexible

**The Type Filter Problem:**

```bash
rpt tree /large/directory -f EXT:rs SIZE:>1MB
```

What actually happens:
1. Traverse `/large/directory` (expensive, processes directories)
2. Apply `EXT:rs` filter
3. Apply `SIZE:>1MB` filter (delegates to INX)
4. Apply `-f` flag (discard all directories)
   
**Step 4 makes steps 1-3 wasteful** because we processed directories we're throwing away.

**What SHOULD happen:**
1. Apply `-f` (files only) as a constraint
2. Traverse `/large/directory` (skip directories entirely)
3. Apply `EXT:rs` during traversal
4. Apply `SIZE:>1MB` via INX query

Positional parsing says "flags come last" but semantics say "type constraints come first."

**This is a fundamental mismatch** between syntax order and logical execution order.

### 3. Grammar Evolution is Constrained

Adding new features requires finding positions:

**Positional:**
```
Current: <tool> <scope> <path> <filters> <routes> <flags>

Want to add aggregation modifiers (GROUP:, SORT:)
Where do they go?

Option A: New bucket after filters
  <tool> <scope> <path> <filters> <aggregations> <routes> <flags>
  Problem: 6 buckets now, users must relearn

Option B: Extend filters bucket
  <tool> <scope> <path> <filters+aggs> <routes> <flags>
  Problem: "filters" bucket name is wrong, semantic confusion

Option C: Special syntax in routes
  <tool> <scope> <path> <filters> <routes+aggs> <flags>
  Problem: routes aren't aggregations, logical mismatch
```

**Every new feature creates bucket debate.**

**Semantic:**
```
Current keywords: tree, TYPE:, EXT:, FORMAT:, -f

Want to add aggregation: GROUP:, SORT:, LIMIT:

Just add them to the keyword list:
  GROUP:ext, SORT:size, LIMIT:10

No bucket debate. Users just learn new keywords.
```

### 4. Tool Delegation is Awkward

CNP's delegation protocol:

```bash
# SIZE: belongs to INX, not RPT
rpt tree . SIZE:>1MB EXT:rs
```

With positional parsing:
```rust
// RPT must:
1. Parse full command positionally
2. Recognize SIZE: isn't ours
3. Serialize it back to string
4. Call INX with serialized command
5. Parse INX results
6. Continue with remaining filters

// Each tool reinvents this logic
```

With semantic parsing:
```rust
// RPT can:
1. Parse command semantically
2. PhaseResolver says "SIZE: → IndexQuery phase"
3. IndexQuery phase knows to call INX
4. INX returns results as data structure
5. Continue with next phase

// Delegation is built into the execution model
```

Positional parsing makes delegation feel like a hack. Semantic parsing makes it natural.

---

## The Solution: Why Semantic Parsing Wins

### 1. Eliminates Cognitive Load

**Semantic Model:**
```bash
rpt <keyword> <keyword> <keyword> ...
```

Users only need to know:
- What keywords exist (NAME:, EXT:, TYPE:, FORMAT:, etc.)
- What values those keywords take
- How keywords combine

They DON'T need to know:
- ❌ Bucket positions
- ❌ Ordering rules
- ❌ Positional syntax

**Mental model simplifies:**
```
Before: "scope then path then filters then routes then flags"
After:  "keywords that describe what I want"
```

**Example:**
```bash
# All equivalent - any order works
rpt tree . TYPE:file EXT:rs
rpt tree TYPE:file . EXT:rs
rpt TYPE:file tree EXT:rs .
rpt EXT:rs TYPE:file tree .
```

User thinks: "I want a tree of files with rust extension"
User types: keywords in whatever order feels natural
Parser figures out: how to execute efficiently

### 2. Execution Optimizes Semantically

**Phase-based execution:**

```bash
Input: rpt tree . TYPE:file SIZE:>1MB EXT:rs

Phases (automatically determined):
1. PathResolution(.)
2. TypeConstraints(File)        ← Applied during traversal
3. IndexQuery([SIZE:>1MB])      ← Delegate to INX first
4. Traversal                     ← Only walk files that match INX results
5. Filtering(EXT:rs)             ← Apply during walk

Result: Optimal execution regardless of input order
```

**Benefits:**
- Early filtering reduces work (TYPE:file eliminates directories immediately)
- Index queries run before filesystem traversal (SIZE:>1MB uses INX)
- Traversal filters apply inline (EXT:rs during walk, not after)

**User doesn't think about optimization.** Parser does it automatically.

### 3. Grammar Extends Naturally

Adding features is just adding keywords:

**Current:**
```
Keywords: tree, sys, disk, TYPE:, EXT:, SIZE:, FORMAT:, TO:, INTO:
```

**Add aggregation:**
```
Keywords: tree, sys, disk, TYPE:, EXT:, SIZE:, FORMAT:, TO:, INTO:,
          GROUP:, SORT:, LIMIT:
```

No buckets to rearrange. Just new keywords.

**Add time filters:**
```
Keywords: tree, sys, disk, TYPE:, EXT:, SIZE:, FORMAT:, TO:, INTO:,
          GROUP:, SORT:, LIMIT:, MODIFIED:, CREATED:, ACCESSED:
```

**Users learn incrementally:**
- Week 1: Basic commands (tree, TYPE:, EXT:)
- Week 2: Discover FORMAT: and TO:
- Week 3: Learn about GROUP: and SORT:
- Week 4: Find MODIFIED: for time filtering

Each new keyword adds capability without changing syntax model.

### 4. Delegation is First-Class

**Semantic model makes delegation native:**

```rust
// Keyword ownership is explicit
enum KeywordOwner {
    Rpt,          // tree, sys, disk
    Inx,          // SIZE:, MODIFIED:, HASH:
    Cmp,          // DUPLICATE:, DIFF:
    Shared,       // TYPE:, EXT:, NAME: (handled locally)
}

// Phase resolver routes automatically
fn resolve_keyword(kw: Keyword) -> ExecutionPhase {
    match keyword_owner(kw) {
        KeywordOwner::Inx => Phase::IndexQuery(kw),
        KeywordOwner::Cmp => Phase::Delegation(Tool::Cmp, kw),
        KeywordOwner::Rpt => Phase::LocalFilter(kw),
        KeywordOwner::Shared => Phase::Filtering(kw),
    }
}
```

**Transparent to users:**
```bash
rpt tree . SIZE:>1MB      # SIZE: automatically delegates to INX
rpt tree . DUPLICATE:     # DUPLICATE: automatically delegates to CMP
```

**Logged for visibility:**
```
[CNP] ℹ️  Delegating SIZE: to INX
[CNP] 🔍 Querying index for files >1MB
[CNP] ✓  Found 42 matching files
```

Users get the benefits of delegation without managing it.

---

## Common Objections Addressed

### "Why not just use Clap? It's battle-tested."

**Response:** Clap is excellent for positional parsing, but we don't want positional parsing.

Clap's model:
```rust
.arg(Arg::new("path").index(1))
.arg(Arg::new("filter").index(2))
.arg(Arg::new("flag").short('f'))
```

This REQUIRES positions. You can't make Clap position-independent.

**We could hack around it:**
```rust
let args = env::args().collect();
let preprocessed = rearrange_args_to_positions(&args)?;
let matches = cli().get_matches_from(preprocessed)?;
```

But now we're building a parser anyway (the preprocessor), and we still have positional constraints.

**Better to build the parser we actually want.**

### "Custom parsers are hard to maintain."

**Response:** True, but the semantic model is actually simpler than the positional workarounds.

**Positional with workarounds:**
```rust
// Per-tool custom logic
fn handle_flags_early(args: &[String]) -> (TypeFilter, Vec<String>) {
    // Extract -f/-d before Clap sees them
}

fn validate_bucket_ordering(matches: &ArgMatches) -> Result<()> {
    // Ensure filters came after path
}

fn delegate_to_inx(size_filter: &str) -> Result<Vec<Entry>> {
    // Serialize, call INX, parse results
}

// Repeated across every tool
```

**Semantic with parser:**
```rust
// Shared across all tools
let phases = cnp_parse::parse_and_resolve(input)?;
executor.execute(phases)?;

// Tool-specific execution logic, not parsing logic
```

**Maintenance burden:**
- Positional: N tools × parsing workarounds = N codebases to maintain
- Semantic: 1 parser + N tool-specific executors = 1 parser codebase + cleaner tool code

### "What about performance?"

**Response:** Parsing is negligible compared to filesystem operations.

**Benchmark (estimated):**
- Clap parsing: ~0.1ms
- Semantic parsing: ~0.5ms
- Filesystem traversal: 100-1000ms

Parsing overhead: 0.4ms added to a 500ms operation = 0.08% slowdown

**And we gain execution optimization:**
- Early type filtering: saves 50% of traversal time (skip directories)
- Index queries: saves 90% of filesystem ops (query index instead)
- Efficient delegation: saves round-trips to tools

**Net performance: semantic is faster** because execution optimizes better.

### "Users won't understand semantic parsing."

**Response:** Users don't need to understand it - they just use keywords.

**User doesn't think:**
"This is semantic position-independent phase-based parsing with delegation"

**User thinks:**
"I type keywords that describe what I want"

**Examples they'll see:**
```bash
rpt tree . TYPE:file         # "Show me files in a tree"
rpt tree TYPE:file           # "Same thing, path defaults to ."
rpt TYPE:file tree           # "Same thing, order doesn't matter"
```

The third example works "magically" and they don't need to know why.

**Analogy:**
- SQL: Nobody thinks "position-independent declarative query language"
- SQL: People think "SELECT what I want, WHERE conditions apply"
- CNP: Same principle - keywords describe intent

### "What if we need to revert?"

**Response:** Reversion plan exists, but unlikely to be needed.

**Phase 1: Pilot in RPT**
- If semantic fails, only RPT affected
- Can revert RPT to Clap
- Other tools unaffected

**Phase 2: Prove the Model**
- User feedback during pilot
- If negative, stop migration
- Keep RPT semantic (it needs it), others stay positional

**Phase 3: Gradual Rollout**
- Only migrate tools that benefit
- No forced migration
- Tools can coexist

**Why reversion is unlikely:**
- Early feedback during development
- Pilot phase validates UX
- Gradual rollout minimizes risk
- Parser is scoped and testable

---

## Strategic Benefits

### 1. Differentiation in CLI Ecosystem

**CNP's Competition:**
- `fd` - faster find
- `ripgrep` - faster grep  
- `bat` - better cat
- `exa` - better ls

**Their pitch:** "We're faster/better than traditional tools"

**CNP's pitch with semantic:** "We rethought how CLI tools should work"

**Semantic parsing becomes identity:**
- "The CLI toolkit with natural command syntax"
- "Position-independent, intent-driven tools"
- "Think about what you want, not where to type it"

This is a **stronger differentiator** than "we're faster."

### 2. Documentation Becomes Simpler

**Positional docs (current):**
```
USAGE:
  rpt <SCOPE> [PATH] [FILTERS...] [ROUTES...] [FLAGS...]

ARGUMENTS:
  <SCOPE>      Report scope (tree, sys, disk, net)
  [PATH]       Target path (default: .)

FILTERS:
  Must come after PATH, before ROUTES
  
  TYPE:<type>  Filter by type (file, dir)
  EXT:<ext>    Filter by extension
  SIZE:<op><n> Filter by size (>, <, ==)

ROUTES:
  Must come after FILTERS, before FLAGS
  
  FORMAT:<fmt> Output format (json, yaml, table)
  TO:<tool>    Delegate to another tool
  INTO:<file>  Write to file

FLAGS:
  Must come last
  
  -f           Files only (alias for TYPE:file)
  -d           Directories only (alias for TYPE:dir)
  -r           Recursive

EXAMPLES:
  rpt tree . TYPE:file EXT:rs
       ^   ^ ^         ^
     scope path filter filter
```

**Semantic docs (proposed):**
```
USAGE:
  rpt <keywords in any order>

KEYWORDS:
  Scopes:
    tree       Show tree view
    sys        System information
    disk       Disk usage
    net        Network information

  Filters:
    TYPE:<type>  Filter by type (file, dir)
    EXT:<ext>    Filter by extension  
    SIZE:<op><n> Filter by size (>, <, ==)

  Routes:
    FORMAT:<fmt> Output format (json, yaml, table)
    TO:<tool>    Delegate to another tool
    INTO:<file>  Write to file

  Flags:
    -f           Files only (alias for TYPE:file)
    -d           Directories only (alias for TYPE:dir)
    -r           Recursive

EXAMPLES:
  rpt tree TYPE:file EXT:rs
  rpt TYPE:file tree EXT:rs     (same result)
  rpt EXT:rs TYPE:file tree     (also same)
```

**Simpler explanation** = easier onboarding = more users.

### 3. Grammar Consistency Across Ecosystem

All CNP tools share the same semantic model:

```bash
# Query tools
xfd . TYPE:file EXT:rs SIZE:>1MB
dsc . TYPE:dir MODIFIED:>7d

# Reporting tools
rpt tree TYPE:file GROUP:ext SORT:size

# Action tools (when migrated)
smv mv . TYPE:file EXT:tmp TO:archive/
```

**Benefits:**
- Learn once, use everywhere
- Tools compose naturally
- Delegation is transparent
- Grammar extends uniformly

**Contrast with positional:**
```bash
# Different tools, different position rules
xfd ./src EXT:rs -p              # path, filter, flag
smv snake ./docs EXT:md -pr       # action, path, filter, flag
rpt tree . GROUP:ext -r          # scope, path, modifier, flag
```

Users must remember per-tool ordering.

### 4. Future-Proofing

**Semantic parsing enables:**

1. **Interactive builders**
   ```
   $ rpt --interactive
   
   Select scope: [tree] sys disk net
   Select filters: [TYPE:file] EXT: SIZE: MODIFIED:
   Selected: TYPE:file
   Add more filters: [EXT:] SIZE: MODIFIED: (done)
   Selected: EXT:rs
   ...
   
   Generated command: rpt tree TYPE:file EXT:rs
   ```

2. **Command suggestions**
   ```
   $ rpt tree TYPE:file
   
   Hint: Add EXT: to filter by extension
   Hint: Add SIZE: to filter by size
   Hint: Add FORMAT:json to export as JSON
   ```

3. **Template expansion**
   ```
   $ cat .cnp-aliases
   large-files = TYPE:file SIZE:>100MB
   
   $ rpt tree @large-files EXT:log
   # Expands to: rpt tree TYPE:file SIZE:>100MB EXT:log
   ```

4. **Natural language hints**
   ```
   $ rpt "show me large rust files"
   
   Interpreted as: rpt tree TYPE:file EXT:rs SIZE:>1MB
   Execute? [y/n]
   ```

**None of these are possible** with rigid positional parsing.

---

## Implementation Confidence

### We Have Prior Art

**Successful semantic CLI systems:**
- **SQL** - Position-independent clauses (SELECT ... WHERE ... ORDER BY)
- **GraphQL** - Flexible query composition
- **kubectl** - Position-independent flags
- **jq** - Flexible filter chaining

**None of these use Clap-style positional parsing.** They all benefit from semantic models.

### We Have the Grammar Specs

**Already documented:**
- CNP_Base_Grammar.md - Universal foundation
- CNP_Query_Grammar.md - Query syntax
- CNP_Query_Grammar_v2_DRAFT.md - Semantic proposal
- CNP_Keyword_Routing.md - Delegation protocol

**The specs ARE the parser design.** We're implementing, not inventing.

### We Have Scoped Risk

**Phase 1: Build parser (3-4 weeks)**
- Isolated codebase
- Comprehensive tests
- No user impact yet

**Phase 2: Pilot in RPT (1-2 weeks)**
- Single tool affected
- Internal testing first
- Can revert if needed

**Phase 3: Gather feedback (2-3 weeks)**
- Real user validation
- Measure success criteria
- Decide on expansion

**Total: 6-9 weeks** from start to validated pilot.

**Worst case:** 9 weeks spent, revert to Clap, lessons learned  
**Best case:** 9 weeks spent, semantic validated, ecosystem benefits

### We Have Clear Success Criteria

**Must achieve:**
- ✅ Position-independent parsing works
- ✅ Performance within 10% of Clap
- ✅ Error messages are helpful
- ✅ 90%+ test coverage
- ✅ RPT features work correctly

**Nice to have:**
- ⭐ Users prefer semantic syntax
- ⭐ Documentation is clearer
- ⭐ New features easier to add

**Evaluation:**
- 8 weeks: Internal review
- 10 weeks: User feedback
- 12 weeks: Go/no-go decision

---

## Recommendation

**Build the semantic parser.** The investment is justified by:

### UX Benefits
- Lower cognitive load (no bucket memorization)
- Natural command composition
- Better error messages
- Easier learning curve

### Technical Benefits
- Execution optimization (phase-based)
- Clean delegation model
- Easier grammar extension
- Better code organization

### Strategic Benefits
- Market differentiation
- Simpler documentation
- Ecosystem consistency
- Future-proofing

### Risk Mitigation
- Scoped implementation (RPT pilot)
- Clear success criteria
- Reversion plan exists
- Gradual rollout

**The time to build this is now:**
- Tools are pre-1.0 (breaking changes acceptable)
- No user lock-in yet
- Grammar specs are ready
- Team has bandwidth

**Next steps:**
1. Approve this argument
2. Review design document
3. Begin Phase 1 implementation
4. Regular progress check-ins

---

## Conclusion

Semantic parsing isn't just a "nice to have" - it's a **strategic investment** in CNP's future. The 6-9 week implementation pays for itself through:

- Happier users (easier to use)
- Cleaner codebase (better execution model)
- Stronger positioning (differentiated UX)
- Future capabilities (interactive builders, suggestions, templates)

**The question isn't "should we build this?"**  
**The question is "can we afford NOT to?"**

And the answer is no - positional parsing will keep limiting us, and the longer we wait, the harder migration becomes.

**Let's build it.**

---

**Prepared by:** CNP Development Team  
**Date:** 2025-11-15  
**Status:** Proposal for approval  
**Next Review:** After Phase 1 completion
