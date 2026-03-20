# CNP Wordmaster Grammar Specification v1.0

**Purpose**: Grammar specification for linguistic query and translation operations
**Extends**: CNP_Base_Grammar.md (universal routes and flags only)
**Status**: Standalone Grammar - not extending Query or Action Grammar
**Tool**: wordmaster

---

## 🎯 Wordmaster Grammar Philosophy

Wordmaster Grammar is designed for linguistic operations: querying word information (definitions, etymology, synonyms) and performing translations (words and files). This grammar uses a declarative "linguistic query" approach optimized for:

- **Natural language operations** through semantic keywords
- **File-to-file translation** with format conversion
- **Minimal verbosity** with sensible defaults
- **Clear linguistic intent** in command structure

---

## 🧱 Wordmaster Grammar Structure

```
wordmaster <query> [modifiers] [routes] [flags]
```

### Positional Semantics

| Segment | Description | Examples |
|---------|-------------|----------|
| `query` | Primary linguistic operation | `DEF:word`, `TR:hello`, `READ:file.txt` |
| `modifiers` | Operation parameters | `LANG:en`, `MODE:enhanced`, `SET:value` |
| `routes` | Output directives (CNP Base) | `INTO:es`, `FORMAT:md` |
| `flags` | Minimal modifiers | `-a`, `-s`, `-h`, `-v`, `-j` |

All segments are parsed **left-to-right**. Positional clarity is **mandatory**.

---

## 🔑 Wordmaster Query Keywords

### Primary Query Keywords (UPPERCASE)

| Keyword | Syntax | Purpose | Default Behavior | Example |
|---------|--------|---------|------------------|---------|
| `DEF:` | `DEF:word` | Query word definition | Primary definition | `DEF:guitar` |
| `TR:` | `TR:word` | Query translation | Single translation | `TR:hello INTO:es` |
| `SYN:` | `SYN:word` | Query synonyms/antonyms | All results | `SYN:happy` |
| `ETY:` | `ETY:word` | Query etymology | Full etymology | `ETY:computer` |
| `CHK:` | `CHK:word` | Check spelling | Correctness + suggestions | `CHK:recieve` |
| `READ:` | `READ:file` | Read and translate file | Auto-detect language | `READ:doc.txt INTO:es` |

### Configuration Keywords

| Keyword | Syntax | Purpose | Example |
|---------|--------|---------|---------|
| `APIKEY:` | `APIKEY:service` | API key operations | `APIKEY:deepl SET:key` |
| `CONFIG:` | `CONFIG:action` | Configuration query | `CONFIG:show` |

### Discovery Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `LANGS` | List available languages | `wordmaster LANGS` |
| `LANGCODES` | Display language code table | `wordmaster LANGCODES` |

---

## 🎛️ Wordmaster Modifiers

### Linguistic Modifiers

| Modifier | Syntax | Purpose | Example |
|----------|--------|---------|---------|
| `LANG:` | `LANG:code` | Explicit source language | `TR:word LANG:fr INTO:en` |
| `MODE:` | `MODE:type` | Translation mode | `READ:file.txt INTO:es MODE:enhanced` |
| `SET:` | `SET:value` | Set configuration value | `APIKEY:deepl SET:your-key` |

### Mode Values

| Mode | Description | Use Case |
|------|-------------|----------|
| `basic` | Microsoft Translator only | Fast, literal translation |
| `enhanced` | MS + DeepL polish | Best quality (default) |
| `compare` | Show both side-by-side | Quality comparison |

---

## 🚩 Wordmaster Flags

Wordmaster uses **minimal flags** with sensible defaults:

### Universal Flags (CNP Base Grammar)

| Flag | Long Form | Description | Required |
|------|-----------|-------------|----------|
| `-h` | `--help` | Show help information | ✅ Yes |
| `-v` | `--version` | Show version information | ✅ Yes |

### Wordmaster-Specific Flags

