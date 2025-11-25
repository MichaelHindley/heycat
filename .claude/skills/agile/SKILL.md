---
name: agile
description: |
  Manage the project's Kanban-style agile workflow. Use this skill when you need to:
  - Create new features, bugs, or tasks in the backlog
  - Move issues through workflow stages (backlog -> todo -> in-progress -> review -> done)
  - List current issues and their status
  - Work through issues with stage-appropriate guidance
  - Archive or delete completed work
---

# Agile Workflow Management

Manage issues in the project's Kanban board located in the `agile/` folder.

## Workflow Stages

```
1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done
```

**Important:** Only sequential transitions are allowed (forward or back by one stage).

## Commands

### Create an Issue

```bash
bun .claude/skills/agile/agile.ts create <type> <name> [--title "Title"] [--owner "Name"] [--stage <stage>]
```

**Arguments:**
- `type`: `feature`, `bug`, or `task`
- `name`: kebab-case identifier (e.g., `user-authentication`)

**Options:**
- `--title, -t`: Human-readable title (defaults to name in Title Case)
- `--owner, -o`: Issue owner/assignee name
- `--stage, -s`: Initial stage (default: `1-backlog`)

**Examples:**
```bash
bun .claude/skills/agile/agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
bun .claude/skills/agile/agile.ts create bug fix-login --stage 2-todo --owner "Bob"
bun .claude/skills/agile/agile.ts create task update-deps --owner "Charlie"
```

### Move an Issue

```bash
bun .claude/skills/agile/agile.ts move <name> <stage>
```

**Arguments:**
- `name`: Issue name (with or without `.md` extension)
- `stage`: Target stage (`1-backlog`, `2-todo`, `3-in-progress`, `4-review`, `5-done`)

**Valid Transitions:**
- `1-backlog` can move to: `2-todo`
- `2-todo` can move to: `1-backlog`, `3-in-progress`
- `3-in-progress` can move to: `2-todo`, `4-review`
- `4-review` can move to: `3-in-progress`, `5-done`
- `5-done` can move to: `4-review` (reopen)

**Transition Requirements (Strict Validation):**
The move command validates that issues meet requirements before advancing:

| Target Stage | Requirements |
|--------------|--------------|
| `2-todo` | Description section must be complete (no placeholders) |
| `3-in-progress` | Owner must be assigned, Technical Notes must be present |
| `4-review` | At least one Definition of Done item must be checked |
| `5-done` | All Definition of Done items must be checked |

If validation fails, run `work <name>` for guidance on what needs to be completed.

**Examples:**
```bash
bun .claude/skills/agile/agile.ts move user-auth 2-todo
bun .claude/skills/agile/agile.ts move user-auth 3-in-progress
```

### Work on an Issue

```bash
bun .claude/skills/agile/agile.ts work <name>
```

Analyzes an issue and provides stage-appropriate guidance for working through it.

**Arguments:**
- `name`: Issue name to analyze

**Output includes:**
- Issue metadata (type, stage, owner, created date)
- Incomplete sections that still have placeholder text
- Definition of Done progress
- Stage-specific guidance and suggested actions
- Readiness status for advancing to the next stage

**Examples:**
```bash
bun .claude/skills/agile/agile.ts work user-auth
bun .claude/skills/agile/agile.ts work dark-mode
```

### List Issues

```bash
bun .claude/skills/agile/agile.ts list [--stage <stage>] [--format table|json]
```

**Options:**
- `--stage, -s`: Filter by stage
- `--format, -f`: Output format (`table` or `json`, default: `table`)

**Examples:**
```bash
bun .claude/skills/agile/agile.ts list
bun .claude/skills/agile/agile.ts list --stage 3-in-progress
bun .claude/skills/agile/agile.ts list --format json
```

### Archive an Issue

```bash
bun .claude/skills/agile/agile.ts archive <name>
```

Moves the issue to `agile/archive/` with a timestamp suffix.

**Example:**
```bash
bun .claude/skills/agile/agile.ts archive completed-feature
# Result: agile/archive/completed-feature-2025-11-25.md
```

### Delete an Issue

```bash
bun .claude/skills/agile/agile.ts delete <name>
```

Permanently removes the issue file.

**Example:**
```bash
bun .claude/skills/agile/agile.ts delete old-task
```

### Get Help

```bash
bun .claude/skills/agile/agile.ts help [command]
```

## Typical Workflow

```bash
# 1. Create a new feature with owner
bun .claude/skills/agile/agile.ts create feature dark-mode --title "Dark Mode Toggle" --owner "Alice"

# 2. Start working on it
bun .claude/skills/agile/agile.ts move dark-mode 2-todo
bun .claude/skills/agile/agile.ts move dark-mode 3-in-progress

# 3. Submit for review
bun .claude/skills/agile/agile.ts move dark-mode 4-review

# 4. Complete the work
bun .claude/skills/agile/agile.ts move dark-mode 5-done

# 5. Archive when no longer needed
bun .claude/skills/agile/agile.ts archive dark-mode
```

## Issue Types

- **feature**: New features and enhancements
- **bug**: Bug reports with reproduction steps
- **task**: General tasks and chores

## Naming Convention

Issue names must be in kebab-case: lowercase letters, numbers, and hyphens only.

Valid: `user-auth`, `fix-login-bug`, `update-deps-2024`
Invalid: `UserAuth`, `fix_login`, `update deps`

## Workflow Agent

A specialized `agile-workflow` agent is available to help work through issues interactively. The agent:

1. Analyzes the issue's current state using the `work` command
2. Provides stage-appropriate guidance based on the workflow stage
3. Helps populate incomplete sections through conversation
4. Validates readiness before advancing to the next stage
5. Handles move failures gracefully with guidance on what's missing

### Stage-Specific Guidance

| Stage | Focus | Key Actions |
|-------|-------|-------------|
| **1-backlog** | Define clearly | Populate description, write acceptance criteria |
| **2-todo** | Prepare for work | Add technical notes, assign owner |
| **3-in-progress** | Support development | Track progress, update notes |
| **4-review** | Ensure quality | Walk through DoD checklist |
| **5-done** | Wrap up | Archive or identify follow-up |

### Usage

The workflow agent is automatically available. When asked to "work on" an issue, it will analyze and guide you through the appropriate stage.
