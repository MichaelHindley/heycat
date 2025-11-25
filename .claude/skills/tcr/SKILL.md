---
name: tcr
description: |
  Test-Commit-Refactor (TCR) workflow automation. Use this skill to enforce test discipline:
  - Automatically runs tests when todos are marked complete
  - Blocks commits if tests are failing
  - Auto-commits WIP when tests pass
  - Tracks failures and prompts after threshold
---

# TCR (Test-Commit-Refactor) Skill

Enforces the Test-Commit-Refactor workflow through Claude Code hooks.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     TCR Workflow Loop                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Write a failing test (start with red)                   │
│                    ↓                                         │
│  2. Write code to make the test pass                        │
│                    ↓                                         │
│  3. Mark todo as "completed"                                │
│                    ↓                                         │
│  ┌──────────────────────────────────────┐                   │
│  │  TCR Hook: PostToolUse on TodoWrite  │                   │
│  │  - Get changed files (git diff)      │                   │
│  │  - Find related tests                │                   │
│  │  - Run tests                         │                   │
│  └──────────────────────────────────────┘                   │
│                    ↓                                         │
│         ┌─────────┴─────────┐                               │
│         │                   │                               │
│    Tests Pass          Tests Fail                           │
│         │                   │                               │
│    Auto-commit         Increment                            │
│    WIP commit          failure count                        │
│         │                   │                               │
│         ↓                   ↓                               │
│    Next task         Continue refactoring                   │
│                      (loop back to step 2)                  │
│                                                              │
│  After 5 failures: Prompt to reconsider approach            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Pre-Commit Guardrail

When you try to `git commit`, TCR intercepts and runs tests:
- **Tests pass** → Commit proceeds
- **Tests fail** → Commit blocked with error message

This ensures you can't commit code that breaks tests.

## Commands

### Run Tests Manually

```bash
bun .claude/skills/tcr/tcr.ts run <files...>
```

Run tests for specific source files.

**Examples:**
```bash
bun .claude/skills/tcr/tcr.ts run src/App.tsx
bun .claude/skills/tcr/tcr.ts run src/utils/auth.ts src/utils/validation.ts
```

### Check Status

```bash
bun .claude/skills/tcr/tcr.ts status
```

Shows:
- Current step being worked on
- Failure count (visual bar)
- Last test result and timestamp

### Reset Failure Counter

```bash
bun .claude/skills/tcr/tcr.ts reset
```

Use when you want to continue past the 5-failure threshold.

### Get Help

```bash
bun .claude/skills/tcr/tcr.ts help [command]
```

## Test Discovery

Tests are found using convention-based mapping:

| Source File | Test File |
|-------------|-----------|
| `src/foo.ts` | `src/foo.test.ts` or `src/foo.spec.ts` |
| `src/bar.tsx` | `src/bar.test.tsx` or `src/bar.spec.tsx` |
| `src-tauri/*.rs` | `cargo test` (module-based) |

## Test Runners

- **Frontend**: `bun test` for TypeScript/JavaScript files
- **Backend**: `cargo test` for Rust files in `src-tauri/`

The target is automatically detected based on which files changed.

## State File

TCR stores state in `.tcr-state.json` at project root:

```json
{
  "currentStep": "add-user-authentication",
  "failureCount": 2,
  "lastTestResult": {
    "passed": false,
    "timestamp": "2025-11-25T10:15:00Z",
    "error": "Expected true, got false",
    "filesRun": ["src/auth.test.ts"],
    "target": "frontend"
  }
}
```

**Add to `.gitignore`:**
```
.tcr-state.json
```

## Failure Threshold

After 5 consecutive failures on the same task, TCR prompts you to:

1. Break down the task into smaller pieces
2. Review the test expectations
3. Take a different approach
4. Run `tcr reset` to continue

## Hook Configuration

The skill requires hooks to be configured in `.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "TodoWrite",
        "hooks": [{
          "type": "command",
          "command": "bun .claude/skills/tcr/tcr.ts hook-todo-complete"
        }]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{
          "type": "command",
          "command": "bun .claude/skills/tcr/tcr.ts hook-pre-commit"
        }]
      }
    ]
  }
}
```

## Prerequisites

- Test infrastructure must be set up (Bun test for frontend, Cargo test for backend)
- Tests should follow naming convention (`*.test.ts` or `*.spec.ts`)

## Tips

1. **Start with a failing test** - Write the test first, watch it fail
2. **Small steps** - Each todo should be a small, testable change
3. **Trust the loop** - Let TCR handle commits; focus on making tests pass
4. **Reset wisely** - If you hit 5 failures, consider if the approach needs rethinking
