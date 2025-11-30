---
name: tcr
description: "TCR workflow automation enforcing test discipline. Run 'tcr check' to test and auto-commit on success. Use for test-first development with 100% coverage enforcement."
---

# TCR (Test-Commit-Refactor) Skill

Enforces the Test-Commit-Refactor workflow with explicit test triggers and pre-commit enforcement.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     TCR Workflow Loop                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Write a failing test (the test IS your scope)           │
│                    ↓                                         │
│  2. Iterate on production code to make it pass              │
│                    ↓                                         │
│  3. Run "tcr check" when you think test should pass         │
│                    ↓                                         │
│  ┌──────────────────────────────────────┐                   │
│  │  TCR Check Command                   │                   │
│  │  - Get changed files (git status)    │                   │
│  │  - Find related tests                │                   │
│  │  - Run tests with coverage           │                   │
│  │  - Output: condensed (or --verbose)  │                   │
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
│    Next task           Fix tests,                           │
│                      run "tcr check"                        │
│                           again                             │
│                                                              │
│  After 5 failures: Prompt to reconsider approach            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Pre-Commit Guardrail (Husky)

Pre-commit enforcement is handled by Husky (`.husky/pre-commit`):
- **Frontend**: Runs `bun run test:coverage` (Vitest with 100% thresholds)
- **Backend**: Runs `cargo +nightly llvm-cov --fail-under-lines 100 --fail-under-functions 100`
- Blocks commit if tests fail or coverage is insufficient

This is repository-level enforcement that applies to all contributors.

## Coverage Exclusions

Use inline comments to exclude untestable code:

**Frontend (TypeScript):**
```typescript
/* v8 ignore next -- @preserve */
await invoke("greet", { name }); // Tauri runtime required

/* v8 ignore start -- @preserve */
// Block of untestable code
/* v8 ignore stop */

/* v8 ignore file -- @preserve */
// At top of file to ignore entire file
```

Note: `-- @preserve` is required because esbuild strips comments during transpilation.

**Backend (Rust):**
```rust
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn untestable_function() { ... }
```

**Backend Test Files (`*_test.rs`):**

Separate test files (e.g., `mod_test.rs`) are automatically excluded from coverage measurement via `--ignore-filename-regex '_test\.rs$'`. No manual annotation needed for these files.

## Commands

### Check (Primary Command)

```bash
bun .claude/skills/tcr/tcr.ts check [step-name] [--verbose]
```

Run tests on changed files and auto-commit on success. This is the main command for the TCR workflow.

**Options:**
- `--verbose`, `-v`: Show full test output (default: condensed on success)

**Examples:**
```bash
bun .claude/skills/tcr/tcr.ts check                    # Uses "manual check" as step name
bun .claude/skills/tcr/tcr.ts check "Add user auth"    # Custom step name for WIP commit
bun .claude/skills/tcr/tcr.ts check --verbose "Debug"  # Show full output for debugging
```

**Behavior:**
1. Detects changed files via `git status -s`
2. Determines target (frontend/backend/both) based on file paths
3. Runs tests with coverage
4. On success: shows condensed output, creates WIP commit
5. On failure: saves full output to `.tcr-state.json`, shows condensed error details, increments failure counter, exits with code 2

**Output (default condensed mode):**
```
TCR: Running check - "Add user auth"
TCR: Found 2 changed file(s)
TCR: Running frontend tests (1 file(s))...
TCR: PASS: Frontend: PASS | Backend: PASS (100%)
TCR: Committed (abc1234)
```

**Failure output:**
```
TCR: Running check - "Add user auth"
TCR: Found 2 changed file(s)
TCR: Running backend tests (1 file(s))...
TCR: Tests failed (2/5) - details saved to .tcr-state.json
=== Backend Failures ===
(condensed error details)
```

When tests fail, full output is persisted to `.tcr-state.json` for debugging. If output exceeds 10KB, chunks are saved to `.tcr/output/` with paths in the state file.

Use `--verbose` to see full vitest/cargo output inline instead of checking the state file.

### Run Tests for Specific Files

```bash
bun .claude/skills/tcr/tcr.ts run <files...>
```

Run tests for specific source files (without auto-committing).

