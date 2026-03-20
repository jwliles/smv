# CNP Action Grammar Specification v1.0

**Purpose**: Grammar specification for action-oriented CNP tools  
**Extends**: CNP_Base_Grammar.md
**Tools**: smv, edt, mkr (and similar action-oriented tools)

---

## 🎯 Action Grammar Philosophy

Action Grammar is designed for tools that **perform operations** on files and directories. Unlike Query Grammar's declarative "what you want" approach, Action Grammar uses an imperative, "how to do it" structure optimized for:

- **Sequential operations** with clear order dependencies
- **Rich flag combinations** that modify behavior
- **Preview and apply** workflows for safety
- **Undo and history** support for reversibility

---

## 🧱 Action Grammar Structure

```
<tool> <scope> <targets> <modifiers>
```

### Structure Explained

| Segment | Meaning | Examples |
|---------|---------|----------|
| **Scope** | Primary operation or transformation | `transform`, `rename`, `organize`, `create` |
| **Targets** | What to operate on | `src/`, `main.rs`, `~/projects`, `. pdf txt` |
| **Modifiers** | How to execute (optional) | `-p`, `-f`, `-r` |

This structure ensures commands flow logically: **scope → targets → modifiers**

---

## 🎛️ Action Grammar Flags

Action Grammar tools use rich flag vocabularies with specific ordering semantics.

### Universal Action Flags

| Flag | Long Form | Description | All Action Tools |
|------|-----------|-------------|------------------|
| `-p` | `--preview` | Show changes without applying | ✅ Required |
| `-f` | `--force` | Skip confirmations/overwrite | ✅ Required |
| `-r` | `--recursive` | Process subdirectories | ✅ Required |
| `-i` | `--interactive` | Interactive confirmation mode | ✅ Required |

### Flag Ordering Semantics

Action Grammar supports **stackable flags with order-dependent behavior**:

```bash
# Order matters - processed left to right
smv snake file.txt -pr    # Preview, then recursive
smv snake file.txt -rp    # Recursive, then preview
smv snake backup/ -fp     # Force, then preview (preview mode wins)
```

### Tool-Specific Flag Extensions

Each action tool may extend the base set:

#### smv (Smart Move)
```bash
-n    --no-clobber     Do not overwrite existing files
-v    --verbose        Verbose output
-u    --undo          Undo last operation
-a    --all           Include hidden files
```

#### edt (Editor)
```bash
-a    --apply         Apply all changes automatically
-cs   --case-sensitive Enable case-sensitive matching
-ni   --no-ignore     Don't ignore build/cache directories
```

#### mkr (Maker)
```bash
-t    --template      Use template for creation
-g    --git           Initialize git repository
-d    --directories   Create directories only
-F    --files         Create files only
```

---

## 🧩 Integration with Query Grammar

Action tools can accept Query Grammar filters through **embedded syntax**:

### Filter Integration Syntax
```bash
# Action tool with Query Grammar filters
smv rm . EXT:log TYPE:file -rp
smv mv . NAME:test* ~/backup/ -f
edt . FIND:old REPLACE:new EXT:rs -a
```

### Processing Order
1. Parse Action Grammar structure (`scope targets modifiers`)
2. Extract embedded Query filters (`EXT:`, `NAME:`, etc.)
3. Apply Action flags (`-r`, `-p`, `-f`)
4. Execute with combined parameters

---

## 🔄 Action Grammar Workflows

### Preview → Apply Pattern
```bash
# 1. Preview changes
smv snake . EXT:pdf -p

# 2. Review output

# 3. Apply if satisfied
smv snake . EXT:pdf -f
```

### Interactive Mode
```bash
# Interactive confirmation for each operation
smv CHANGE:"old" INTO:"new" . -i

# Combines with other flags
smv snake . EXT:txt -ri    # Recursive + interactive
```

### Undo Support
```bash
# Perform operation
smv snake . EXT:pdf -f

# Undo if needed
smv -u
```

---

## 📤 Route Integration

Action tools MUST support Universal Routes from CNP_Base_Grammar:

### Output Routing
```bash
# Save operation log
smv snake . EXT:pdf -p INTO:changes.log

# Export as CNP format
smv organize downloads/ FORMAT:cnp INTO:cleanup.cnp

# Chain to reporting tool
smv cleanup . -p TO:rpt
```

### Delegation Examples
```bash
# Use Query tool for discovery, then act
xfd . EXT:tmp TO:smv rm -f

# Action then report
smv cleanup . -f TO:rpt summary
```

---

## 🧠 Command Examples by Tool

### smv (Smart Move)
```bash
# Case transformations
smv snake . EXT:pdf -p
smv kebab downloads/ -r

# File operations
smv mv *.txt backup/ -rf
smv cp important/ ~/backup/ -p

# Pattern replacement
smv CHANGE:"old" INTO:"new" . -r

# Organization
smv sort downloads/ -p
smv group . TYPE:file -r
```

### edt (Editor)
```bash
# Basic replacement
edt . FIND:dotforge REPLACE:forge -p

# Multi-file editing
edt ./src FIND:old REPLACE:new EXT:rs -a

# Interactive review
edt . FIND:bug REPLACE:fix -i

# Complex patterns
edt . FIND:"api|API" REPLACE:Api EXT:md,txt -rp
```

### mkr (Maker)
```bash
# Template creation
mkr -template rust-cli myproject git readme

# Simple creation
mkr -file config.toml
mkr -dir src/utils

# Batch creation
mkr -template web-app frontend backend database
```

---

## ⚠️ Action Grammar Rules

### Flag Conflict Resolution
1. **Later flags override earlier** in stackable sequences
2. **Contradictory flags** produce warnings
3. **Unknown flags** suggest corrections
4. **Base grammar flags** always take precedence

### Safety Requirements
- **Destructive operations** MUST support `-p` preview
- **File overwrites** MUST respect `-n` no-clobber  
- **Recursive operations** MUST warn on large scope
- **Undo capabilities** SHOULD be available for file operations

### Performance Guidelines
- **Large operations** SHOULD show progress
- **Preview mode** SHOULD be fast and accurate
- **Interactive mode** SHOULD batch confirmations sensibly
- **Recursive operations** SHOULD optimize traversal

---

## 🔗 Related Specifications

- **CNP_Base_Grammar.md**: Foundation that this extends
- **CNP_Query_Grammar.md**: Complementary declarative grammar
- **CNP_Schedule_Grammar.md**: Schedule and timing integration
- **CNP_File_Format.md**: Output format integration

---

## 📝 Grammar Extensions

### Adding New Action Tools

New action-oriented tools SHOULD:
1. **Follow the scope → targets → modifiers structure**
2. **Implement universal action flags** (`-p`, `-f`, `-r`, `-i`)
3. **Support Query Grammar filter embedding**
4. **Provide preview/apply workflows**
5. **Document tool-specific flag extensions**

### Flag Addition Guidelines
- **Use short forms** for frequent operations
- **Provide long forms** for clarity
- **Avoid conflicts** with universal flags
- **Document order semantics** for stackable flags
- **Test combinations** thoroughly

---

**Version**: 1.0  
**Converted from**: CNP-CLI-Philosophy.md  
**Last Updated**: 2025-07-13
