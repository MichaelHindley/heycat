---
name: agile
description: "Manage agile workflow with folder-based issues. Use when user wants to: create features/bugs/tasks, move through kanban stages, break down into specs, list/archive/delete issues, continue with next spec, check issue status/progress, resume work on an issue, or see what's next."
---

# Agile Workflow Management

Manage issues in the project's Kanban board located in the `agile/` folder. Each issue is a **folder** containing a main spec, technical guidance, and multiple SPS (Smallest Possible Spec) files.

## Issue Structure

```
agile/<stage>/<issue-name>/
  - feature.md (or bug.md/task.md)  # Main issue spec
  - technical-guidance.md            # Technical investigation
  - *.spec.md                        # SPS spec files
```

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

Creates a folder-based issue with main spec and technical guidance files.

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
```

### Move an Issue

```bash
bun .claude/skills/agile/agile.ts move <name> <stage>
```

**Transition Requirements (Strict Validation):**

| Target Stage | Requirements |
|--------------|--------------|
| `2-todo` | Description section must be complete (no placeholders) |
| `3-in-progress` | Owner assigned, Technical guidance exists |
| `4-review` | All specs completed, Guidance updated, >=1 DoD checked |
| `5-done` | All Definition of Done items must be checked |

### Work on an Issue

```bash
bun .claude/skills/agile/agile.ts work <name>
```

Analyzes an issue and provides stage-appropriate guidance, including:
- Specs status (pending, in-progress, completed)
- Technical guidance status
- Definition of Done progress
- Readiness to advance

### List Issues

```bash
bun .claude/skills/agile/agile.ts list [--stage <stage>] [--format table|json]
```

Shows all issues with spec progress (e.g., `[3/5 specs]`).

### Archive / Delete

```bash
bun .claude/skills/agile/agile.ts archive <name>  # Move folder to archive
bun .claude/skills/agile/agile.ts delete <name>   # Permanently delete folder
```

## Spec Commands

Manage SPS (Smallest Possible Spec) files within an issue.

### List Specs

```bash
bun .claude/skills/agile/agile.ts spec list <issue>
```

### Add a Spec

```bash
bun .claude/skills/agile/agile.ts spec add <issue> <name> [--title "Title"]
```

### Update Spec Status

```bash
bun .claude/skills/agile/agile.ts spec status <issue> <spec> <pending|in-progress|in-review|completed>
```

**Spec Status Lifecycle:**
```
pending → in-progress → in-review → [APPROVED] → completed
               ↑            ↓
               └──── [NEEDS_WORK] (via /agile:fix)
```

**Transition Rules:**
- `in-progress` → `in-review`: Always allowed (ready for review)
- `in-review` → `completed`: Requires APPROVED verdict in review section
- `in-review` → `in-progress`: Requires NEEDS_WORK verdict (via `/agile:fix`)
- `completed` → `in-review`: Always allowed (re-review)

**Note:** The `review_round` field in spec frontmatter tracks how many review cycles a spec has gone through.

### Delete a Spec

```bash
bun .claude/skills/agile/agile.ts spec delete <issue> <spec>
```

### Suggest Specs (AI-Assisted)

```bash
bun .claude/skills/agile/agile.ts spec suggest <issue>
```

The agent will analyze the issue description and suggest a breakdown into specs following the SPS pattern.

## Guidance Commands

Manage technical guidance for an issue.

### Show Guidance

```bash
bun .claude/skills/agile/agile.ts guidance show <issue>
```

### Mark as Updated

```bash
bun .claude/skills/agile/agile.ts guidance update <issue>
```

Sets the last-updated timestamp. Required before completing specs.

### Validate Guidance

```bash
bun .claude/skills/agile/agile.ts guidance validate <issue>
```

Checks if guidance needs update (completed specs since last update).

### Set Status

```bash
bun .claude/skills/agile/agile.ts guidance status <issue> <draft|active|finalized>
```

## Review Commands

Review specs to verify implementation and code quality. Supports iterative review cycles.

### Review Spec in In-Review Status

```bash
bun .claude/skills/agile/agile.ts review
```

Automatically finds and reviews a spec in `in-review` status:

1. Finds the issue in `3-in-progress` (must be exactly one)
2. Identifies specs with `in-review` status
3. Uses git history to find the most recently modified in-review spec
4. Outputs a review prompt for Claude to execute

**Review covers:**
- **Implementation Verification**: Checks each acceptance criterion against actual code
- **Code Quality Audit**: Patterns, error handling, test coverage

**Output:**
Claude reads the prompt and:
- Examines implementation files referenced in the spec
- Verifies acceptance criteria are met
- Checks test coverage matches test cases
- Appends a `## Review` section to the spec file with verdict (APPROVED/NEEDS_WORK)

