---
date_created: 2025-10-02T11-25-26
date_updated: 2025-08-30T02-49-20
timestamp: 1759404326268
title: file-operations
id: 2e45f707-91d2-42a8-b0ad-3fe2a129b9de
hash: 46399dba7c256f5a0b058056abcd1455b7ce9987e2123baf6cfaffd9c770bfbe
---
# File Operations

Guide to SMV's file operation commands.

## Move Operations
```bash
smv mv source.txt dest.txt

# Multi-source move with CNP grammar and preview
smv -move EXT:png FROM:a/,b/ TO:c/ -p
smv -move EXT:png FROM:a/*.png,b/*.png TO:c/ -gr -p
```

## Copy Operations  
```bash
smv cp source.txt backup.txt
```

## Delete Operations
```bash
smv rm . EXT:log -p
```

## Creation Operations
```bash
smv -cd newdir
smv -cf newfile.txt
```

More detailed documentation coming soon...
