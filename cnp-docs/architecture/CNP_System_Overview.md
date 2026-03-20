# 🧠 CNP Architecture Overview - Canopy

**CNP** is a unified, modular ecosystem designed to manage your filesystem with intelligence, reversibility, and structure. It combines tools for creation, navigation, transformation, and indexing under a single interactive shell environment that functions similarly to a language REPL.

This document merges and supersedes the original project design document, consolidating the vision, system design, and operational structure under the CNP identity.

---

## 🧭 High-Level Components

### 1. `CNP` (Core Shell)

* The entry point into the ecosystem
* Behaves like a domain-specific shell or REPL
* Controls and orchestrates all sub-tools (mkr, smv, xfd, inx)
* Supports history, preview, undo, and batch operation chaining
* Also supports direct CLI use: `CNP smv --rename ./photos`

---

## 🧰 Tool Modules

### 2. `smv` (Smart Move)

* Enhanced `mv` replacement
* Supports sorting, renaming, bulk transformations
* Includes preview/dry-run and interactive mode
* Fully reversible via CNP's undo system

### 3. `mkr` (Maker)

* Replacement for `mkdir` and `touch`
* Supports file/folder creation, scaffolding, and templates
* Uses smart inference for file vs. folder
* Tied to project initialization flows

### 4. `xfd` (Extra Find)

* Search and filter tool, like `find` but smarter
* Supports filters by name, extension, content, metadata
* Index-aware via `inx`

### 5. `inx` (Indexer)

* Metadata indexer, feeding into `xfd` and undo tracking
* Tracks timestamps, sizes, types, regions, and tags
* Provides persistent context to CNP shell sessions

---

## 🧠 Core Services

### 6. `Undo Engine`

* Centralized history graph shared by all tools
* Supports:

  * Tree-like branching (Vim-style undo)
  * Region-based undo (Emacs-style scopes)
  * Kill-ring (operation clipboard)
* Exposed in GUI and TUI with time-travel, labeling, and diffs

### 7. `Preview System`

* Shared dry-run engine across all tools
* Visual diff previews before committing actions
* Powers both CLI and GUI/TUI feedback

### 8. `Region Manager`

* Logical groupings of operations (per directory, project, or path group)
* Enables scoped undo, selective history viewing, and targeted reverts

### 9. `Shell Interface`

* Interactive prompt launched with `CNP`
* Accepts tool commands (`smv`, `mkr`, `xfd`, `inx`) as subcommands
* Supports help, history, fuzzy search, plugin loading, and undo navigation
* Optional TUI overlay for browsing tree structure and regions

---

## 🖥 Interfaces

### CLI

* Tools runnable standalone (`smv`, `xfd`, etc.)
* Or invoked through `CNP` with direct subcommands

### Interactive Shell

* Launch `CNP` → drop into intelligent REPL
* Immediate access to all tools
* Shared undo, preview, and context

### GUI

* Uses egui or Tauri
* Graphical version of `CNP` with visual workflows
* Preview graphs, file trees, undo history, and diffing

### TUI

* Text-based interface via Crossterm
* For power users who want structured interactivity

---

## 📦 Ecosystem Relationships

```
           +---------------------+
           |      CNP Shell      |
           +---------------------+
                     |
        +------------+------------+
        |            |            |
     [smv]         [mkr]        [xfd]
        \            |             /
         \           |            /
         +-----------+-----------+
                     |
                  [inx]
                     |
                [Undo Core]
                     |
                [Preview Core]
                     |
               [Region Manager]
```

* All tools funnel state changes to the **Undo Engine**
* **Inx** feeds metadata and indexing into all tools
* **Preview and Undo** are core cross-tool services
* The **Shell (CNP)** is the umbrella that orchestrates tool access, context, and interactivity

---

## ⚙️ System Architecture Highlights

* **Shared Codebase**: CLI, GUI, and TUI interfaces are built on common libraries.
* **Interface Parity**: Every tool and feature is accessible via all UIs.
* **Plugin and Scripting**: Extensible via APIs and embedded scripting support.
* **Structured Caching**: Tiered cache and shared schema for efficiency.
* **Cross-Component Communication**: Cap’n Proto or gRPC handles messaging.

---

## 🧪 Design Philosophy

* **Smart and Simple**: Every command should feel intelligent but lightweight
* **Modular but Unified**: Tools operate standalone or inside `CNP`
* **Undoable by Design**: Every action is tracked, previewed, and reversible
* **CLI-first, GUI-aware**: First-class CLI experience with optional GUI/TUI control center
* **Hackable and Scriptable**: Plugins and shell scripting encouraged

---

## ✅ Summary

`CNP` is not just a shell — it's a thinking layer over your filesystem. With `smv`, `mkr`, `xfd`, and `inx` operating as modular limbs, and preview/undo/region systems keeping everything safe and reversible, CNP gives you full creative and operational control over your system like no other tooling suite.

This document reflects the complete, unified design vision for CNP as both an interface platform and a modular tool ecosystem.

## 🔗 CNP Tool Interactions and Architecture Overview

This document describes how tools in the CNP ecosystem interact, where their responsibilities begin and end, and which components they delegate to.

---

### 🧱 Core Layers

#### 1. **CNP Shell (**``**)**

- **Role**: Central entry point and coordinator.
- **Interfaces**: CLI / REPL / TUI / GUI.
- **Responsibilities**:
  - Hosts and launches subtools.
  - Manages shared state (undo, preview, region tracking).
  - Supports interactive command chaining and scripting.

---

### 🧰 Tool Boundaries and Delegation

