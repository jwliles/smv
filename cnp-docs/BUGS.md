# CNP Ecosystem Bugs and Issues

## FIXED ISSUES ✅

### SKL Default Search Behavior
**Date**: 2025-08-06  
**Status**: FIXED ✅  
**Priority**: High

**Issue**: SKL defaulted to live system search instead of closest local database above current root.

**Fix Applied**: Modified `detect_search_mode()` in `skl/skl-cli/src/main.rs` to:
- Search for `tree_diff.db` in directory tree above current location
- Default to database mode when local database found
- Added `run_interactive_database_mode()` for no-query database searches
- Maintains backward compatibility with explicit flags

### DSC Regex Search Functionality
**Date**: 2025-08-06  
**Status**: FIXED ✅  
**Priority**: Medium

**Issue**: The `-r` flag no longer performed regex searches; DSC defaulted to glob instead of regex.

**Fix Applied**: Updated `dsc-rs/src/pattern.rs` to:
- Default to regex mode (matching help text: "regex (default)")
- Only use glob when `-g` flag is explicitly specified
- Maintains backward compatibility

### INX Command-Line Parsing Bug
**Date**: 2025-08-06  
**Status**: FIXED ✅  
**Priority**: Critical

**Issue**: `inx --help` was treated as directory to scan instead of showing help message.

**Fix Applied**: Updated `inx/src/main.rs` to:
- Check for `--help` and `-h` flags before processing arguments
- Proper help display for both long and short flag variants

---

## ACTIVE IMPLEMENTATION ISSUES

### CSV Range Syntax for CNP_Base_Grammar
**Date**: 2025-08-06  
**Status**: PARTIALLY IMPLEMENTED 🔄  
**Priority**: Medium

**Issue**: CSV range syntax needs to be implemented across all CNP tools with filters per CNP_Base_Grammar specification.

**Current Implementation Status**:
- ✅ **DSC**: Full CSV range syntax implemented
  - `SIZE:1MB` (upper bound), `SIZE:1MB,100MB` (range), `SIZE:1MB,` (open range)
  - `DEPTH:3` (exact depth), `AGE:7d` (age filter), `AGE:1d,7d` (age range)
- ❌ **SMV**: Has filter system but needs CSV range syntax
- ❌ **SKL**: Uses INX database queries, may need range syntax
- ❌ **INX**: Database queries may need range syntax support
- ❌ **RPT**: Unknown filter support status
- ❌ **EDT**: Unknown filter support status

**Required Actions**:
1. Implement CSV range syntax in SMV filter parser
2. Assess and implement range syntax in other tools with filters
3. Update CNP_Base_Grammar.md with complete specification
4. Ensure backward compatibility with existing `SIZE>` and `SIZE<` syntax

**Tools with Filter Systems Identified**:
- DSC (✅ implemented)
- SMV (❌ needs implementation) 
- SKL (❌ assessment needed)
- INX (❌ assessment needed)

**Impact**: Inconsistent filter syntax across CNP ecosystem tools.

### INX Scanning Performance and Functionality Issues  
**Date**: 2025-08-06  
**Status**: PARTIALLY FIXED 🔄  
**Priority**: High

**Issue**: INX scanning (both new and delta modes) exhibited severe performance problems, unresponsive UI, and potential incomplete file discovery.

**FIXES APPLIED**:

✅ **1. Expensive MIME Detection Fixed** (`main.rs:199-203`)
   - **FIXED**: Removed expensive `tree_magic_mini::from_filepath()` calls
   - Now uses only fast `mime_guess::from_path()` (extension-based detection)
   - Eliminated file content reading I/O bottleneck
   - Removed unused `tree_magic_mini` import

✅ **2. Comprehensive Scanning with Optional Ignore Patterns** (`main.rs:137-183`)
   - **FIXED**: INX now defaults to comprehensive scanning (all non-hidden files)
   - **NEW FLAGS**: `--ignore` to skip dev/cache dirs, `--all` to include hidden files
   - By default: scans all non-hidden files and folders as intended
   - Optional performance optimization with `--ignore` flag when needed
   - Proper comprehensive indexing while allowing selective exclusions

✅ **3. Fast Mode Option Added**
   - **NEW**: Added `--fast` flag to disable expensive operations
   - Help text updated with performance options
   - `--fast` disables hashing (major performance gain)

⏳ **REMAINING ISSUES** (Need Implementation):

❌ **4. Memory-Inefficient Directory Walking** (`main.rs:142-147`)
   - Still collects ALL directory entries into Vec before processing
   - High memory usage on large directories
   - Blocks UI until complete directory enumeration
   - **TODO**: Implement streaming processing with batched database inserts

❌ **5. Individual Database Inserts** (`main.rs:232-251`)  
   - Still uses individual INSERT per file in single transaction
   - **TODO**: Implement batch INSERT with prepared statements

⏳ **6. Progress Reporting**
   - **PARTIAL**: Added spinner progress bar
   - **TODO**: Improve real-time feedback during expensive operations

**Expected Performance Gains from Applied Fixes**:
- 🚀 **50-80% faster** scanning due to removed MIME content analysis
- 🚀 **Comprehensive indexing** by default (captures all non-hidden files as intended)
- 🚀 **Optional performance boost** with `--ignore` for dev directories
- 🚀 **Zero file hashing** with `--fast` mode

**Usage Examples**:
- `inx /path/to/scan` - Full comprehensive scan (default)
- `inx /path/to/scan --fast` - Fast scan without hashing
- `inx /path/to/scan --ignore` - Skip common dev/cache directories  
- `inx /path/to/scan --all` - Include even hidden files

**Remaining Work**: Memory-efficient streaming and batch database operations

---