**After Review:**
- If APPROVED: Run `spec status <issue> <spec> completed`
- If NEEDS_WORK: Run `/agile:fix` to address feedback

### Fix Failed Review

```bash
bun .claude/skills/agile/agile.ts fix
```

Handles specs that received a NEEDS_WORK verdict:

1. Finds spec in `in-review` status with NEEDS_WORK verdict
2. Parses the review section to extract specific issues:
   - Failed acceptance criteria with evidence
   - Missing test coverage
   - Code quality concerns
3. Displays a structured fix guide
4. Transitions spec back to `in-progress` for fixes
5. Records review history in spec frontmatter

**After Fixing:**
Run `/agile:review` for another independent review round.

**Review History:**
Each review round is tracked in the spec's frontmatter:
- `review_round`: Current review iteration number
- `review_history`: Array of past review verdicts and issues

## SPS Pattern (Smallest Possible Spec)

Each spec should be the **smallest deliverable unit** - roughly the size of one "todo" item. Specs contain:

- **Title** and **Description**: What this spec accomplishes
- **Acceptance Criteria**: Specific, testable criteria
- **Test Cases**: Expected behaviors to verify
- **Dependencies**: Other specs that must complete first
- **Preconditions**: System state required
- **Implementation Notes**: Technical details (updated during work)
- **Related Specs**: Links to related work

## Typical Workflow

```bash
# 1. Create a new feature
bun .claude/skills/agile/agile.ts create feature dark-mode --title "Dark Mode Toggle" --owner "Alice"

# 2. Fill in description, then move to todo
bun .claude/skills/agile/agile.ts move dark-mode 2-todo

# 3. Generate spec breakdown
bun .claude/skills/agile/agile.ts spec suggest dark-mode
# Agent suggests specs, you approve/edit

# 4. Move to in-progress
bun .claude/skills/agile/agile.ts move dark-mode 3-in-progress

# 5. Work through specs one at a time
bun .claude/skills/agile/agile.ts spec status dark-mode ui-toggle in-progress
# ... implement ...
bun .claude/skills/agile/agile.ts guidance update dark-mode
bun .claude/skills/agile/agile.ts spec status dark-mode ui-toggle in-review

# 5b. Review the spec (using independent subagent)
bun .claude/skills/agile/agile.ts review
# Subagent verifies implementation and appends review section

# 5c. Handle review verdict
# If APPROVED:
bun .claude/skills/agile/agile.ts spec status dark-mode ui-toggle completed

# If NEEDS_WORK:
bun .claude/skills/agile/agile.ts fix
# ... fix issues based on feedback ...
# Run /agile:review again for re-review

# 6. Complete all specs, move to review
bun .claude/skills/agile/agile.ts move dark-mode 4-review

# 7. Complete DoD items, move to done
bun .claude/skills/agile/agile.ts move dark-mode 5-done

# 8. Archive when no longer needed
bun .claude/skills/agile/agile.ts archive dark-mode
```

## Stage-Specific Guidance

| Stage | Focus | Key Actions |
|-------|-------|-------------|
| **1-backlog** | Define clearly | Populate description, write acceptance criteria |
| **2-todo** | Break into specs | Run `spec suggest`, assign owner, update guidance |
| **3-in-progress** | Work through specs | Complete specs one at a time, update guidance |
| **4-review** | Ensure quality | Walk through DoD checklist, verify guidance |
| **5-done** | Wrap up | Archive or identify follow-up |

