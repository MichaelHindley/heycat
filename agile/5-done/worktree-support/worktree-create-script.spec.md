---
status: completed
created: 2025-12-23
completed: 2025-12-23
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

### Pre-Review Gates

**Build Warning Check:**
```
warning: unused import: `load_embedded_models` (unrelated to this spec)
warning: method `get` is never used (unrelated to this spec)
```
Pre-existing warnings - PASS (not related to this spec)

**Command Registration Check:** N/A (script-only, no Tauri commands)

**Event Subscription Check:** N/A (script-only, no events)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Script creates worktree via `git worktree add` at specified path | PASS | `scripts/create-worktree.ts:165-175` - uses `Bun.spawn(["git", "worktree", "add", worktreePath, "-b", branchName])` |
| Script calculates worktree identifier using same algorithm as Rust backend | PASS | Script uses `basename(resolve(worktreePath))` (line 123), Rust uses `gitdir_path.file_name()` (detector.rs:96). Both extract the final path component. Git uses the worktree directory name as the identifier in `.git/worktrees/`. |
| Script creates initial settings file with unique default hotkey | PASS | `scripts/create-worktree.ts:143-159` - creates settings at correct path `~/Library/Application Support/com.heycat.app/settings-{identifier}.json` |
| Script displays the worktree path and generated hotkey | PASS | `scripts/create-worktree.ts:254-275` - comprehensive output with path, hotkey, and next steps |
| Script provides instructions for running dev server in new worktree | PASS | `scripts/create-worktree.ts:259-266` - step-by-step instructions including `cd`, `bun install`, and `bun run tauri dev` |
| Script validates that branch/path doesn't already exist | PASS | `scripts/create-worktree.ts:215-232` - validates `branchExists()`, `existsSync(worktreePath)`, and `worktreeExistsAtPath()` |
| Script handles errors gracefully with helpful messages | PASS | Colored error output with suggestions (e.g., lines 217-219 shows alternative command for existing branch) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Creates worktree successfully with valid branch name | PASS (manual) | `scripts/create-worktree.ts:236-240` - git worktree add command |
| Fails gracefully if branch already exists | PASS | `scripts/create-worktree.ts:216-220` - branchExists() check with helpful suggestion |
| Fails gracefully if worktree path already exists | PASS | `scripts/create-worktree.ts:223-227` - existsSync() check |
| Generated hotkey is unique (e.g., based on worktree hash) | PASS | `scripts/create-worktree.ts:103-115` - hash-based hotkey selection from [1-9,0] |
| Settings file is valid JSON with correct structure | PASS | `scripts/create-worktree.ts:153-158` - JSON.stringify with proper `hotkey.recordingShortcut` key |

### Manual Review Questions

**1. Is the code wired up end-to-end?**
N/A - This is a developer utility script, not application code. It's designed to be run manually via `bun scripts/create-worktree.ts`.

**2. What would break if this code was deleted?**
| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| create-worktree.ts | script | Developer CLI | N/A - utility script |

The script is a developer tool, not production application code.

**3. Where does the data flow?**
```
CLI args (branch, path)
     |
     v
Validation (branch exists?, path exists?, in main repo?)
     |
     v
git worktree add (creates worktree)
     |
     v
Identifier calculation (basename of path)
     |
     v
Hotkey generation (hash-based)
     |
     v
Settings file creation (~/Library/Application Support/com.heycat.app/settings-{id}.json)
     |
     v
User instructions (terminal output)
```

**4. Are there any deferrals?**
No TODO, FIXME, XXX, or HACK comments found.

### Code Quality

**Strengths:**
- Clean, well-documented TypeScript code with clear function separation
- Comprehensive help output with examples (`--help` flag)
- Colored terminal output for better developer experience
- Graceful error handling with helpful suggestions for recovery
- Three-way validation: branch existence, path existence, worktree existence at path
- Worktree identifier algorithm correctly matches Rust backend (both extract final path component)
- Settings file location correctly uses Tauri bundle identifier path (`com.heycat.app`)
- Hotkey generation uses deterministic hash for consistency

**Concerns:**
- None identified. Previous issue with settings file location has been fixed.

### Verification of Previous Review Fix

The previous review identified a CRITICAL issue: settings file was being created at `~/Library/Application Support/heycat/` instead of `~/Library/Application Support/com.heycat.app/`.

**Verified Fixed:** Lines 131-138 now correctly use:
- macOS: `~/Library/Application Support/com.heycat.app`
- Linux: `~/.local/share/com.heycat.app`

This matches the actual Tauri settings location at `~/Library/Application Support/com.heycat.app/settings.json`.

### Verdict

**APPROVED** - All acceptance criteria verified. The script correctly creates git worktrees with heycat-specific setup, generates unique hotkeys using a hash-based algorithm, creates settings files at the correct Tauri bundle identifier path, and provides comprehensive error handling and user instructions. The previous settings path issue has been resolved.
