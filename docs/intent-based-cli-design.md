---
date_created: 2025-10-02T11-25-26
date_updated: 2025-08-30T02-49-20
timestamp: 1759404326182
title: intent-based-cli-design
id: a5591ad6-d944-4263-a998-49201d83c2a0
hash: 86e8d262ec2262769f4ff6cd6b87fffc078ba9606e268987e810193b71301459
---
# Command Design Philosophy: Intent-Based Flags

## Core Principles

1. **Intent Over Mechanics**: Command flags should express *what the user wants to achieve*, not how the program will implement it.

2. **Self-Contained Operations**: Each flag should represent a complete, meaningful operation without requiring additional modifiers.

3. **Sensible Defaults**: Commands should implement the most common use case by default, with minimal required options.

4. **Conceptual Naming**: Name commands based on the mental model users have about the task, not the technical implementation.

## Guidelines

- **Keep flags concise**: Prefer `--extract` over `--move-files-preserve-directories`
- **New concepts deserve new names**: When operations are commonly combined, create a new named command rather than combining multiple flags
- **Avoid flag dependencies**: Users shouldn't need to read documentation to know which flags require other flags
- **Minimize cognitive load**: A user should be able to guess the right command without remembering complex rules

## Examples

Instead of:
```
tool --process --output-format=json --no-validation
```

Consider:
```
tool --export-json
```

Instead of:
```
tool --combine --minify --optimize
```

Consider:
```
tool --package
```

## Implementation Strategy

1. Identify common user workflows and the intent behind them
2. Group related operations that are frequently performed together
3. Create named commands that map to these complete workflows
4. Document the implicit behavior clearly, but don't require users to specify it

This approach creates a tool that feels intuitive and "just works" while requiring less typing and memorization from users.