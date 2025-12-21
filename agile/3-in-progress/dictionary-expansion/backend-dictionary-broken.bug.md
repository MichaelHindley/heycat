---
status: pending
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
