# CNP Base Grammar Specification v1.0

**Purpose**: Foundational grammar specification that Query and Action grammars extend
**Scope**: Universal standards for all CNP ecosystem tools  
**Status**: Authoritative - supersedes duplicated information in other grammar specs

---

## 🎯 Design Principles

1. **Grammar Consistency**: All CNP tools share common base elements
2. **Interface Compatibility**: Grammars must interoperate cleanly
3. **No Conflicts**: Reserved elements cannot be redefined by extensions
4. **Delegation Protocol**: Standard mechanism for cross-tool communication

---

## 🧱 Universal Command Structure

```
<tool> [base-flags] [grammar-specific-syntax] [universal-routes]
```

### Base Elements (All Tools)

| Element | Type | Description | Required |
|---------|------|-------------|----------|
| `tool` | Command | CNP tool name | ✅ Yes |
| `base-flags` | Flags | Universal flags (see below) | Optional |
| `universal-routes` | Routes | Cross-tool directives | Optional |

---

## 🚩 Universal Base Flags

These flags MUST be consistent across ALL CNP tools:

| Flag | Long Form | Description | Grammar |
|------|-----------|-------------|---------|
| `-h` | `--help` | Show help information | Universal |
| `-v` | `--version` | Show version information | Universal |

### Reserved Flag Characters

| Character | Status | Reserved For |
|-----------|--------|--------------|
| `h` | ✅ Reserved | Help (universal) |
| `v` | ✅ Reserved | Version (universal) |
| `H` | 🔒 Reserved | Future CNP use |
| `V` | 🔒 Reserved | Future CNP use |

---

## 📤 Universal Route Keywords

Cross-tool communication and output routing:

| Route | Syntax | Description | Example |
|-------|--------|-------------|---------|
| `TO:` | `TO:tool` | Delegate to another CNP tool | `TO:smv` |
| `INTO:` | `INTO:file` | Write output to file | `INTO:results.txt` |
| `FORMAT:` | `FORMAT:type` | Output format specification | `FORMAT:json` |

### Route Processing Order
1. Tool executes its primary function
2. Apply `FORMAT:` transformation
3. Execute `TO:` delegation
4. Write to `INTO:` destination

---

## 🔍 Universal Filter Keywords

Reserved across ALL grammars (Query tools use natively, Action tools via delegation):

| Filter | Syntax | Meaning | Example |
|--------|--------|---------|---------|
| `NAME:` | `NAME:pattern` | File/directory name matching | `NAME:config` |
| `EXT:` | `EXT:extension` | File extension filter | `EXT:rs` |
| `TYPE:` | `TYPE:type` | File type (`file`, `dir`, `symlink`) | `TYPE:file` |
| `MORE:` | `MORE:value` | File size greater than | `MORE:1MB` |
| `LESS:` | `LESS:value` | File size less than | `LESS:100KB` |
| `DEEPER:` | `DEEPER:N` | Directory depth greater than | `DEEPER:3` |
| `SHALLOWER:` | `SHALLOWER:N` | Directory depth less than | `SHALLOWER:2` |
| `NEWER:` | `NEWER:date` | Modified after date | `NEWER:2024-01-01` |
| `OLDER:` | `OLDER:date` | Modified before date | `OLDER:2023-12-31` |

### Filter Value Standards
- **Size Units**: `B`, `KB`, `MB`, `GB` (1024-based)
- **Time Formats**: ISO 8601 (`YYYY-MM-DD`) or relative (`1h`, `30m`, `2d`)
- **Patterns**: Support glob and regex (tool-dependent)

## 🔄 Filter Precedence and Tie-Breaking

When multiple filters apply to the same target, CNP tools follow these precedence rules:

### Precedence Order (High to Low)
1. **Explicit filters** (`NAME:`, `EXT:`, `TYPE:`)
2. **Size/time filters** (`MORE:`, `LESS:`, `NEWER:`, `OLDER:`, `DEEPER:`, `SHALLOWER:`)
3. **Content filters** (`CONTAINS:`, `HASH:`)
4. **Semantic filters** (`FOR:`, `WHERE:`)

### Tie-Breaking Examples
```bash
# Multiple explicit filters - ALL must match (AND logic)
xfd . NAME:config EXT:json TYPE:file
# Result: Files named "config" with .json extension that are files

# Conflicting size filters - MORE specific wins
xfd . MORE:1MB LESS:500KB
# Result: No matches (impossible condition, tool should warn)

# Mixed precedence - explicit overrides semantic
xfd . FOR:configs NAME:database
# Result: Files named "database" (NAME: wins over FOR:)

# Same precedence - processed left to right
xfd . NAME:test NAME:config
# Result: Files matching "config" (rightmost NAME: wins)
```

