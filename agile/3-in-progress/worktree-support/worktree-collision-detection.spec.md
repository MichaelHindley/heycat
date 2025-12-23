---
status: in-progress
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-paths"]
review_round: 1
review_history:
  - round: 1
    date: 2025-12-23
    verdict: NEEDS_WORK
    failedCriteria: []
    concerns: ["`format_collision_error` function is exported and tested but never called from production code (TEST-ONLY usage). This is dead code in production. The function provides structured error formatting with resolution steps, but lib.rs constructs error messages inline instead.", "Multiple `#[allow(dead_code)]` annotations on public functions that ARE used in production (check_collision, create_lock, remove_lock, cleanup_stale_lock). These annotations are unnecessary and misleading - the functions are genuinely used in lib.rs."]
---

# Spec: Detect and report configuration collisions

## Description

Implement collision detection at app startup to identify situations where worktree isolation may have failed or where conflicting instances are running. Display clear error messages with resolution steps when collisions are detected.

## Acceptance Criteria

- [ ] Detect if another instance is using the same data directory (lock file check)
- [ ] Detect if worktree-specific paths already exist from a different worktree with same hash (unlikely but possible)
- [ ] Display user-friendly error dialog explaining the collision
- [ ] Provide specific resolution steps (e.g., "Close the other instance" or "Run cleanup script")
- [ ] Log collision details to console for debugging
- [ ] Allow app to continue in read-only mode if user acknowledges warning (optional)

## Test Cases

- [ ] No error when data directories are unused
- [ ] Error shown when lock file exists from another running instance
- [ ] Error includes the path to the conflicting resource
- [ ] Resolution steps are actionable and accurate
- [ ] App can be force-started with acknowledgment (if implemented)

## Dependencies

- worktree-detection (provides worktree identifier for path construction)
- worktree-paths (provides resolved paths to check for conflicts)

## Preconditions

- worktree-detection and worktree-paths are implemented
- Lock file mechanism defined (e.g., `heycat.lock` in data directory)

## Implementation Notes

- Create lock file on startup: `~/.local/share/heycat-{id}/heycat.lock`
- Lock file contains PID and timestamp
- Check if lock file exists and if PID is still running
- On macOS/Linux: use `kill(pid, 0)` to check if process exists
- On Windows: use process enumeration API
- Use Tauri dialog for error display: `tauri::api::dialog::message()`
- Clean up lock file on graceful shutdown

## Related Specs

- worktree-detection (dependency)
- worktree-paths (dependency - provides paths to lock)
- worktree-cleanup-script (can clean stale lock files)

## Integration Points

- Production call site: `src-tauri/src/lib.rs::setup()` - after path resolution, before store init
- Connects to:
  - worktree-paths module for path resolution
  - Tauri dialog API for error display
  - App lifecycle (shutdown hook for lock cleanup)

## Integration Test

- Test location: Manual testing - start two instances from same worktree
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Detect if another instance is using the same data directory (lock file check) | PASS | lib.rs:85-101 calls `check_collision()` at startup |
| Detect if worktree-specific paths already exist from a different worktree with same hash | DEFERRED | Not implemented - only lock file collision is checked. The spec notes this is "unlikely but possible" |
| Display user-friendly error dialog explaining the collision | PASS | lib.rs:97-101 returns error string from setup() which Tauri displays to user |
| Provide specific resolution steps | PASS | Error message includes "Please close it before starting a new instance" |
| Log collision details to console for debugging | PASS | lib.rs:90-95 logs error with PID and data_dir, plus resolution warning |
| Allow app to continue in read-only mode if user acknowledges warning (optional) | DEFERRED | Not implemented (marked optional in spec) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| No error when data directories are unused | PASS | collision_test.rs:29 `test_no_collision_when_lock_file_absent` |
| Error shown when lock file exists from another running instance | PASS | collision_test.rs:42 `test_detects_running_instance_collision` |
| Error includes the path to the conflicting resource | PASS | collision_test.rs:206 `test_format_collision_error_for_running_instance` - verifies paths in output |
| Resolution steps are actionable and accurate | PASS | collision_test.rs:206-219 verifies resolution steps exist and contain PID |
| App can be force-started with acknowledgment (if implemented) | DEFERRED | Not implemented (optional per spec) |

### Code Quality

**Strengths:**
- Clean separation of collision detection logic in dedicated module
- Complete lock file lifecycle: create, check, remove, cleanup stale
- Cross-platform process detection (Unix kill(0) and Windows OpenProcess)
- Comprehensive test coverage with 15 behavior-focused tests
- Good error handling with specific error types (CollisionError)
- Stale lock detection and automatic cleanup

**Concerns:**
- `format_collision_error` function is exported and tested but never called from production code (TEST-ONLY usage). This is dead code in production. The function provides structured error formatting with resolution steps, but lib.rs constructs error messages inline instead.
- Multiple `#[allow(dead_code)]` annotations on public functions that ARE used in production (check_collision, create_lock, remove_lock, cleanup_stale_lock). These annotations are unnecessary and misleading - the functions are genuinely used in lib.rs.

### Automated Check Results

```
Build warnings:
warning: unused import: `load_embedded_models`
warning: method `get` is never used (dead_code)

Command registration check:
check_parakeet_model_status
download_model
(These are unrelated to this spec)

Deferrals in codebase (unrelated to this spec):
src-tauri/src/parakeet/utils.rs:24 - TODO for parakeet-rs upstream issue
```

### Data Flow

```
[App Startup]
     |
     v
lib.rs:68 setup()
     |
     v
worktree::check_collision(context)  lib.rs:85
     |
     ├── NoCollision → create_lock() and continue
     |
     ├── InstanceRunning → return Err() → app fails with message
     |
     └── StaleLock → cleanup_stale_lock() → create_lock() and continue

[App Shutdown]
     |
     v
window close event  lib.rs:425-435
     |
     v
worktree::remove_lock(context)
```

### Verdict

**NEEDS_WORK** - `format_collision_error` is dead code in production (TEST-ONLY). Either:
1. Remove `format_collision_error` function and its tests since it's not used, OR
2. Use `format_collision_error` in lib.rs when constructing the error message to provide structured error formatting

The inline error message in lib.rs:97-101 duplicates the logic that `format_collision_error` provides. Choose one approach - either use the helper function or remove it.
