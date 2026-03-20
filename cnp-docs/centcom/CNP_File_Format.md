# 📦 CNP File Format Specification (v1.0)

  ## 🧭 Purpose

  The `.cnp` format is a portable, tool-agnostic snapshot format for the Canopy (CNP) ecosystem. It allows any CNP tool to export and import structured command data, query results, audit logs, rename plans, tree views, and more.

------

  ## 🔖 Format Summary

  - **Extension**: `.cnp`
  - **Serialization**: YAML (preferred), optionally JSON or TOML
  - **Use Cases**:
    - Export CLI queries, plans, trees, and summaries
    - Import into compatible tools for preview or execution
    - Replay command context in the REPL
    - Share offline bundles between environments

------

  ## 🧱 Top-Level Structure

  ```yaml
  version: 1
  cnp_id: "cnp-bundle-000142"
  created: "2025-07-12T19:43:00Z"
  creator: "xfd v0.6.1"
  tags: [index, query, smv, audit]

  command: "xfd ./vault EXT:md SIZE>1MB FORMAT:cnp INTO:vault-heavy.cnp"
  description: "Markdown files larger than 1MB in vault, exported by XFD"
  notes: |
    This bundle was generated for previewing documentation cleanup.
    It may be piped to SMV or analyzed with RPT.

  origin: "/home/user/projects/cnp"

  entries:
    - tool: xfd
      type: result
      label: "query results"
      query:
        path: ./vault
        filters: ["EXT:md", "SIZE>1MB"]
        flags: ["-rp"]
      output:
        format: json
        records:
          - path: "./vault/guide.md"
            size: 1342123
            modified: "2024-11-03T10:22:13Z"
            ext: md
          - path: "./vault/api/design.md"
            size: 1783002
            modified: "2024-12-28T15:43:00Z"
            ext: md

    - tool: smv
      type: plan
      label: "rename plan"
      preview: true
      source: xfd
      operations:
        - from: "./vault/guide.md"
          to: "./docs/guide.md"
        - from: "./vault/api/design.md"
          to: "./docs/api/design.md"

    - tool: rpt
      type: summary
      label: "size report"
      columns: ["path", "size"]
      total_size: "3125125"
      max_size: "1783002"
      grouped_by: "ext"

    - tool: look
      type: tree
      label: "file tree"
      format: ascii
      tree:
        - "vault/"
        - "├── guide.md"
        - "└── api/"
        - "    └── design.md"
  ```

------

  ## 🧩 Entry Types (per tool)

  | Tool   | Type         | Description                         |
  | ------ | ------------ | ----------------------------------- |
  | `xfd`  | result       | CLI query result                    |
  | `inx`  | scan, delta  | Index snapshot or diff              |
  | `smv`  | plan         | Rename/move operation plan          |
  | `rpt`  | summary      | Aggregated report                   |
  | `look` | tree         | Directory tree or grouped structure |
  | `say`  | match, audit | Expression evaluation or debug tree |
  | `cnp`  | session      | REPL history or multi-step chain    |

------

  ## 🔄 Replay Support

  All entries may include:

  - `command`: original CLI form
  - `source`: originating tool or file
  - `timestamp`: entry-specific
  - `query`, `flags`, or `filters`
  - `output`: result records or structured content

  Tools importing `.cnp` should:

  - Filter for `entries.tool == self`
  - Ignore or warn on unsupported entries
  - Support `--from-cnp` or REPL `load` command

------

  ## 📤 Export Guidelines

  All tools may emit:

  ```sh
  TO:FORMAT:cnp INTO:output.cnp
  # or
  --export-cnp=output.cnp
  ```

  Tools may support:

  ```sh
  --append-to=existing.cnp
  ```

------

  ## 📥 Import Guidelines

  All tools should support:

  ```sh
  --from-cnp=input.cnp
  # or in REPL
  cnp> load input.cnp
  ```

------

  ## 🧠 Route Keyword: `EXPLAIN:`

  The `EXPLAIN:` route can be used in place of `TO:` or `INTO:` to emit a readable breakdown of the query’s evaluation plan or match logic tree. This is helpful for debugging or for tools like `say` and `skl`.

  ```bash
  xfd . NAME:CNP EXPLAIN:
  ```

  Tools that support `EXPLAIN:` should output:

  - A visual tree or description of the query’s structure
  - Details on parsed filters, logic, and match routing
  - Optional structured format for audit (`--json`, `--summary`)

------

  ## 📝 Field Naming Conventions

  ### Top-Level Fields
  - Use **lowercase** for standard metadata: `version`, `created`, `creator`, `origin`
  - Use **snake_case** for compound identifiers: `cnp_id`, `created_at`, `modified_at`
  - Use **plural nouns** for collections: `entries`, `tags`, `operations`

  ### Entry-Level Fields
  - Use **lowercase** for common properties: `tool`, `type`, `label`, `source`
  - Use **snake_case** for compound properties: `total_size`, `max_size`, `grouped_by`
  - Use **camelCase** for JSON compatibility when needed: `fileName`, `fileSize`

  ### Field Naming Examples
  ```yaml
  # Correct naming
  cnp_id: "bundle-001"           # snake_case for IDs
  created: "2025-07-12T19:43:00Z" # lowercase for timestamps
  total_size: "3125125"          # snake_case for compounds
  entries: []                    # plural for collections

  # Avoid mixing styles
  creation_date: "..."           # Don't mix with 'created'
  totalSize: "..."              # Don't mix camelCase with snake_case
  entry: []                      # Use plural 'entries' not singular
  ```

  ## 🛡 Validation Rules

  - `.cnp` files must include `version`
  - Unknown `entries.tool` types must be ignored safely
  - JSON and TOML variants must preserve top-level structure
  - Field names must follow naming convention consistency within each file

------

  ## 🧠 Future Extensions

  - Entry signing (GPG, hash)
  - Binary blobs / file previews
  - Change history / undo logs
  - Cross-platform path normalization
  - Streamable mode (`.cnp.log`) for watch-based tools

------

  This document defines the canonical specification for `.cnp` files in the CNP ecosystem.
