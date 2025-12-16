---
status: in-review
created: 2025-12-16
completed: null
dependencies: []
review_round: 1
priority: P1
---

# Spec: Remove or repurpose TranscriptionManager wrapper

## Description

`TranscriptionManager` in `parakeet/manager.rs` is a thin wrapper around `SharedTranscriptionModel` that adds no value. It simply passes through all calls without adding any functionality. This violates DRY and creates unnecessary indirection.

Either remove the wrapper entirely (consumers use SharedTranscriptionModel directly) or give it a meaningful purpose (e.g., managing multiple models, caching, metrics).

## Acceptance Criteria

- [ ] Evaluate: Remove wrapper OR give it real responsibility
- [ ] If removing: Update all callers to use SharedTranscriptionModel directly
- [ ] If keeping: Add meaningful functionality (document what)
- [ ] Remove unused code paths
- [ ] Update tests accordingly

## Test Cases

- [ ] Test that transcription still works after change
- [ ] Test all callers updated correctly
- [ ] Test no regression in functionality

## Dependencies

- transcription-race-condition.spec.md (modifying same module)

## Preconditions

- Current TranscriptionManager wrapper exists

## Implementation Notes

**File:** `src-tauri/src/parakeet/manager.rs`

**Current state:**
TranscriptionManager wraps SharedTranscriptionModel but just forwards calls:
```rust
pub struct TranscriptionManager {
    shared_model: Arc<SharedTranscriptionModel>,
}

impl TranscriptionManager {
    pub fn new() -> Self { ... }

    pub fn transcribe_file(&self, path: &str) -> Result<...> {
        self.shared_model.transcribe_file(path)  // Just forwards!
    }

    pub fn get_shared_model(&self) -> Arc<SharedTranscriptionModel> {
        self.shared_model.clone()  // Exposes underlying model anyway
    }
}
```

**Option A: Remove wrapper (recommended)**
- Delete manager.rs
- Update exports in mod.rs
- Callers use SharedTranscriptionModel directly
- SharedTranscriptionModel is already Arc-wrapped and thread-safe

**Callers to update:**
- `src-tauri/src/commands/logic.rs` - AppState uses TranscriptionManager
- `src-tauri/src/hotkey/integration.rs` - Uses TranscriptionManager
- `src-tauri/src/listening/manager.rs` - Uses shared model

**Option B: Give it purpose**
If we keep it, possible responsibilities:
- Manage model lifecycle (load on demand, unload on timeout)
- Track transcription metrics
- Implement caching for repeated transcriptions
- Support multiple models (different languages/sizes)

**Recommendation:** Option A - Remove the wrapper. The wrapper adds cognitive overhead without benefit. SharedTranscriptionModel is already the right abstraction.

## Related Specs

- transcription-race-condition.spec.md (modifies shared.rs)
- shared-transcription-model.spec.md (completed)

## Integration Points

- Production call site: `src-tauri/src/commands/logic.rs`
- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: AppState, HotkeyIntegration

## Integration Test

- Test location: Existing transcription tests
- Verification: [ ] Integration test passes

## Review

**Date:** 2025-12-16
**Reviewer:** Independent Subagent
**Commit:** c9df35f
**Round:** 1

---

### 1. Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Evaluate: Remove wrapper OR give it real responsibility | PASS | Option A chosen - wrapper removed. `src-tauri/src/parakeet/manager.rs` deleted (173 lines) |
| If removing: Update all callers to use SharedTranscriptionModel directly | PASS | `commands/logic.rs:429-450` now uses `SharedTranscriptionModel` directly; `hotkey/integration.rs:20,80,142` updated from `transcription_manager` to `shared_transcription_model`; `lib.rs:81,103,155` updated wiring |
| If keeping: Add meaningful functionality | N/A | Wrapper was removed, not kept |
| Remove unused code paths | PARTIAL | Wrapper removed, but new warning: `method 'state' is never used` at `shared.rs:178`. The `state()` method on `SharedTranscriptionModel` was called by `TranscriptionManager` but is now only used via trait impl (in tests) |
| Update tests accordingly | PASS | 22 shared model tests pass. Manager tests were deleted with the file |

