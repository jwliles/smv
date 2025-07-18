## 🧱 Core Functionality (Must-Have)

These are *non-negotiables* to match basic POSIX tools.

### ✅ Basic Commands

* `mv <src> <dest>` – Move files/directories
* `cp <src> <dest>` – Copy files/directories
* Recursive copy (`-r`)
* Force overwrite (`-f`)
* No-clobber / skip existing (`-n`)
* Interactive confirm on overwrite (`-i`)

### ✅ Permissions and Metadata

* Preserve mode, ownership, and timestamps (`--preserve` or `-p`)
* Ability to copy symlinks as symlinks or dereference (`-L`, `-P`)

### ✅ Wildcards & Globs

* Accepts shell-style wildcards (e.g. `*.txt`)
* Expands directories properly

---

## ⚙️ Intermediate Features (Should-Have)

These help it scale for power users and scripting.

### 🧾 Dry Run & Logging

* `--dry-run` for preview
* Verbose logging (`-v`)
* Summary output (count of files moved/copied, skipped, failed)

### 🧰 Advanced Options

* Rename-on-collision (`--rename` or `--suffix` support)
* Backup mode (`--backup`)
* Move only if source is newer (`--update`)
* Progress reporting (especially for large files)

### 🗃️ Metadata Handling

* Extended attribute support (`xattr`)
* Preserve hardlinks, ACLs, etc. (especially for backup workflows)

---

## 💡 Quality of Life (Nice-to-Have)

These are not needed to *replace* `mv`/`cp`, but help it stand out.

### 🧠 Smarter Interface

* Human-readable output
* Error messages with suggested fixes
* Suggestions for commands when syntax is invalid

### 📦 Batch/Interactive Mode

* Interactive TUI/CLI interface for resolving conflicts
* Allow batching multiple operations from a config file or stdin

### 🔌 Extensibility

* Plugin or scriptable hooks (pre/post move scripts)
* Integration with `dsc`, `pathmaster`, etc. for path resolution

### 📂 Rename Templates

* Bulk rename using patterns (`--template '{name}_bak{ext}'`)
* Regex-based renaming

### ☁️ Remote Support

* Operate across remote volumes (e.g., SFTP, cloud)
* Seamless fallback to `rsync` or similar tools

---

## 🚨 Error Handling

* Safe fallback on partial failure (transactional move/copy)
* Retry logic or resumable copy
* Clear handling of symbolic and hard links

---

## 🌐 System Awareness

* Filesystem boundary awareness (`--one-file-system`)
* Copy/move respecting mount points, permissions, and quotas

---

### Optional Goals for a Modern Toolset

If SMV aspires to be more than a core tool:

* File integrity check (checksums before/after copy)
* Deduplication during copy (like `rsync --inplace`)
* Parallelization/threading for large batch ops
* Config file or environment profiles
* JSON or machine-readable output (`--json`)