**Examples:**
```bash
bun .claude/skills/tcr/tcr.ts run src/App.tsx
bun .claude/skills/tcr/tcr.ts run src/utils/auth.ts src/utils/validation.ts
```

### Check Status

```bash
bun .claude/skills/tcr/tcr.ts status
bun .claude/skills/tcr/tcr.ts status --coverage  # Include live coverage metrics
```

Shows:
- Current step being worked on
- Failure count (visual bar)
- Last test result and timestamp
- Recent errors from `.tcr-errors.log` (if any)

Use `--coverage` or `-c` to also display current coverage metrics (runs tests).

### Reset Failure Counter

```bash
bun .claude/skills/tcr/tcr.ts reset
```

Use when you want to continue past the 5-failure threshold. Also clears the error log.

### Coverage Commands

```bash
bun .claude/skills/tcr/tcr.ts coverage              # Run coverage on changed files
bun .claude/skills/tcr/tcr.ts coverage --debug      # Show detailed per-file output
```

Run coverage checks on changed files (same behavior as `tcr check`). Target (frontend/backend/both) is auto-detected from changed file paths.

Use `--debug` or `-d` to see detailed per-file coverage output, which is helpful for identifying exactly which lines/functions are missing coverage.

### Verify Configuration Sync

```bash
bun .claude/skills/tcr/tcr.ts verify-config
```

Checks that coverage thresholds are consistent across all three configuration files. Exits with code 1 if mismatches are found.

### Get Help

```bash
bun .claude/skills/tcr/tcr.ts help [command]
```

## Test Discovery

**Frontend:** Convention-based mapping

| Source File | Test File |
|-------------|-----------|
| `src/foo.ts` | `src/foo.test.ts` or `src/foo.spec.ts` |
| `src/bar.tsx` | `src/bar.test.tsx` or `src/bar.spec.tsx` |

**Backend:** Module-based filtering (supports both inline tests and separate test files)

| Source File | Test Filter |
|-------------|-------------|
| `src-tauri/src/lib.rs` | `tests::` (crate root) |
| `src-tauri/src/main.rs` | `tests::` (crate root) |
| `src-tauri/src/foo.rs` | `foo::foo_test` (if `foo_test.rs` exists) or `foo::tests::` |
| `src-tauri/src/bar/mod.rs` | `bar::tests::` |
| `src-tauri/src/bar/baz.rs` | `bar::baz_test` (if `baz_test.rs` exists) or `bar::baz::tests::` |
| `src-tauri/src/bar/baz_test.rs` | `bar::baz_test` (tests directly in test module) |

Backend tests can be either:
- **Inline tests:** `#[cfg(test)] mod tests { }` blocks within source files
- **Separate test files:** `*_test.rs` files (e.g., `foo_test.rs` tests `foo.rs`)

**Note:** If no test files are found for changed frontend files, the hook warns and exits without auto-committing. Write tests first, or commit manually.

## Test Runners

- **Frontend**: Vitest with v8 coverage (via `bun run test:coverage`)
- **Backend**: cargo-llvm-cov with nightly toolchain (via `cargo +nightly llvm-cov`)

The target is automatically detected based on which files changed.

## State Files

TCR stores state in files at project root:

### `.tcr-state.json` - Main State

```json
{
  "currentStep": "add-user-authentication",
  "failureCount": 2,
  "lastTestResult": {
    "passed": false,
    "timestamp": "2025-11-25T10:15:00Z",
    "error": "Coverage below threshold",
    "filesRun": ["src/auth.test.ts"],
    "target": "frontend",
    "output": {
      "truncated": "First 5KB of test output...",
      "fullChunks": [".tcr/output/chunk-1732...-0.txt"],
      "totalSize": 15000
    }
  }
}
```

The `output` field captures full test output for debugging:
- `truncated`: First 5KB of output (always stored in state file)
- `fullChunks`: Paths to chunk files if output > 10KB (null if under threshold)
- `totalSize`: Total output size in bytes

### `.tcr/` - Output Overflow Directory

When test output exceeds 10KB, full output is split into 5KB chunks:

```
.tcr/
  output/
    chunk-{timestamp}-0.txt
    chunk-{timestamp}-1.txt
    ...
```

Chunk paths are stored in `.tcr-state.json` under `lastTestResult.output.fullChunks`.

### `.tcr-errors.log` - Error Log

