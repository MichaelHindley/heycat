# Development

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== DEVELOPMENT DOCS LOADED ===".


## Commands

```bash
# Start development mode (runs both frontend and Tauri)
bun run tauri dev

# Build production app
bun run tauri build

# Run frontend only (Vite dev server)
bun run dev

# Type check and build frontend
bun run build
```

## Prerequisites

```bash
# Required for Rust coverage (commits will be blocked without it)
rustup toolchain install nightly
cargo install cargo-llvm-cov
```

## macOS Accessibility Permission

The app requires Accessibility permission for global hotkey capture. In **debug builds**, the permission prompt is automatically suppressed to avoid interrupting test/build workflows.

| Build Type | Prompts? |
|------------|----------|
| `cargo test` | No |
| `cargo build` | No |
| `bun tauri dev` | No |
| `bun tauri build` (release) | Yes |

To test the first-run permission UX in dev mode:

```bash
HEYCAT_ACCESSIBILITY_PROMPT=1 bun tauri dev
```

## Worktrees

heycat supports git worktrees for parallel feature development. Each worktree gets isolated configuration and data directories, allowing multiple instances to run simultaneously with different hotkeys.

### Create a worktree

```bash
# Creates worktree at worktrees/heycat-<branch> with unique hotkey
bun scripts/create-worktree.ts <branch-name>

# Or specify custom path
bun scripts/create-worktree.ts <branch-name> <path>
```

Example:
```bash
bun scripts/create-worktree.ts feature-audio
# Creates worktrees/heycat-feature-audio with branch feature-audio
# Assigns unique hotkey (e.g., CmdOrControl+Shift+3)
```

Then:
```bash
cd worktrees/heycat-feature-audio
bun install
bun run tauri dev
```

### How isolation works

| Data | Main Repo | Worktree |
|------|-----------|----------|
| Dev port | 1420 | 1421-1429 (deterministic) |
| Hotkey | Default | Unique per worktree |
| Settings | `settings.json` | `settings-{id}.json` |
| Data dir | `~/.local/share/heycat/` | `~/.local/share/heycat-{id}/` |
| Config dir | `~/.config/heycat/` | `~/.config/heycat-{id}/` |

The `{id}` is the worktree directory name (e.g., `heycat-feature-audio`). Port and hotkey are assigned deterministically based on the identifier hash, so they're consistent across sessions.

### Clean up worktree data

When you remove a git worktree, the data directories remain. Use the cleanup script:

```bash
# List all worktree data directories
bun scripts/cleanup-worktree.ts --list

# Clean up orphaned data (worktrees that no longer exist)
bun scripts/cleanup-worktree.ts --orphaned

# Clean up specific worktree
bun scripts/cleanup-worktree.ts <identifier>

# Also remove the git worktree itself
bun scripts/cleanup-worktree.ts <identifier> --remove-worktree
```

### Run multiple instances

Each worktree uses a different dev server port and recording hotkey, so you can run the main repo and worktrees simultaneously:

```bash
# Terminal 1: main repo
bun run tauri dev   # Uses port 1420, default hotkey

# Terminal 2: worktree
cd worktrees/heycat-feature-audio
bun run tauri dev   # Uses port 1421-1429, unique hotkey
```

The wrapper script automatically detects the worktree context and configures both Vite and Tauri with the correct port.

### Cattle worktree workflow (PR-based)

Worktrees are ephemeral - created per-feature, deleted after PR merge:

```
/create-worktree → develop → /submit-pr → (review) → /close-worktree
```

**1. Create worktree** (from main repo):
```bash
# Via command (recommended for agents)
/create-worktree

# Or manually
bun scripts/create-worktree.ts <branch-name>
```

**2. Develop feature**:
- Make changes, commit with WIP messages
- Use `/sync-worktree` to rebase on latest main if needed

**3. Submit PR** (from worktree):
```bash
/submit-pr
```
This pushes the branch and creates a PR via `gh pr create`.

**4. Review phase**:
- Worktree stays alive for fixes
- Push additional commits if changes requested

**5. Close worktree** (after PR merged):
```bash
/close-worktree
```
This deletes the worktree and cleans up all data directories.

### Legacy: Direct merge to main

The `complete-feature.ts` script still exists for direct merging but is deprecated in favor of the PR workflow above.
