# REPL Mode (Planned)

Interactive command environment that solves the CNP verbosity problem.

## Overview

REPL (Read-Eval-Print Loop) mode provides an interactive shell environment where you can use familiar commands without the `smv` prefix.

## Benefits

### Solves Verbosity Problem
```bash
# Current: Always need smv prefix
smv mv file.txt dest.txt     # 6 chars + args
smv cp file.txt backup.txt   # 6 chars + args  
smv rm old_files.txt         # 6 chars + args

# REPL mode: Familiar commands
$ smv repl
smv> mv file.txt dest.txt    # 2 chars + args!
smv> cp file.txt backup.txt  # 2 chars + args!
smv> rm old_files.txt        # 2 chars + args!
```

### Session Context
- **Persistent state**: Remember current directory and settings
- **Command history**: Navigate previous commands with arrow keys
- **Tab completion**: Auto-complete file names and commands
- **Multi-operation workflows**: Chain operations efficiently

## Planned Features

### Command Support
All SMV commands available without prefix:
```bash
smv> mv source.txt dest.txt
smv> snake . EXT:txt -p
smv> rm . EXT:log -F
smv> CHANGE "old" INTO "new" . -p
```

### Enhanced Features
- **Auto-completion**: Files, directories, and command options
- **History search**: Ctrl+R to search command history
- **Command validation**: Real-time syntax checking
- **Help integration**: `help` command for inline documentation

### Session Management
```bash
# Enter REPL
$ smv repl

# Exit REPL
smv> exit
smv> quit
smv> Ctrl+D
```

## Use Cases

### Batch File Operations
```bash
$ smv repl
smv> cd ~/Downloads
smv> snake . EXT:pdf -p
smv> snake . EXT:pdf -F
smv> mv . EXT:pdf ~/Documents/PDFs/
smv> rm . EXT:tmp -F
```

### Development Workflows
```bash
smv> cd ./src
smv> snake . EXT:rs -p
smv> kebab . EXT:md -p
smv> CHANGE "old_api" INTO "new_api" . EXT:rs -p
```

### Interactive Exploration
```bash
smv> ls
smv> snake . -p
smv> snake . EXT:txt -p  # Refine the operation
smv> snake . EXT:txt -F  # Apply when satisfied
```

## Implementation Status

🚧 **In Development**: REPL mode is a high-priority feature planned for the next development sessions.

### Planned Architecture
- **Rustyline integration**: Command line editing and history
- **Command parser**: Reuse existing SMV command parsing
- **Session state**: Track current directory and user preferences
- **Error handling**: Graceful error recovery in interactive mode

### Timeline
- **Phase 1**: Basic REPL with core commands
- **Phase 2**: Auto-completion and history
- **Phase 3**: Advanced features and integration