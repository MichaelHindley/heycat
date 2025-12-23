---
status: in-review
created: 2025-12-23
completed: null
dependencies: ["worktree-detection"]
review_round: 1
---

# Spec: Worktree-aware path resolution for data directories

## Description

Modify all data directory path resolution functions to incorporate the worktree identifier when running from a worktree. This ensures models, recordings, and other data files are stored in isolated locations per worktree.

## Acceptance Criteria

- [ ] `get_models_dir()` returns `~/.local/share/heycat-{worktree_id}/models/` when in worktree
- [ ] `get_recordings_dir()` returns `~/.local/share/heycat-{worktree_id}/recordings/` when in worktree
- [ ] Main repo paths remain unchanged: `~/.local/share/heycat/`
- [ ] Config dir paths also incorporate worktree identifier: `~/.config/heycat-{worktree_id}/`
- [ ] All existing path resolution callsites work without modification (API-compatible)
- [ ] Directories are created on first access if they don't exist

## Test Cases

- [ ] `get_models_dir()` returns standard path when worktree context is None
- [ ] `get_models_dir()` returns worktree-specific path when worktree context exists
- [ ] `get_recordings_dir()` behaves correctly for both contexts
- [ ] Path resolution is consistent across multiple calls (same worktree = same path)
- [ ] Cross-platform path separators handled correctly (Windows vs Unix)

## Dependencies

- worktree-detection (provides worktree identifier)

## Preconditions

- worktree-detection module is implemented and accessible
- Worktree context is available in app state

## Implementation Notes

- Create a centralized path resolution module (e.g., `src-tauri/src/paths.rs`)
- Modify `get_models_dir()` in `src-tauri/src/model/download.rs`
- Modify `get_recordings_dir()` in `src-tauri/src/commands/logic.rs`
- Modify config paths in `src-tauri/src/voice_commands/registry.rs` and `src-tauri/src/dictionary/store.rs`
- Use format: `heycat-{worktree_id}` where worktree_id is 8-char hash

## Related Specs

- worktree-detection (dependency - provides identifier)
- worktree-config (sibling - uses similar pattern for settings)
- worktree-cleanup-script (uses paths to know what to clean)

## Integration Points

- Production call site: Multiple files that resolve data paths
  - `src-tauri/src/model/download.rs::get_models_dir()`
  - `src-tauri/src/commands/logic.rs::get_recordings_dir()`
  - `src-tauri/src/voice_commands/registry.rs::with_default_path()`
  - `src-tauri/src/dictionary/store.rs::with_default_path()`
- Connects to: worktree-detection module for context

## Integration Test

- Test location: Integration tests for path resolution with mocked worktree context
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `get_models_dir()` returns worktree-specific path when in worktree | PASS | `paths.rs:81-83` - uses `get_data_dir()` which incorporates worktree identifier via `get_app_dir_name()` |
| `get_recordings_dir()` returns worktree-specific path when in worktree | PASS | `paths.rs:90-92` - uses `get_data_dir()` which incorporates worktree identifier |
| Main repo paths remain unchanged (`~/.local/share/heycat/`) | PASS | `paths.rs:49-53` - when context is None, returns `heycat` without suffix |
| Config dir paths incorporate worktree identifier | PASS | `paths.rs:71-74` - `get_config_dir()` uses same `get_app_dir_name()` pattern |
| All existing path resolution callsites work without modification (API-compatible) | PASS | All callsites updated: `model/download.rs:124`, `commands/logic.rs:310`, `dictionary/store.rs:78`, `voice_commands/registry.rs:95`, `audio/wav.rs:60` - all use `with_context(None)` wrappers for backward compatibility |
| Directories are created on first access if they don't exist | FAIL | `ensure_dir_exists()` at `paths.rs:97` is implemented but **never called from production code**. Directories are still created manually via `std::fs::create_dir_all` at various callsites |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `get_models_dir()` returns standard path when worktree context is None | PASS | `paths_test.rs:68-77` |
| `get_models_dir()` returns worktree-specific path when worktree context exists | PASS | `paths_test.rs:80-90` |
| `get_recordings_dir()` behaves correctly for both contexts | PASS | `paths_test.rs:93-115` |
| Path resolution is consistent across multiple calls | PASS | `paths_test.rs:120-127` |
| Cross-platform path separators handled correctly | PASS | Tests check both `/` and `\\` separators throughout `paths_test.rs` |

### Pre-Review Gate Results

```
warning: function `get_models_dir` is never used
   --> src/model/download.rs:133:8

warning: variant `DirectoryCreationFailed` is never constructed
  --> src/paths.rs:29:5

warning: function `ensure_dir_exists` is never used
  --> src/paths.rs:97:8
```

**FAIL: New code has unused warnings.** The spec introduces:
1. `get_models_dir()` (no-context version) in `download.rs:133` - never called
2. `DirectoryCreationFailed` variant - never constructed
3. `ensure_dir_exists()` - never called from production code

### Code Quality

**Strengths:**
- Clean centralized path resolution module with clear documentation
- Consistent pattern: `*_with_context()` for worktree-aware, `*()` for API-compatible
- Good test coverage of path resolution logic
- Proper use of `dirs` crate for cross-platform compatibility

**Concerns:**
- Three unused code items that cause cargo warnings (see above)
- `ensure_dir_exists()` exists but is not wired into production code - directories are still created manually at callsites
- The `DirectoryCreationFailed` error variant is defined but never constructed (the `ensure_dir_exists` function that would construct it is itself unused)

### Verdict

**NEEDS_WORK** - The implementation introduces unused code that triggers cargo warnings. Specifically:

1. **What failed:** Pre-Review Gate 1 (Build Warning Check) - 3 new unused warnings
2. **Why it failed:**
   - `ensure_dir_exists()` and `DirectoryCreationFailed` are implemented but never wired to production
   - `get_models_dir()` (no-context convenience wrapper) is never called
3. **How to fix:** Either:
   - (A) Remove the unused code: delete `ensure_dir_exists()`, `DirectoryCreationFailed`, and `get_models_dir()` (no-context version) if they're not needed for this spec
   - (B) Wire `ensure_dir_exists()` into production callsites to replace manual `create_dir_all` calls, making the code actually used
   - (C) Add `#[allow(dead_code)]` with a comment explaining these are public API for future specs (not recommended - see acceptance criterion about directories being created on first access)