| Flag | Long Form | Description | Example |
|------|-----------|-------------|---------|
| `-a` | `--all` | All results/definitions | `DEF:word -a` |
| `-s` | `--short` | Concise/short output | `ETY:word -s` |
| `-j` | `--json` | JSON output format | `DEF:word -j` |

### Flag Behavior

- **`-a` (all)**: Get all results instead of primary/common
  - `DEF:word -a` → All definitions, not just primary
  - Default without `-a`: Most common/relevant result

- **`-s` (short)**: Get concise output instead of full
  - `ETY:word -s` → Short etymology summary
  - Default without `-s`: Full detailed output (for ETY only)

- **`-j` (json)**: Output as JSON for scripting
  - Works with any query keyword
  - Structured machine-readable output

---

## 📤 Routes (CNP Base Grammar)

Wordmaster supports Universal Routes from CNP Base Grammar:

### Standard Routes

| Route | Syntax | Purpose | Example |
|-------|--------|---------|---------|
| `INTO:` | `INTO:target` | Target language or file | `TR:hello INTO:es` |
| `FORMAT:` | `FORMAT:type` | Output format | `READ:file.txt INTO:es FORMAT:md` |

### Route Processing

1. Execute primary query
2. Apply modifiers (`LANG:`, `MODE:`)
3. Apply format transformation (`FORMAT:`)
4. Write to target (`INTO:`)

---

## 🧠 Command Examples

### Word Queries

```bash
# Definitions
wordmaster DEF:guitar                    # Primary definition
wordmaster DEF:guitar -a                 # All definitions
wordmaster DEF:guitar -j                 # JSON output
wordmaster DEF:hola LANG:es              # Spanish word definition

# Translations
wordmaster TR:hello INTO:es              # Translate to Spanish
wordmaster TR:bonjour LANG:fr INTO:en    # From French to English
wordmaster TR:word INTO:de -j            # JSON output

# Synonyms/Antonyms
wordmaster SYN:happy                     # Get synonyms and antonyms
wordmaster SYN:fast -j                   # JSON format

# Etymology
wordmaster ETY:computer                  # Full etymology (default)
wordmaster ETY:telephone -s              # Short etymology
wordmaster ETY:psychology -a             # All etymology details

# Spell Check
wordmaster CHK:recieve                   # Check spelling
wordmaster CHK:accommodate               # Get suggestions if wrong
```

### File Translation

```bash
# Basic file translation
wordmaster READ:words.txt INTO:es                    # Auto-detect source
wordmaster READ:document.md INTO:fr                  # Markdown file
wordmaster READ:data.txt LANG:en INTO:de             # Explicit source language

# With format conversion
wordmaster READ:input.txt INTO:es FORMAT:md          # Convert to markdown
wordmaster READ:notes.md INTO:fr FORMAT:json         # Convert to JSON

# With translation mode
wordmaster READ:file.txt INTO:es MODE:basic          # Fast translation
wordmaster READ:doc.txt INTO:fr MODE:enhanced        # Best quality (default)
wordmaster READ:article.txt INTO:de MODE:compare     # Compare both modes

# Complete pipeline
wordmaster READ:input.txt LANG:en INTO:es FORMAT:md MODE:enhanced
```

### Configuration

```bash
# Set API keys
wordmaster APIKEY:microsoft_translator SET:your-azure-key
wordmaster APIKEY:deepl SET:your-deepl-key
wordmaster APIKEY:wordnik SET:your-wordnik-key

# View configuration
wordmaster CONFIG:show                   # Show all config (keys masked)
wordmaster CONFIG:path                   # Show config file location
wordmaster APIKEY:deepl                  # Show specific key (masked)
```

### Language Discovery

```bash
# List languages
wordmaster LANGS                         # Simple list
wordmaster LANGCODES                     # Formatted table with codes
```

---

## 🎯 Grammar Rules

### Keyword Processing Order

