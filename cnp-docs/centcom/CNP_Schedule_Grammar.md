---
date_created: 2025-10-02T12-00-00
date_updated: 2025-10-18T18-00-32
timestamp: 1759406400432
title: CNP_Schedule_Grammar
id: 8a54694f-87b0-4141-920d-68b1c454ccc1
hash: c7e8b1dee5f7020914d6202d49ef9ad4609e4aae05afcf82e264f9ba6c1f5cc7
---
# CNP Schedule Grammar Specification v1.0

**Purpose**: Define the grammar for scheduled and event-driven execution in the Canopy (CNP) ecosystem.

**Extends**: None directly (but embeds Query and Action grammar structures)

**Applies to**: `sched` (future tool), any daemon-based or repeatable task runner in CNP

---

## Overview

Schedule Grammar allows CNP tools to declare **when**, **what**, and optionally **how** to run other tools, using a structured, portable, declarative format.

Each schedule entry specifies:

- a unique ID
- a time or trigger expression (`when:`)
- a command (`run:`) using any valid CNP grammar
- output handling (`log:`)
- optional metadata and execution control fields

---

## Grammar Structure (YAML)

```yaml
- id: <string>
  when: <time expression | event trigger>
  timezone: <optional IANA zone, e.g. UTC or America/New_York>
  run: |
    <CNP command>
  log:
    path: <filename or directory>
    rotate: [none | daily | weekly | size:10MB]
    keep: <number of rotated logs to keep>
  tags: [optional, list, of, strings]
  retry:
    policy: [fixed | linear | exponential]
    max_attempts: <int>
    initial_delay: <duration>
    max_delay: <duration>
  timeout: <duration, e.g. 2m>
  depends_on:
    - job_id: <string>
      condition: [success | completion]
  concurrency: [allow | skip | queue | kill-previous]
  condition: <optional shell or CNP expression>
  resources:
    max_memory: <e.g. 2GB>
    max_cpu: <e.g. 50%>
    nice: <int>
  vars:
    <key>: <value>  # Must be defined explicitly
  user: <username>
  working_dir: <path>
  env:
    <key>: <value>
  on_success: <hook | email | command>
  on_failure: <hook | email | command>
```

---

## Supported `when:` Syntax

Parsed by the internal `sched-time` engine.

Examples:

```yaml
when: "every 15 minutes"
when: "at 01:00 on Mondays"
when: "on the last Friday of each month"
when: "every 2 hours between 09:00 and 17:00 on weekdays"
when: "at 09:00 on the 1st and 15th of each month"
```

Timezones default to the system timezone unless overridden by `timezone:`.

---

## `run:` Field

Contains any valid CNP command or pipeline. Supports:

- Query Grammar (`dsc`, `xfd`, etc.)
- Action Grammar (`smv`, `edt`, etc.)
- Multi-line, multi-step REPL-style blocks

---

## `log:` Block

```yaml
log:
  path: logs/cleanup.cnp
  rotate: daily
  keep: 30
```

- `path` can be a file or directory. If a directory, filenames will include timestamp.
- `rotate` can be `none`, `daily`, `weekly`, or `size:<limit>`
- `keep` controls how many rotated logs to retain

---

## `retry:` Block

```yaml
retry:
  policy: exponential
  max_attempts: 5
  initial_delay: 10s
  max_delay: 5m
```

- Retry is triggered on non-zero exit code unless `condition:` fails first

---

## `depends_on:`

```yaml
depends_on:
  - job_id: disk-check
    condition: success
```

Dependencies are evaluated before job execution. Cycle detection is mandatory.

---

## `concurrency:`

Controls overlapping runs:

- `allow`: Run concurrently (default)
- `skip`: Do not run if already active
- `queue`: Defer until prior completes
- `kill-previous`: Terminate the last run if it’s still active

Lockfile-based or process-based enforcement is recommended.

---

## `condition:`

Evaluated as a preflight check. Must return 0 (true) or non-zero (false). Examples:

```yaml
condition: "rpt /var/log disk_usage < 80%"
condition: "say match 'EXT:tmp AND SIZE>100MB'"
```

Logical chaining may be supported later (`AND`, `OR`).

---

## `resources:`

Limits applied to execution environment:

```yaml
resources:
  max_memory: 2GB
  max_cpu: 75%
  nice: 5
```

Use Linux primitives (`ulimit`, `nice`, cgroups) for enforcement.

---

## `vars:`

Explicit variable definitions. Only these may be referenced:

```yaml
vars:
  timestamp: "$(date +%Y%m%d-%H%M%S)"
```

Used like:

```yaml
run: |
  dsc ~/data TO:backup_$timestamp
```

Expansion is restricted to prevent injection or unbounded access.

---

## Execution Context

```yaml
user: cleanup
working_dir: /opt/scripts
env:
  LOG_LEVEL: debug
  PATH: /opt/bin:$PATH
```

These are applied via process attributes (`chdir`, `setuid`, and environment substitution).

---

## Notifications

```yaml
on_success: "webhook:https://internal/ok"
on_failure: "email:admin@example.com"
```

Types:

- `webhook:<url>`
- `email:<address>`
- `command:<shell command>`

---

## Example

```yaml
- id: enhanced-example
  when: "every 2 hours between 09:00 and 17:00 on weekdays"
  timezone: UTC
  condition: "rpt /var/log disk_usage < 80%"
  concurrency: skip
  resources:
    max_memory: 1GB
    nice: 5
  vars:
    timestamp: "$(date +%Y%m%d-%H%M%S)"
  run: |
    dsc ~/temp AGE>7d TO:archive_$timestamp
  log:
    path: logs/cleanup.cnp
    rotate: daily
    keep: 30
  on_success: "webhook:https://monitor.internal/job-complete"
  on_failure: "email:ops@company.com"
  user: cleanup
  working_dir: /opt/scripts
  env:
    CLEANUP_MODE: aggressive
  tags: [maintenance, automated]
  retry:
    policy: exponential
    max_attempts: 5
    initial_delay: 10s
    max_delay: 5m
  timeout: 30m
  depends_on:
    - job_id: disk-check
      condition: success
```

---

**Version**: 1.0
**Status**: Draft pending implementation
**Next Step**: Implement `sched` with `sched-time` parser, job state model, and lifecycle enforcement