### Complex Logic Resolution
```bash
# WHERE: groups override individual filters
xfd . EXT:txt WHERE:(NAME:log OR NAME:temp)
# Result: .txt files named either "log" or "temp"

# Parentheses control evaluation order
xfd . WHERE:(EXT:md OR EXT:txt) AND NAME:readme
# Result: readme.md or readme.txt files
```

---
---

## 🔄 Tool Delegation Protocol

### Delegation Syntax
```
<primary-tool> [args] TO:<target-tool>
```

### Delegation Rules
1. **Output Compatibility**: Delegating tool must produce compatible output format
2. **Error Handling**: Delegation failures must be reported clearly
3. **Tool Discovery**: `TO:` targets must be validated at runtime
4. **Chain Limits**: Maximum 3 delegations to prevent infinite loops

### Standard Delegation Flows
| From Grammar | To Grammar | Use Case |
|-------------|------------|----------|
| Query → Action | Search → Action | `xfd . EXT:log TO:smv` |
| Action → Query | Action → Discovery | `smv move TO:dsc` |
| Any → Any | Processing → Reporting | `tool args TO:rpt` |

---

## 📁 CNP File Format Integration

### .cnp Export Support
All tools SHOULD support:
```
tool [args] FORMAT:cnp INTO:output.cnp
```

### .cnp Import Support  
All tools SHOULD support:
```
tool --from-cnp=input.cnp [additional-args]
```

---

## 🧩 Grammar Extension Rules

### For Query Grammar

- MUST implement all Universal Filter Keywords
- MUST use `<tool> <path> <FILTERS> <ROUTES> <flags>` structure
- MAY add tool-specific filters with UPPERCASE syntax
- MUST NOT redefine Universal routes or base flags

### For Action Grammar  
- MUST implement Universal routes for delegation
- MUST use `<tool> <scope> <targets> <modifiers>` structure
- MAY add grammar-specific flags (non-conflicting)
- MUST support Universal Filter Keywords via delegation

### For Traditional Grammar
- MUST implement base flags (`-h`, `-v`)
- MUST support Universal routes where applicable
- MAY use GNU-style flag conventions
- SHOULD document CNP integration points

---

## ⚠️ Conflict Resolution

### Flag Conflicts
1. **Universal flags** take precedence
2. **Grammar-specific** flags must not conflict within grammar family
3. **Cross-grammar** conflicts resolved by delegation
4. **Tool-specific** flags documented and justified

### Keyword Conflicts
1. **Universal Keywords** cannot be redefined
2. **Grammar extensions** must use consistent semantics
3. **Tool-specific** keywords must be namespaced if conflicting

---

## 🧪 Validation Requirements

### Base Grammar Compliance
All CNP tools MUST:
- ✅ Implement `-h` and `-v` flags consistently
- ✅ Support Universal routes (`TO:`, `INTO:`, `FORMAT:`)
- ✅ Handle delegation protocol correctly
- ✅ Use consistent error reporting
- ✅ Support .cnp format integration

### Grammar Extension Compliance
Grammar specifications MUST:
- ✅ Document extensions clearly
- ✅ Avoid conflicts with base grammar
- ✅ Provide delegation mapping
- ✅ Include validation examples

---

## 📚 Implementation Notes

### Parsing Priority
1. Universal base flags (`-h`, `-v`)
2. Grammar-specific syntax parsing
3. Universal routes processing
4. Delegation execution

### Error Handling
- **Unknown flags**: Suggest correct usage
- **Delegation failures**: Report tool availability
- **Route conflicts**: Clear precedence rules
- **Grammar violations**: Specific error messages

### Performance Considerations
- **Route processing**: Lazy evaluation where possible
- **Delegation overhead**: Minimize process spawning
- **Filter parsing**: Cache compiled filters
- **Format conversion**: Stream when possible

---

## 🔗 Related Specifications

- **CNP_Query_Grammar.md**: Extends this base for declarative tools
- **CNP_Action_Grammar.md**: Extends this base for action-oriented tools
- **CNP_Keyword_Routing.md**: Delegation protocol details
- **CNP_File_Format.md**: .cnp format specification

---

## 📝 Maintenance

This specification is the **single source of truth** for:
- Universal flag definitions
- Route keyword semantics  
- Filter keyword standards
- Delegation protocol rules

Changes to this specification require:
1. Impact analysis on all existing tools
2. Migration plan for breaking changes
3. Updated validation requirements
4. Coordinated releases

**Version**: 1.0  
**Last Updated**: 2025-07-13  
**Next Review**: Before any v1.0.0 tool releases