1. **Query keyword** parsing and validation (`DEF:`, `TR:`, `READ:`, etc.)
2. **Modifier** application (`LANG:`, `MODE:`, `SET:`)
3. **Route** processing (`INTO:`, `FORMAT:`)
4. **Flag** application (`-a`, `-s`, `-j`)

### Sensible Defaults

| Operation | Default Behavior | Override |
|-----------|------------------|----------|
| Definitions | Primary/common only | `-a` for all |
| Etymology | Full detail | `-s` for short |
| Translation | Enhanced mode (MS + DeepL) | `MODE:basic` or `MODE:compare` |
| Source language | Auto-detect | `LANG:code` for explicit |
| Output format | Human-readable text | `-j` for JSON, `FORMAT:` for files |

### Error Handling

- **Invalid keywords** → Suggest corrections, show help
- **Missing required parameters** → Clear error message (e.g., `TR:word` without `INTO:`)
- **Unknown language codes** → Suggest similar codes
- **File not found** → Clear path error message
- **Missing API keys** → Instructive error with setup commands

---

## ⚠️ Safety and Validation

### Input Validation

- **Language codes** → Validate against supported languages
- **File paths** → Check existence and permissions
- **API keys** → Validate format before saving
- **Keywords** → Case-insensitive recognition with correction

### Performance Considerations

- **Caching** → Cache API results to reduce calls
- **Rate limiting** → Respect API rate limits
- **File size** → Warn on large files (>1MB)
- **Timeout handling** → Clear timeout errors with retry hints

---

## 🔗 CNP Integration

### CNP Base Grammar Compliance

Wordmaster **follows** CNP Base Grammar:
- ✅ Universal flags: `-h`, `-v`
- ✅ Universal routes: `INTO:`, `FORMAT:`
- ✅ UPPERCASE keyword style
- ✅ Left-to-right parsing
- ✅ Positional clarity

### Independence from Query/Action Grammars

Wordmaster **does not extend** Query or Action Grammar:
- ❌ No Query filter keywords (`EXT:`, `TYPE:`, `MORE:`, etc.)
- ❌ No Query flags (`-p` for paths, `-r` for recursive)
- ❌ No Action scope keywords (`transform`, `rename`, `organize`)
- ❌ No Action flags (`-f` for force, `-i` for interactive)

This ensures:
- Clean flag namespace (can use `-a`, `-s` freely)
- No confusion with file operations
- Clear linguistic focus
- Simpler user experience

---

## 📝 Grammar Extensions

### Adding New Query Types

New linguistic query types SHOULD:
1. **Use UPPERCASE keywords** following existing pattern
2. **Have sensible defaults** requiring minimal flags
3. **Support JSON output** with `-j` flag
4. **Work with modifiers** where applicable (`LANG:`, `-a`, `-s`)
5. **Provide clear error messages** for invalid input

### Extension Guidelines

- **Use short keywords** (3-4 characters preferred)
- **Follow linguistic semantics** (what does the user want to know?)
- **Document default behavior** clearly
- **Test with real-world examples**

---

## 🔍 Comparison with Other CNP Grammars

| Feature | Query Grammar | Action Grammar | Wordmaster Grammar |
|---------|---------------|----------------|-------------------|
| **Purpose** | File discovery | File operations | Linguistic operations |
| **Filters** | `EXT:`, `TYPE:`, `SIZE>` | Embedded Query filters | Linguistic: `DEF:`, `TR:` |
| **Flags** | `-p`, `-r`, `-n` | `-p`, `-f`, `-r`, `-i` | `-a`, `-s`, `-j` |
| **Routes** | `TO:`, `INTO:`, `FORMAT:` | `INTO:`, `FORMAT:` | `INTO:`, `FORMAT:` |
| **Structure** | `path filters routes` | `scope targets modifiers` | `query modifiers routes` |
| **Philosophy** | Declarative discovery | Imperative action | Linguistic query |

---

**Version**: 1.0
**Created**: 2025-12-22
**Author**: Justin Wayne Liles
**Tool**: wordmaster
**Last Updated**: 2025-12-22