#### `dsc` – Discovery

- **Does**: Live filesystem scanning.
- **Delegated to by**:
  - `inx` for populating or validating indexes.
  - `edt` and `skl` for content scanning outside indexed scope.
- **Boundary**: Only touches raw filesystem (no index).

#### `inx` – Indexer

- **Does**: Stores metadata snapshot of files.
- **Used by**:
  - `xfd`, `smv`, `skl`, `dff`, `rpt`, `cmp`.
- **Delegates to**: `dsc` when building or refreshing index.

#### `xfd` – Extra Find

- **Does**: Search based on indexed metadata.
- **Boundary**: Cannot query the live filesystem directly.
- **Delegates to**: `inx` (always). `say` for pattern logic.

#### `smv` – Smart Move

- **Does**: Rename, move, sort, refactor files.
- **Delegates to**:
  - `dsc` for live source selection.
  - `inx` for indexed move plans.
  - `rpt` for reporting move results.
- **Shares**: Undo system, preview system.

#### `edt` – Structural Editor

- **Does**: Interactive multi-file editing.
- **Delegates to**:
  - `dsc` for file discovery.
  - `skl` for in-file match finding.
  - `say` for structural match patterns.

#### `skl` – Skeleton (Content Search)

- **Does**: Grep-like recursive search.
- **Delegates to**:
  - `inx` for known content.
  - `dsc` for fallback.
  - `say` for query parsing.

#### `say` – Matcher Engine

- **Does**: Declarative pattern matching.
- **Used by**: `xfd`, `skl`, `edt`, `dff`.
- **Boundary**: Pure matcher, no file I/O.

#### `rpt` – Reporter

- **Does**: Format operation output.
- **Used by**: `smv`, `xfd`, `inx`, `cmp`.
- **Boundary**: Read-only transformation layer.

#### `cmp` – Compare

- **Does**: Compare snapshots of file trees.
- **Delegates to**:
  - `inx` for historical or indexed states.
  - `dsc` for live checks.

#### `arc` – Archive

- **Does**: Compress/restore file trees.
- **May interact with**: `smv`, `mkr`, `cmp`.
- **Planned**: Hook into pre/post move/archive state tracking.

#### `pathmaster`

- **Does**: Manage system PATH.
- **Standalone**: No delegation or integration needed.

---

### 🔄 Shared Systems

| System             | Used By                              |
| ------------------ | ------------------------------------ |
| **Undo Engine**    | `smv`, `edt`, `mkr`, (future: `cmp`) |
| **Preview Core**   | `smv`, `mkr`, `edt`, `rpt`           |
| **Region Manager** | `smv`, `edt`, `cnp shell`            |

---

### 🧭 Flow Example: Interactive Refactor

```sh
cnp> xfd --ext rs --name "temp"
 → Uses inx to locate candidates
 → say handles fuzzy matching
 → Results piped to smv
cnp> smv --rename snake_case
 → Uses preview system and undo engine
```

---

### 📌 Summary

- `dsc` = live scanner
- `inx` = index storage
- `xfd` = query interface for inx
- `smv` = moves files, may use either
- `edt` = edits content, delegates to `skl` and `say`
- `cnp` = glue that hosts everything
- Shared systems make all actions reversible and previewable

# CNP Grammar, INX, and DSC Integration Summary

## ✨ CNP Grammar and Tool Communication

- **CNP Grammar** serves as a *universal translator* at the central hub ("Centcom").
- Each tool communicates in its native syntax:
  - **XFD** speaks **Query** (declarative filters, search language)
  - **SMV** speaks **Action** (imperative source-to-destination commands)
  - **MKR** speaks **Creation** (directory and scaffold generation)
- Tools translate to/from the central grammar using the `say` tool.
- This approach maintains native syntax per tool, avoiding enforced conformity or "language imperialism."

## ⚔️ Delegation and Discovery

- `dsc` **has surpassed** `fd` in all benchmark categories and is now the default discovery engine.
- All delegation for file discovery from tools like INX or SMV should route through `dsc`.
- **Delegation Example Flow:**
  ```bash
  smv *.md archive/  # SMV command
  ↓
  Translated to CNP via say (Centcom)
  ↓
  Routed as discovery query to `dsc`
  ↓
  Result passed back as action list to SMV
  ```

## 🏦 INX + DSC: The Faux Live System

- **INX** provides heavy, indexed snapshot-based data storage.
- **DSC** delivers ultra-fast filesystem scanning on-demand.
- Together, they simulate a **"live view"** without constant reindexing.

### ✍️ User-Defined YAML Scan Schedule

- Users define scan behavior using a YAML config file that includes:
  - **Scan frequencies per path**
  - **Blacklist/whitelist behavior**
  - **Trigger rules (events, intervals)**

### 🕒 Example:

```yaml
/home/jwl/:
  every: 10s
  dispatch: dsc
  on-change:
    - action: index
```

- DSC checks `/home/jwl/` every 10s for changes.
- If changes are detected, DSC returns the list to INX for indexing.

## 🔎 Query Fallback & Escalation

1. User performs a search via INX.
2. INX checks its index.
3. If no results:
   - INX dispatches `dsc` to confirm the file exists in the given path.
4. If still not found:
   - `dsc` escalates to scan **one directory level higher**, repeating the process.
5. If file is located:
   - INX indexes it.
   - Result is returned to user as if it were always indexed.

## 🌟 Benefits

- Instant response when index is valid
- Graceful fallback to real-time scan
- "Feels live" without constant crawling
- Modular architecture using language delegation
