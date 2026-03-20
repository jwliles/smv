# CNP Orchestration Design Specification

**Purpose:**  
Define how the `cnp` tool functions as the central orchestrator and keyword router for all CNP tools (RPT, INX, CMP, SAY, SMV, etc.).

---

## 1. Overview

The `cnp` binary coordinates commands across the suite by:

- Parsing user input into **Base + Query + Reporting + Action grammar components**.
- Mapping keywords to their **owning tool** via a central registry.
- Building **execution pipelines** (e.g., INX → RPT → SMV).
- Executing these pipelines with **canonical input/output formats** (JSON/CNP).
- Supporting **REPL mode** for iterative multi-tool workflows.

---

## 2. Responsibilities

### Centralized
- Maintain **keyword ownership registry** (single source of truth).
- Enforce **Base Grammar compliance** (flags and routes).
- Handle **delegation pipelines** across tools.
- Provide **cross-tool translation** and route coordination.

### Tool-specific
- Each tool (RPT, INX, etc.) implements:
  - Its own filters, modifiers, and routes.
  - Canonical input/output handling.
  - A contract for integration (e.g., JSON schema).

---

## 3. Command Flow

### Example Command
```
cnp rpt files SIZE:>100MB GROUP:ext FORMAT:json INTO:report.json
```

**Flow:**
1. Parse command (tokenize keywords, routes, flags).
2. Classify tokens by ownership (`SIZE` → INX, `GROUP` → RPT).
3. Plan pipeline: `inx` filters → `rpt` aggregates → output routed.
4. Execute tools sequentially (initially via subprocess).
5. Pipe outputs using canonical format.

---

## 4. Keyword Ownership Registry

Defined in a central module:

```
SIZE       → INX
MODIFIED   → INX
HASH       → CMP
DUPLICATE  → CMP
GROUP      → RPT
COLUMNS    → RPT
STYLE      → RPT
NAME       → SAY
STARTS     → SAY
ENDS       → SAY
```

- Registry is used by `cnp` to split workloads.
- Tools reference it to warn about out-of-scope keywords when run standalone.

---

## 5. Pipeline Planner

### Input:
Parsed command: `rpt files SIZE:>100MB GROUP:ext`

### Output:
Pipeline stages:
1. `inx` (SIZE:>100MB)
2. `rpt` (GROUP:ext)

### Algorithm:
- Group filters by tool.
- Determine required execution order.
- Spawn subprocess for each tool (MVP) → pass output via JSON/CNP.
- Apply routes (TO/INTO/FORMAT) at the final stage.

---

## 6. Base Grammar Compliance

Handled entirely in `cnp`:

- Flags:
  - `-h` (help), `-v` (version)
  - `-p` (preview pipeline)
  - `-r` (recursive; forwarded where relevant)
- Routes:
  - `TO:` (send to another tool)
  - `INTO:` (write to file)
  - `FORMAT:` (json, csv, cnp, table, tree)

---

## 7. Execution Modes

### Direct Mode
```
cnp rpt files SIZE:>1MB GROUP:ext
```
- Parses, delegates, executes pipeline.

### REPL Mode
```
$ cnp
CNP> rpt files SIZE:>1MB
CNP> smv rename snake_case
CNP> exit
```

---

## 8. Tool Contracts

- **Input**: Accept canonical JSON/CNP describing file/system state.
- **Output**: Emit canonical JSON/CNP (ready for next stage or user).
- **Declarative**: Tools never need to know about upstream/downstream tools.

---

## 9. Future Enhancements
- Replace subprocess pipelines with **IPC** (Unix sockets or shared library calls).
- **Dynamic plugin model** for keyword ownership (extensible without recompilation).
- **Unified audit logging** for pipeline execution.

---

## 10. Next Steps

1. Implement **Base Grammar parsing** in `cnp`.
2. Add **keyword ownership registry** (static table).
3. Build **pipeline planner** (determine stages).
4. Stub **execution** (just prints planned pipeline for now).
5. Incrementally wire tools into pipeline.
