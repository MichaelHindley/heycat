# Development

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== DEVELOPMENT DOCS LOADED ===".


## Commands

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
# Creates worktree at ../heycat-<branch> with unique hotkey
bun scripts/create-worktree.ts <branch-name>

# Or specify custom path
bun scripts/create-worktree.ts <branch-name> <path>
```

Example:
```bash
bun scripts/create-worktree.ts feature-audio
# Creates ../heycat-feature-audio with branch feature-audio
# Assigns unique hotkey (e.g., CmdOrControl+Shift+3)
```

Then:
```bash
cd ../heycat-feature-audio
bun install
bun run tauri dev
```

### How isolation works

| Data | Main Repo | Worktree |
|------|-----------|----------|
| Settings | `settings.json` | `settings-{id}.json` |
| Data dir | `~/.local/share/heycat/` | `~/.local/share/heycat-{id}/` |
| Config dir | `~/.config/heycat/` | `~/.config/heycat-{id}/` |

The `{id}` is the worktree directory name (e.g., `heycat-feature-audio`).

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

Each worktree uses a different recording hotkey, so you can run the main repo and worktrees simultaneously:

```bash
# Terminal 1: main repo
bun run tauri dev   # Uses default hotkey

# Terminal 2: worktree
cd ../heycat-feature-audio
bun run tauri dev   # Uses unique hotkey (CmdOrControl+Shift+N)
```
