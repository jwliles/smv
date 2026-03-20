# CNP Tool Status Matrix

**Last Updated**: 2025-07-31
**Purpose**: Authoritative source of truth for all Canopy (CNP) ecosystem tools
**Note**: This document supersedes tool status information in all other documentation files

## 🛠️ Production Tools

| Tool | Status | Version | Grammar | Primary Function | Binary Name |
|------|--------|---------|---------|------------------|-------------|
| **pathmaster** | ✅ Published | v0.2.9 | Traditional | Path alias management | `pathmaster` |
| **smv** | ✅ Published | v0.4.0 | Action | Smart file operations (move, rename, transform) | `smv` |

## 🚧 Beta Tools

| Tool | Status | Version | Grammar | Primary Function | Binary Name |
|------|--------|---------|---------|------------------|-------------|
| **dsc** | 🔄 Release Candidate | v1.0.0 | Query | Ultra-fast directory scanning and statistics | `dsc` |
| **edt** | 🚧 Beta | v0.2.0 | Action | Sed replacement - multi-file structural editor | `edt` |
| **inx** | 🚧 Beta | v0.1.0 | Query | Filesystem indexing and metadata storage | `inx` |
| **skl** | 🚧 Beta | v0.1.0 | Query | FZF replacement - fast fuzzy finding | `skl` |
| **xfd** | 🚧 Beta | v0.1.0 | Query | RipGrep replacement - content search | `xfd` |
| **rpt** | 🚧 Beta | v0.1.0 | Query | Report generation from tool outputs | `rpt` |

## 📋 Planned Tools

| Tool | Status | Version | Grammar | Primary Function | Binary Name |
|------|--------|---------|---------|------------------|-------------|
| **mkr** | 📋 Planned | v0.1.0 | Action | Smart mkdir/touch with templates | `mkr` |
| **cmp** | 📋 Planned | v0.1.0 | Query | File/directory comparison and diffing | `cmp` |
| **arc** | 📋 Planned | v0.1.0 | Traditional | Archive creation and management | `arc` |
| **dff** | 📋 Planned | - | Query | Duplicate file finder | `dff` |

## 📚 Grammar Classifications

### Query Grammar Tools
**Pattern**: `<tool> <path> <FILTERS> <ROUTES> <flags>`  
**Tools**: dsc, skl, xfd, inx, rpt, cmp, dff  
**Example**: `dsc . EXT:rs SIZE>1MB -hp`

### Action Grammar Tools  
**Pattern**: `<tool> <scope> <targets> <modifiers>`  
**Tools**: smv, edt, mkr
**Example**: `smv -snake . pdf recursive`

### Traditional Grammar Tools
**Pattern**: Standard GNU-style flags  
**Tools**: pathmaster, arc  
**Example**: `pathmaster add work ~/dev/work`

## 🔄 Tool Delegation Matrix

| From Tool | Delegates To | For Function |
|-----------|--------------|--------------|
| edt | dsc, skl | File discovery |
| smv | dsc | Live filesystem scanning |
| xfd | inx | Indexed content search |
| skl | inx | Metadata queries |
| All Filter Tools | dsc | Fallback when index unavailable |

## 📊 Implementation Status by Feature

### Core Features
- ✅ **File Operations**: smv (published)
- ✅ **Path Management**: pathmaster (published)
- 🔄 **Directory Scanning**: dsc (release candidate)
- 🚧 **Indexing**: inx (beta)
- 🚧 **Content Search**: xfd (beta)
- 🚧 **Fuzzy Finding**: skl (beta)
- 🚧 **Multi-file Editing**: edt (beta)

### Advanced Features
- 🚧 **Report Generation**: rpt (beta)
- 📋 **File Creation**: mkr (planned)
- 📋 **Comparison**: cmp (planned)
- 📋 **Archiving**: arc (planned)
- 📋 **Duplicate Detection**: dff (planned)

### Integration Features
- 📋 **CNP File Format**: Specification complete, implementation pending
- 📋 **Cross-tool Communication**: Delegation protocols defined
- 🚧 **Unified GUI**: lar (Look And Retrieve) in development
- 📋 **Unified CLI**: cnp shell planned
- ✅ **Undo/Preview Systems**: Implemented in smv, edt

## 🎯 Version Targets

### Pre-v1.0.0 Requirements
All tools must reach stable status with:
- Complete CNP grammar compliance
- Documentation alignment
- Integration testing
- .cnp format support where applicable

### v1.0.0 Ecosystem Release
Coordinated release of all stable tools with:
- Unified documentation
- Complete cross-tool compatibility
- CNP shell integration
- Comprehensive testing suite

## 📝 Usage Guidelines

### For Documentation Authors
- **Always reference this file** for tool status
- **Do not document** deprecated/uncertain tools
- **Mark planned features clearly** in tool-specific docs

### For Developers
- **Update this file** when tool status changes
- **Coordinate grammar changes** with affected tools
- **Test delegation** between related tools

### For Users
- **Stable tools** are safe for production use
- **Beta tools** may have breaking changes
- **Planned tools** are not yet available

---

**Maintenance**: This file should be updated whenever:
- Tool status changes (alpha → beta → stable)
- New tools are added or removed
- Grammar specifications change
- Version numbers are updated
