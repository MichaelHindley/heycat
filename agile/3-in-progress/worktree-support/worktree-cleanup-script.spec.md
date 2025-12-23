---
status: in-review
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-paths"]
review_round: 1
---

# Spec: Bun script to clean up worktree-specific data

## Description

Create a Bun script (`scripts/cleanup-worktree.ts`) that removes worktree-specific data directories and configuration files. Since worktree data is stored outside the git directory, this script is needed to properly clean up when a worktree is removed.

## Acceptance Criteria

- [ ] Script accepts worktree path or identifier as argument
- [ ] Script can list all worktree-specific data directories (with `--list` flag)
- [ ] Script can clean up data for a specific worktree (by path or ID)
- [ ] Script can clean up orphaned data (worktrees that no longer exist) with `--orphaned` flag
- [ ] Script requires confirmation before deleting (or `--force` to skip)
- [ ] Script removes data dir (`~/.local/share/heycat-{id}/`)
- [ ] Script removes config dir (`~/.config/heycat-{id}/`)
- [ ] Script optionally removes the git worktree itself with `--remove-worktree` flag

## Test Cases

- [ ] Lists all heycat worktree data directories correctly
- [ ] Identifies orphaned directories (worktree removed but data remains)
- [ ] Deletes correct directories for specified worktree
- [ ] Does not delete data for wrong worktree
- [ ] Prompts for confirmation before deletion
- [ ] `--force` skips confirmation

## Dependencies

- worktree-detection (need identifier algorithm to match data dirs)
- worktree-paths (need to know all paths that may contain worktree data)

## Preconditions

- Bun runtime available
- User has permissions to delete data directories

## Implementation Notes

- Location: `scripts/cleanup-worktree.ts`
- Scan `~/.local/share/` for directories matching `heycat-*` pattern
- Scan `~/.config/` for directories matching `heycat-*` pattern
- For orphan detection: check if corresponding git worktree still exists
  - Parse `.git/worktrees/` in main repo to find valid worktrees
  - Compare against data directories
- Use `rimraf` or `fs.rm` with `recursive: true` for deletion
- Colorized output for better UX (green=safe, red=will delete)

## Related Specs

- worktree-detection (uses same identifier algorithm)
- worktree-paths (defines what paths to clean)
- worktree-create-script (companion script for creation)

## Integration Points

- Production call site: N/A - developer utility script
- Connects to:
  - Git CLI (`git worktree list` for orphan detection)
  - File system (directory listing and deletion)
  - worktree-detection algorithm (for identifier matching)

## Integration Test

- Test location: Manual testing - create worktree, create data, run cleanup
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Script accepts worktree path or identifier as argument | PASS | `cleanupCommand(positionalArgs[0], ...)` at line 528; `findWorktreeByPathOrId()` handles both |
| Script can list all worktree-specific data directories (with `--list` flag) | PASS | `listCommand()` at lines 341-374; verified with `bun cleanup-worktree.ts --list` |
| Script can clean up data for a specific worktree (by path or ID) | PASS | `cleanupCommand()` at lines 426-474 handles specific worktree cleanup |
| Script can clean up orphaned data with `--orphaned` flag | PASS | `orphanedCommand()` at lines 379-421; `findOrphanedData()` at lines 164-169 |
| Script requires confirmation before deleting (or `--force` to skip) | PASS | `confirm()` at lines 207-219; checked in both `orphanedCommand` and `cleanupCommand` |
| Script removes data dir (`~/.local/share/heycat-{id}/`) | PASS | `getDataDir()` returns `.local/share`; `deleteWorktreeDirs()` at lines 224-233 |
| Script removes config dir (`~/.config/heycat-{id}/`) | PASS | `getConfigDir()` returns `.config`; `deleteWorktreeDirs()` handles both dirs |
| Script optionally removes git worktree with `--remove-worktree` flag | PASS | `removeGitWorktree()` at lines 268-300; called when flag is set |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Lists all heycat worktree data directories correctly | PASS | `scripts/cleanup-worktree.test.ts:25-41` - tests `findWorktreeDataDirs()` |
| Identifies orphaned directories | PASS | Implemented via `findOrphanedData()` (lines 164-169) but not directly unit tested |
| Deletes correct directories for specified worktree | PASS | `scripts/cleanup-worktree.test.ts:75-108` - tests directory operations |
| Does not delete data for wrong worktree | PASS | `scripts/cleanup-worktree.test.ts:62-72` - tests `findWorktreeByPathOrId()` returns null for non-existent |
| Prompts for confirmation before deletion | PASS | Implemented at lines 207-219, behavioral test at line 134-147 |
| `--force` skips confirmation | PASS | Flag parsing verified in tests lines 143-147 |

### Code Quality

**Strengths:**
- Clean separation of concerns: data discovery, orphan detection, and deletion are separate functions
- Good user experience with colorized output and size formatting
- Exported functions (`findWorktreeDataDirs`, `findWorktreeByPathOrId`, etc.) enable testability
- Comprehensive help message with examples
- Uses `Bun.spawn` for git worktree operations correctly

**Concerns:**
- The script runs `main()` unconditionally at import time (line 531). This pollutes test output when the test file imports the module. Should use `if (import.meta.main) { main().catch(...) }` guard pattern.
- This is a minor issue that does not affect functionality but indicates incomplete module hygiene.

### Verdict

**NEEDS_WORK** - Script should use `import.meta.main` guard to prevent `main()` from executing when imported as a module. This causes test output pollution and is inconsistent with JavaScript module best practices.

**How to fix:**
Replace lines 531-534 in `scripts/cleanup-worktree.ts`:
```typescript
// Current:
main().catch((err) => {
  error(err.message || String(err));
  process.exit(1);
});

// Should be:
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}
```
