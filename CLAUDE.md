# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Development Commands

```bash
# Start development mode (runs both frontend and Tauri)
bun run tauri dev

# Build production app
bun run tauri build

# Run frontend only (Vite dev server on port 1420)
bun run dev

# Type check and build frontend
bun run build
```

## Architecture

### Frontend (src/)
- React 18 with TypeScript
- Vite bundler (port 1420 for dev)
- Entry point: `src/main.tsx` → `src/App.tsx`
- Communicates with Rust backend via `invoke()` from `@tauri-apps/api/core`

### Backend (src-tauri/)
- Rust with Tauri v2
- Entry point: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- Tauri commands are defined with `#[tauri::command]` attribute and registered in `invoke_handler()`
- Config: `src-tauri/tauri.conf.json`

### Frontend-Backend Communication
```typescript
// Frontend: call Rust command
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { arg1, arg2 });
```

```rust
// Backend: define command in lib.rs
#[tauri::command]
fn command_name(arg1: &str, arg2: i32) -> String {
    // implementation
}
// Register in invoke_handler: tauri::generate_handler![command_name]
```

## Agile Workflow

Project uses a Kanban-style issue tracking system in `/agile`.

**When to use:** Invoke the `agile` skill when the user wants to:
- Create a new feature, bug, or task
- Work on or refine an existing issue
- Move issues through workflow stages
- List, archive, or delete issues

The skill provides full documentation and CLI commands. Transitions require complete content (description, owner, DoD items).

### Agile Skill Triggers

**ALWAYS invoke the `agile` skill** when the user says any of these (or similar):
- "continue with the next spec" / "next spec" / "what's next"
- "work on the issue" / "resume work" / "pick up where I left off"
- "issue status" / "current progress" / "where are we"
- "create a feature/bug/task"
- "move the issue" / "advance to next stage"
- "list issues" / "show backlog"

The skill's `work <name>` command provides stage-appropriate guidance including spec status.

**Shortcut commands:**
- `/agile:next` - Auto-find and implement the next spec in the in-progress issue
- `/agile:status` - Show current issue status and progress
- `/agile:review` - Run independent code review using a fresh subagent

### Review Independence

**NEVER self-review your own implementation.** When you implement a spec:
- DO NOT add a "## Review" section
- DO NOT mark acceptance criteria as verified
- DO NOT update spec status to "completed"

Reviews must be performed by a **fresh subagent** with no implementation context. Use `/agile:review` which launches an independent code-reviewer agent.

## TCR (Test-Commit-Refactor) Workflow

The TCR skill enforces test discipline through explicit triggers and pre-commit enforcement:

### Development Workflow (True TDD)
1. **Write a failing test first** (red) - the test defines what you're building
2. **Iterate on production code** (non-test files) to make the test pass
3. **Run `tcr check`** when you believe the test should now pass
4. **Tests pass** → WIP commit created, refactor if needed
5. **Tests fail** → continue iterating, run `tcr check` again (after 5 failures, reconsider approach)

**Key principle:** The failing test IS your scope. It guides development and defines what "done" means. Run `tcr check` when you think you've solved it.

**Smaller increments:** Write smaller, more focused tests - not fewer file changes. One test = one behavior.

### Pre-Commit Enforcement (Husky)
- Husky runs both **frontend** and **backend** tests with coverage before every commit
- **100% coverage required** for both frontend and backend
- Untestable code must be explicitly excluded (see Coverage Exclusions below)
- Commits blocked if tests fail or coverage is insufficient

### Coverage Exclusions

Both frontend and backend require 100% coverage. Use inline exclusion comments for untestable code:

#### Frontend (TypeScript/React)

Uses Vitest with `/* v8 ignore ... -- @preserve */` comments. The `-- @preserve` is required because esbuild strips comments during transpilation:

```typescript
/* v8 ignore next -- @preserve */
setGreetMsg(await invoke("greet", { name })); // Single line

/* v8 ignore start -- @preserve */
// Multiple lines excluded
ReactDOM.createRoot(document.getElementById("root")).render(<App />);
/* v8 ignore stop */

/* v8 ignore file -- @preserve */
// At top of file to ignore entire file
```

#### Backend (Rust)

Uses `#[coverage(off)]` attribute (requires nightly):

```rust
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn untestable_function() {
    // ...
}
```

### Prerequisites

```bash
# Required for Rust coverage (commits will be blocked without it)
rustup toolchain install nightly
cargo install cargo-llvm-cov
```

### Commands

```bash
bun .claude/skills/tcr/tcr.ts check [step-name] [-v] # Run tests and auto-commit on success (primary)
bun .claude/skills/tcr/tcr.ts run <files>            # Run tests for specific files
bun .claude/skills/tcr/tcr.ts status                 # Show current state
bun .claude/skills/tcr/tcr.ts status --coverage      # Show state with coverage metrics
bun .claude/skills/tcr/tcr.ts coverage               # Run coverage on changed files
bun .claude/skills/tcr/tcr.ts coverage --debug       # Show detailed per-file coverage
bun .claude/skills/tcr/tcr.ts verify-config          # Verify coverage thresholds are in sync
bun .claude/skills/tcr/tcr.ts reset                  # Reset failure counter and clear error log
bun .claude/skills/tcr/tcr.ts help                   # Show help message
```

**Output optimization:** The `check` command uses condensed output by default (saves context window). Use `-v` or `--verbose` for full test output when debugging.

**Error persistence:** When tests fail, full output is saved to `.tcr-state.json` (first 5KB in `output.truncated`). If output exceeds 10KB, chunks are saved to `.tcr/output/` with paths in the state file.

### Test Discovery
- **Frontend**: Convention-based (`foo.ts` → `foo.test.ts` or `foo.spec.ts`)
- **Backend**: Rust tests in `#[cfg(test)]` modules (`src-tauri/src/*.rs`)
- **Backend test files**: `*_test.rs` files are automatically excluded from coverage

## Integration Verification

For multi-component features:

1. **Mock Usage Audit**: When reviewing specs with mocked dependencies, verify the mocked component is actually instantiated in production code (lib.rs, main.tsx, etc.)

2. **Deferral Tracking**: Any comment like "handled separately", "will be implemented later", or "managed elsewhere" MUST reference a specific spec or ticket. Flag as NEEDS_WORK if no reference exists.

3. **Final Integration Spec**: Multi-component features require a final "integration" spec that:
   - Verifies all components are wired together in production
   - Includes an integration test (automated)
   - Documents the end-to-end flow with file:line references

4. **Feature Completion Gate**: Before moving to 4-review:
   - All "handled separately" comments must have corresponding completed specs
   - Integration test must exist and pass
