# CNP Project Command Philosophy

The `cnp` toolset is built around a core principle of **clarity through consistency**. Each tool follows a common command-line structure that helps users easily remember, predict, and script interactions across the suite.

Inspired by the ISO 8601 date format (`YYYY-MM-DD`), the CLI design uses a "largest to smallest" structure:

---

## General Command Format

```
<tool> [scope] [targets] [modifiers]
```

### Interactive Guidance System

All CNP tools feature an Excel-like command guidance system:

```bash
$ smv -snake . pdf
[smv] [-snake] [. pdf] [preview|recursive|force]
 cmd   scope    targets    modifiers (optional)
```

- **F1**: Context-sensitive help for current position
- **Tab**: Cycle through valid options
- **Backspace**: Navigate to previous position
- Real-time validation and option preview

### Segments Explained

| Segment        | Meaning                               | Examples                                      |
| -------------- | ------------------------------------- | --------------------------------------------- |
| **Scope** | Primary operation (single flag)       | `-template`, `-sort`, `-scan`, `-snake` |
| **Targets**    | What to operate on | `src/`, `main.rs`, `~/projects`, `. pdf txt`               |
| **Modifiers**  | How to execute (optional)          | `preview`, `force`, `recursive`          |

This structure ensures your commands flow logically from intent ➜ customization ➜ subject.

---

## Tool-by-Tool Consistency

### `mkr` – File/Folder Creation

```sh
mkr -template rust-cli mytool git readme
```

* Scope: `-template`
* Target: `rust-cli mytool`
* Modifiers: `git readme`

### `smv` – Move, Rename, Organize

```sh
smv -sort downloads/ preview
smv -snake . pdf txt recursive
smv -move *.pdf ~/backup/
```

* Scope: `-sort`, `-snake`, `-move`
* Targets: `downloads/`, `. pdf txt`, `*.pdf ~/backup/`
* Modifiers: `preview`, `recursive`

### `xfd` – Search and Filter

```sh
xfd -name ~/vault "*.md" limit 10
```

* Scope: `-name`
* Targets: `~/vault "*.md"`
* Modifier: `limit 10`

### `inx` – Indexing and Metadata

```sh
inx -scan ~/projects depth 3
```

* Scope: `-scan`
* Target: `~/projects`
* Modifier: `depth 3`

---

## Universal Behaviors

* **CNP REPL Integration**: All tools work within CNP REPL environment
* **Interactive Guidance**: Excel-like command preview with F1 help and Tab completion
* **Position-based Parsing**: Sequence and quantity constraints eliminate ambiguity
* **No `--` Clutter**: After scope, flags are simple words (`preview`, `recursive`)
* **Inference**: Smart detection of intent (e.g., file vs. folder in `mkr`)
* **Parent Directory Creation**: Default on (like `mkdir -p`)
* **Preview / Dry-run Mode**: Available as `preview` modifier in most tools
* **Undo**: Built-in where destructive actions may occur (e.g., `-undo` scope)
* **Fallback to TUI**: If run with no arguments, tools may launch TUI or interactive mode

---

## CNP REPL Workflow

CNP serves as the unified entry point for all tools:

```bash
$ cnp
CNP> smv -snake . pdf preview
CNP> mkr -template rust-cli mytool
CNP> xfd -name ~/vault "*.md"
CNP> exit
$
```

* **Persistent context**: REPL remembers project state
* **Tool discovery**: Tab completion shows all available CNP tools
* **Cross-tool workflows**: Chain operations without re-prefixing
* **Consistent guidance**: Same interactive help system across all tools

---

## Alternative Command Formats

For compatibility and different use cases, CNP also supports traditional long-form options:

### Traditional Format (Alternative)

```
<tool> [scope/mode] [modifiers/options] [targets]
```

**Examples:**
```sh
mkr --template rust-cli --git --readme mytool
smv --sort --preview downloads/
xfd --name "*.md" --limit 10 ~/vault
inx --scan --depth 3 ~/projects
```

### Optional Disambiguation Flags

Some tools offer optional disambiguation flags:

* `--cf` (create file)
* `--md` (make directory)

These are not required but help in ambiguous cases like `mkr docs`.

---

This command philosophy helps every `cnp` tool feel familiar while remaining powerful and focused. Write once, remember forever.