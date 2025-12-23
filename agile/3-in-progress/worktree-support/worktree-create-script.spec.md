---
status: in-review
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-config"]
review_round: 1
---

# Spec: Bun script to create new worktrees with proper setup

## Description

Create a Bun script (`scripts/create-worktree.ts`) that automates the creation of new git worktrees with proper heycat setup. The script handles git worktree creation, generates a unique default hotkey, and provides instructions for running the dev server.

## Acceptance Criteria

- [ ] Script creates worktree via `git worktree add` at specified path
- [ ] Script calculates the worktree identifier using same algorithm as Rust backend
- [ ] Script creates initial settings file with unique default hotkey
- [ ] Script displays the worktree path and generated hotkey
- [ ] Script provides instructions for running dev server in new worktree
- [ ] Script validates that branch/path doesn't already exist
- [ ] Script handles errors gracefully with helpful messages

## Test Cases

- [ ] Creates worktree successfully with valid branch name
- [ ] Fails gracefully if branch already exists
- [ ] Fails gracefully if worktree path already exists
- [ ] Generated hotkey is unique (e.g., based on worktree hash)
- [ ] Settings file is valid JSON with correct structure

## Dependencies

- worktree-detection (need to match identifier algorithm)
- worktree-config (need to match settings file format and location)

## Preconditions

- Git repository with at least one commit
- Bun runtime available
- Running from main repository (not from a worktree)

## Implementation Notes

- Location: `scripts/create-worktree.ts`
- Use `Bun.spawn` for git commands
- Hotkey generation: Use worktree hash to pick from predefined set of hotkeys
  - E.g., `Cmd+Shift+1`, `Cmd+Shift+2`, etc. based on hash modulo
  - Or generate based on branch name: `Cmd+Shift+{first letter}`
- Settings file location: `~/.local/share/heycat-{id}/settings.json` (match Tauri store path)
- TypeScript for type safety and better DX

## Related Specs

- worktree-detection (must match identifier algorithm)
- worktree-config (must match settings format)
- worktree-cleanup-script (companion script for teardown)

## Integration Points

- Production call site: N/A - developer utility script
- Connects to:
  - Git CLI (`git worktree add`)
  - File system (settings file creation)
  - worktree-detection algorithm (for identifier calculation)

## Integration Test

- Test location: Manual testing - run script and verify worktree + settings
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Script creates worktree via `git worktree add` at specified path | PASS | `scripts/create-worktree.ts:163-175` - uses `Bun.spawn(["git", "worktree", "add", ...])` |
| Script calculates worktree identifier using same algorithm as Rust backend | PASS | Script uses `basename(resolve(worktreePath))` (line 123), Rust uses `gitdir_path.file_name()` (detector.rs:96). Both extract the final path component, and git uses the worktree directory name as the identifier in `.git/worktrees/` |
| Script creates initial settings file with unique default hotkey | FAIL | Script creates file at WRONG location - see Concerns |
| Script displays the worktree path and generated hotkey | PASS | `scripts/create-worktree.ts:245-269` - comprehensive output with path, hotkey, and next steps |
| Script provides instructions for running dev server in new worktree | PASS | `scripts/create-worktree.ts:257-265` - step-by-step instructions including `bun install` and `bun run tauri dev` |
| Script validates that branch/path doesn't already exist | PASS | `scripts/create-worktree.ts:215-232` - checks `branchExists()` and `existsSync(worktreePath)` and `worktreeExistsAtPath()` |
| Script handles errors gracefully with helpful messages | PASS | Colored error output with suggestions (e.g., line 217-219 shows alternative command for existing branch) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Creates worktree successfully with valid branch name | PASS (manual) | `scripts/create-worktree.ts:234-239` - git worktree add command |
| Fails gracefully if branch already exists | PASS | `scripts/create-worktree.ts:215-220` - branchExists() check with helpful suggestion |
| Fails gracefully if worktree path already exists | PASS | `scripts/create-worktree.ts:222-226` - existsSync() check |
| Generated hotkey is unique (e.g., based on worktree hash) | PASS | `scripts/create-worktree.ts:103-115` - hash-based hotkey selection |
| Settings file is valid JSON with correct structure | PASS | `scripts/create-worktree.ts:152-157` - JSON.stringify with proper key format |

### Code Quality

**Strengths:**
- Clean, well-documented code with clear function separation
- Comprehensive help output with examples
- Colored terminal output for better UX
- Graceful error handling with helpful suggestions
- Validates preconditions (main repo check, branch existence, path existence)
- Worktree identifier algorithm correctly matches Rust backend (both extract final path component)

**Concerns:**
- **CRITICAL: Settings file location mismatch.** The script creates settings at:
  - macOS: `~/Library/Application Support/heycat/settings-{identifier}.json`
  - Linux: `~/.local/share/heycat/settings-{identifier}.json`

  But Tauri plugin store uses the app's data directory based on bundle identifier:
  - macOS: `~/Library/Application Support/com.heycat.app/settings-{identifier}.json`

  Evidence: Verified actual Tauri settings location at `~/Library/Application Support/com.heycat.app/settings.json`

  The settings file created by the script will **never be read by the application**.

### How to Fix

1. In `scripts/create-worktree.ts`, update `getAppSupportDir()` function (lines 130-137) to use the correct Tauri app directory:

```typescript
function getAppSupportDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, "Library/Application Support/com.heycat.app");
  }
  // Linux: Tauri uses ~/.config/{bundle-id} or similar - verify actual path
  return resolve(home, ".config/com.heycat.app");
}
```

Note: Linux path may also need verification - Tauri uses different paths depending on XDG_CONFIG_HOME.

### Verdict

**NEEDS_WORK** - The script's settings file location (`~/Library/Application Support/heycat/`) does not match Tauri's actual settings location (`~/Library/Application Support/com.heycat.app/`). The worktree-specific settings file will be created in the wrong directory and will not be found by the application. Fix the `getAppSupportDir()` function to use the correct Tauri bundle identifier path.
