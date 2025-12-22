---
status: completed
severity: major
origin: manual
created: 2025-12-21
completed: null
parent_feature: "dictionary-expansion"
parent_spec: null
---

# Bug: Backend Dictionary Broken

**Created:** 2025-12-21
**Severity:** Major

## Problem Description

When speaking phrases that match dictionary entries (e.g., "Hey"), the expanded content is not returned and no logs indicate dictionary activity. The dictionary expansion feature does not work after adding entries via the UI.

## Steps to Reproduce

1. Start the app
2. Add a dictionary entry via the Dictionary page (e.g., "Hey" → "Hello there")
3. Use voice transcription with a phrase containing "Hey"
4. Expected: Text is expanded to "Hello there"
5. Actual: Text remains as "Hey" with no dictionary logs

## Root Cause

The `DictionaryExpander` is created **once at app startup** (lib.rs:163-190) with initial entries loaded from disk. However, when users add/update/delete entries via the frontend, the CRUD commands in `commands/dictionary.rs`:
1. Update the `DictionaryStore` and persist to disk ✓
2. Emit `dictionary_updated` events ✓
3. **DO NOT** update the `DictionaryExpander` in `RecordingTranscriptionService` ✗

The expander holds pre-compiled regex patterns from startup and is never refreshed when the dictionary changes.

## Fix Approach

1. Change `RecordingTranscriptionService.dictionary_expander` from `Option<Arc<DictionaryExpander>>` to `Arc<Mutex<Option<DictionaryExpander>>>` for interior mutability
2. Add `update_dictionary_expander(&self, expander: Option<DictionaryExpander>)` method
3. In dictionary commands, after mutations, rebuild the expander and call the update method
4. Adjust `process_recording` to lock and use the mutex-wrapped expander

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Update dictionary expander method works | Expander is updated with new entries | [ ] |
| Dictionary expansion uses updated entries | New entries are applied after update | [ ] |
| Existing transcription tests still pass | No regression | [ ] |

## Integration Points

- `RecordingTranscriptionService` - needs interior mutability for expander
- `commands/dictionary.rs` - needs to call update method after mutations
- `lib.rs` - startup wiring needs to use new field type

## Integration Test

Verify that adding a dictionary entry via `add_dictionary_entry` command causes subsequent transcriptions to use the expanded text.

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude (Independent Review)

### Root Cause Analysis

**Verdict: CORRECTLY IDENTIFIED**

The bug correctly identifies that the `DictionaryExpander` was created once at startup and never updated when dictionary entries changed via CRUD operations. The root cause is accurate: dictionary commands updated the store and emitted events, but the expander in the transcription service was stale.

### Fix Implementation Analysis

**Changes Reviewed:**

1. **`src-tauri/src/transcription/service.rs`:**
   - Changed `dictionary_expander` from `Option<Arc<DictionaryExpander>>` to `Arc<RwLock<Option<DictionaryExpander>>>` - enables interior mutability for runtime updates
   - Added `update_dictionary(&self, entries: &[DictionaryEntry])` method - correctly rebuilds expander from fresh entries
   - Updated `process_recording` to use `RwLock::read()` pattern - thread-safe read access during transcription
   - Uses `RwLock` instead of `Mutex` (as proposed in fix approach) - better choice for read-heavy workload

2. **`src-tauri/src/commands/dictionary.rs`:**
   - Added `refresh_dictionary_expander()` helper function - calls `transcription_service.update_dictionary()` with current entries
   - All three mutation commands (`add_dictionary_entry`, `update_dictionary_entry`, `delete_dictionary_entry`) now call `refresh_dictionary_expander()` after successful mutations
   - Commands now take `transcription_service: State<'_, TranscriptionServiceState>` parameter for access to the service

3. **`src-tauri/src/lib.rs`:**
   - Simplified startup wiring to use `with_dictionary_expander()` builder pattern
   - No longer wraps expander in Arc (handled internally by service)

### Test Coverage Analysis

**Verdict: ADEQUATE**

Tests added in `transcription/service.rs`:
- `test_dictionary_expander_graceful_fallback_no_expander` - verifies no-op when expander is None
- `test_dictionary_expander_graceful_fallback_empty_entries` - verifies passthrough for empty dictionary
- `test_dictionary_expander_runtime_update` - **KEY TEST** - verifies that RwLock-based expander can be updated at runtime and subsequent expansions use new entries

All 18 dictionary-related tests pass.

### Regression Risk Assessment

**Risk: LOW**

- Existing transcription flow preserved - only the internal storage mechanism changed
- RwLock provides better concurrency than Mutex for read-heavy transcription operations
- Graceful fallback on lock acquisition failure (logs warning, returns original text)
- No changes to frontend code required - commands are stateless from frontend perspective

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Bug no longer reproducible | PASS | Fix correctly updates expander after each CRUD operation |
| Root cause addressed | PASS | Interior mutability via RwLock allows runtime updates |
| Tests added to prevent regression | PASS | `test_dictionary_expander_runtime_update` directly tests the fix |
| Related specs/features not broken | PASS | All 18 dictionary tests pass; no breaking changes to API |

### Minor Observations

1. The proposed fix mentioned `Arc<Mutex<...>>` but implementation uses `Arc<RwLock<...>>` - this is actually better for the read-heavy transcription use case
2. Method named `update_dictionary` (not `update_dictionary_expander` as proposed) - cleaner API, accepts entries directly

### Verdict

**APPROVED**

The fix correctly addresses the root cause by introducing interior mutability to the dictionary expander, allowing runtime updates when dictionary entries change. The implementation is thread-safe, well-tested, and follows established patterns in the codebase. Test coverage specifically validates the runtime update scenario that was broken.