---

### 2. Integration Path Trace

This spec is backend-only (no frontend-backend interaction changes). The integration path:

```
[Tauri Command: transcribe_file]
         |
         v
[commands/mod.rs:237] -----> [logic.rs:429 transcribe_file_impl]
                                      |
                                      v
                               [SharedTranscriptionModel.transcribe()]
                                      |
                                      v
                               [Parakeet TDT model]
```

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Command registered | `transcribe_file` in invoke_handler | `lib.rs:235` | PASS |
| State managed | `SharedTranscriptionModel` managed | `lib.rs:103` | PASS |
| Logic uses SharedTranscriptionModel | Direct call to shared_model | `logic.rs:448-450` | PASS |

---

### 3. Registration Audit

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| SharedTranscriptionModel | managed state | YES | `lib.rs:103: app.manage(shared_transcription_model.clone())` |
| transcribe_file command | Tauri command | YES | `lib.rs:235` in invoke_handler |

---

### 4. Mock-to-Production Audit

No new mocks introduced. `SharedTranscriptionModel` is used directly in production with `Arc` wrapper.

---

### 5. Event Subscription Audit

No new events introduced by this spec. Existing transcription events unchanged.

---

### 6. Deferral Tracking

No TODOs, FIXMEs, or deferrals found in changed files.

---

### 7. Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| Test that transcription still works after change | `shared.rs` tests: `test_transcribe_file_*`, `test_transcribe_samples_*` | PASS (4 tests) |
| Test all callers updated correctly | Build succeeds with callers using SharedTranscriptionModel | PASS |
| Test no regression in functionality | 22 shared model tests pass | PASS |

---

### 8. Build Warning Audit

**Backend (Rust):**
```
warning: method `state` is never used
   --> src/parakeet/shared.rs:178:12
```

This warning is NEW and introduced by this spec. Previously, `TranscriptionManager` called `shared_model.state()`. Now that the manager is removed, the direct `state()` method on `SharedTranscriptionModel` is only used via the trait implementation (for tests).

**Resolution required:** Add `#[allow(dead_code)]` to `SharedTranscriptionModel::state()` or call it from production code.

| Item | Type | Used? | Evidence |
|------|------|-------|----------|
| SharedTranscriptionModel::state() | method | NO (in prod) | Only used via TranscriptionService trait in tests |

---

### 9. Code Quality Notes

- [x] Error handling appropriate - maintains existing patterns
- [x] No unwrap() on user-facing code paths
- [x] Types are explicit
- [x] Consistent with existing patterns in codebase

---

### 10. Verdict

**NEEDS_WORK**

**What failed:** Build Warning Audit (Section 8)

**Why it failed:** The removal of `TranscriptionManager` left the `state()` method on `SharedTranscriptionModel` (line 178) unused in production code, causing a new `dead_code` warning.

**How to fix:**
1. Add `#[allow(dead_code)]` attribute above `pub fn state(&self)` at `src-tauri/src/parakeet/shared.rs:178`, OR
2. Remove the method if not needed (but it's part of the `TranscriptionService` trait impl, so option 1 is preferred)

```rust
// At shared.rs:177-178
#[allow(dead_code)] // Used via TranscriptionService trait in tests
pub fn state(&self) -> TranscriptionState {
```

---

### Review Checklist

- [x] Read the spec file completely
- [x] Read implementation notes and integration points in spec
- [x] Traced integration path with diagram
- [x] Verified all registrations in lib.rs
- [x] Audited mocks vs production
- [x] Audited event emission vs subscription
- [x] Searched for deferrals
- [x] Mapped test cases to actual tests
- [x] Ran `cargo build`, found new unused code warning
- [ ] Ran `bun run build` - has unrelated pre-existing type errors