Persists hook errors that would otherwise only appear in console. Shown by `tcr status` and cleared by `tcr reset`.

**Add to `.gitignore`:**
```
.tcr-state.json
.tcr-errors.log
.tcr/
```

## Failure Threshold

After 5 consecutive failures on the same task, TCR prompts you to:

1. Break down the task into smaller pieces
2. Review the test expectations
3. Take a different approach
4. Run `tcr reset` to continue

## Hook Configuration (Optional)

The TodoWrite auto-trigger hook is **disabled by default**. This is intentional for TDD discipline:
- Tests should run when you **expect them to pass**, not on arbitrary events
- Todo completion doesn't mean "ready to test" - you run `tcr check` when you believe the failing test should now pass
- Manual triggering gives control over the red-green-refactor loop

To re-enable automatic testing on todo completion, add this to `.claude/settings.json`:

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
    ]
  }
}
```

Pre-commit enforcement is handled separately by Husky (see above) and always active.

## Prerequisites

**Frontend:**
- Vitest configured with coverage (`vitest.config.ts`)
- Tests follow naming convention (`*.test.ts` or `*.spec.ts`)

**Backend:**
- Rust nightly toolchain: `rustup toolchain install nightly`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`
- Tests in `#[cfg(test)] mod tests { }` blocks

## Tips

1. **The test IS your scope** - The failing test defines what you're building. It guides development and tells you when you're done.
2. **Smaller tests, not fewer files** - "Small increments" means writing focused tests for one behavior at a time, not limiting how many files you touch.
3. **Run `tcr check` when you expect green** - Don't run tests arbitrarily. Run them when you believe the failing test should now pass.
4. **Trust the loop** - Let TCR handle commits; focus on making tests pass
5. **Reset wisely** - If you hit 5 failures, the test might be too big. Consider breaking it into smaller behaviors.

## Mock Awareness

When your tests use mocks (MockBackend, MockFileWriter, etc.), remember:

- **Mocks prove the interface contract works** - They validate that your code correctly uses the interface
- **Mocks do NOT prove production code uses the interface** - A passing mock test doesn't mean the real implementation is instantiated
- **After completing a mocked component, verify it's instantiated** - Check lib.rs, main.tsx, or equivalent entry point
- **Integration tests complement unit tests** - Unit tests with mocks verify behavior; integration tests verify wiring

### Warning Signs

If you see these patterns, pause and verify integration:
- Multiple `MockXxx` implementations in test files
- Comments like "handled separately" or "managed elsewhere"
- Tests passing but no production instantiation of the tested component
- Coverage at 100% but feature doesn't work

## Maintenance Notes

### Coverage Configuration Sync

Coverage thresholds are enforced in **three separate locations** that must stay in sync:

1. `.claude/skills/tcr/lib/coverage/config.ts` - TCR status display and reporting
2. `vitest.config.ts` - Frontend thresholds (coverage.thresholds)
3. `.husky/pre-commit` - Backend thresholds (--fail-under-lines/--fail-under-functions)

**If you change coverage thresholds, update all three files!**

### Exit Code Behavior

The TCR hook uses Claude Code's exit code system to enforce test discipline:

| Scenario | Exit Code | Effect |
|----------|-----------|--------|
| Tests pass | 0 | Agent continues, WIP commit created |
| Tests fail | 2 | Agent BLOCKED, must fix tests |
| No test files found | 0 | Agent continues, no auto-commit (warning shown) |
| Hook runtime error | 0 | Agent continues (fail-open), error logged |

**Test failures block the agent** - Claude receives the error via stderr and must respond to it before continuing. This enforces TCR discipline.

**Hook runtime errors are fail-open** - If the hook itself crashes (can't parse input, can't run tests), it exits with code 0 so bugs in the hook don't block all work. Errors are logged to `.tcr-errors.log` for inspection via `tcr status`.

### Known Limitations

- **Frontend test discovery**: Uses convention-based mapping (foo.ts → foo.test.ts). Non-standard test file locations won't be auto-discovered.

- **Backend test filtering**: Uses module-based filtering derived from file paths. Tests must be in `#[cfg(test)] mod tests { }` blocks within each source file.

- **Coverage configuration sync**: Thresholds are defined in three places. Use `tcr verify-config` to check they're in sync.
