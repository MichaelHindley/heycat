---
status: in-progress
created: 2025-12-21
completed: null
dependencies: ["dictionary-expander"]
review_round: 1
review_history:
  - round: 1
    date: 2025-12-21
    verdict: NEEDS_WORK
    failedCriteria: []
    concerns: ["**CRITICAL:** The `with_dictionary_expander()` builder method is implemented but **never called in production code**. In `lib.rs:152-176`, the `RecordingTranscriptionService` is built with command registry, matcher, dispatcher, and emitter, but **no expander is wired up**.", "This means dictionary expansion will NEVER occur in production - the feature is effectively dead code.", "The unused code warning confirms this: `warning: method 'with_dictionary_expander' is never used`"]
---

# Spec: Transcription Pipeline Integration (Backend)

## Description

Integrate the `DictionaryExpander` into the `RecordingTranscriptionService` pipeline. Dictionary expansion is applied after Parakeet transcription and before command matching, ensuring expanded text is used for both commands and clipboard.

See: `## Transcription + Expansion Pipeline Detail` in technical-guidance.md.

## Acceptance Criteria

- [ ] `RecordingTranscriptionService` accepts optional `DictionaryExpander` via builder
- [ ] Expansion applied after transcription result, before command matching
- [ ] Expanded text used for command matching (not original)
- [ ] Expanded text copied to clipboard (not original)
- [ ] `transcription_completed` event contains expanded text
- [ ] Graceful fallback: no expander = no expansion (original text used)

## Test Cases

- [ ] Full pipeline with dictionary: transcribed text expanded in clipboard and `transcription_completed` event
- [ ] Expanded text passed to command matcher (not original)

## Dependencies

- dictionary-expander.spec.md (provides DictionaryExpander)
- dictionary-store.spec.md (provides entries to expander)

## Preconditions

- DictionaryExpander implemented and tested
- DictionaryStore can load entries

## Implementation Notes

**Files to modify:**
- `src-tauri/src/transcription/service.rs` - Add expander integration

**Integration point in process_recording (around line 276-301):**
```rust
// After: let text = ... (transcription result)
// Before: let command_handled = Self::try_command_matching(...)

// Apply dictionary expansion
let expanded_text = if let Some(expander) = &self.dictionary_expander {
    expander.expand(&text)
} else {
    text.clone()
};

// Use expanded_text for command matching and clipboard
```

**Builder pattern addition:**
```rust
pub fn with_dictionary_expander(mut self, expander: Arc<DictionaryExpander>) -> Self {
    self.dictionary_expander = Some(expander);
    self
}
```

## Related Specs

- dictionary-expander.spec.md (provides the expander)
- dictionary-store.spec.md (source of entries)

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs:276-301`
- Connects to: DictionaryExpander, DictionaryStore, command matching, clipboard

## Integration Test

- Test location: Manual testing with dictionary entries
- Verification: [ ] Transcribed text with trigger words shows expansions in clipboard

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Pre-Review Gates

**1. Build Warning Check:**
```
warning: associated function `new` is never used
   = note: `#[warn(dead_code)]` on by default
warning: multiple associated items are never used
warning: method `with_dictionary_expander` is never used
warning: `heycat` (lib) generated 3 warnings
```
**FAIL** - `with_dictionary_expander` method is never used in production code.

**2. Command Registration Check:** N/A - no new commands added.

**3. Event Subscription Check:** N/A - uses existing `transcription_completed` event.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| RecordingTranscriptionService accepts optional DictionaryExpander via builder | PASS | `service.rs:162-165` - `with_dictionary_expander()` method implemented |
| Expansion applied after transcription result, before command matching | PASS | `service.rs:288-301` - expansion logic in correct position |
| Expanded text used for command matching (not original) | PASS | `service.rs:304-306` - `expanded_text` passed to `try_command_matching()` |
| Expanded text copied to clipboard (not original) | PASS | `service.rs:309-320` - `expanded_text` used in clipboard write |
| transcription_completed event contains expanded text | PASS | `service.rs:324-327` - `expanded_text` in payload |
| Graceful fallback: no expander = no expansion (original text used) | PASS | `service.rs:289-301` - else branch returns `text` unchanged |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Full pipeline with dictionary: transcribed text expanded in clipboard and transcription_completed event | PASS | `service.rs:583-621` - `test_dictionary_expander_integration_with_transcription_flow` |
| Expanded text passed to command matcher (not original) | DEFERRED | Logic tested via unit test pattern; full integration requires Tauri runtime |
| Graceful fallback: no expander | PASS | `service.rs:623-658` - `test_dictionary_expander_graceful_fallback_no_expander` and `_empty_entries` |

### Code Quality

**Strengths:**
- Correct placement of expansion logic in the pipeline (after transcription, before command matching)
- Clean builder pattern consistent with existing codebase style
- Good debug logging when expansion is applied
- Unit tests cover the expansion pattern correctly

**Concerns:**
- **CRITICAL:** The `with_dictionary_expander()` builder method is implemented but **never called in production code**. In `lib.rs:152-176`, the `RecordingTranscriptionService` is built with command registry, matcher, dispatcher, and emitter, but **no expander is wired up**.
- This means dictionary expansion will NEVER occur in production - the feature is effectively dead code.
- The unused code warning confirms this: `warning: method 'with_dictionary_expander' is never used`

### What Would Break If This Code Was Deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `with_dictionary_expander()` | fn | NONE | NO - DEAD CODE |
| `dictionary_expander` field | struct field | `process_recording()` | NO - always None |
| Expansion logic in `process_recording()` | code block | service.rs:288-301 | NO - expander is always None |

### Verdict

**NEEDS_WORK** - The dictionary expander integration logic is correctly implemented within `RecordingTranscriptionService`, but it is not wired up in production. The `with_dictionary_expander()` builder method is never called when constructing the service in `lib.rs`.

**What failed:** Pre-Review Gate #1 (unused code warning), Manual Review Question #1 (code not wired up end-to-end)

**Why it failed:** `with_dictionary_expander()` is implemented but the `RecordingTranscriptionService` in `lib.rs` is built without calling this method, leaving `dictionary_expander` as `None` in production.

**How to fix:**
1. In `lib.rs`, after line 170, add a call to wire up the dictionary expander:
   ```rust
   // After building with voice commands, add dictionary expansion
   if let Ok(store) = dictionary::DictionaryStore::load() {
       let entries = store.entries();
       if !entries.is_empty() {
           let expander = Arc::new(dictionary::DictionaryExpander::new(&entries));
           transcription_service = transcription_service.with_dictionary_expander(expander);
           debug!("Dictionary expander wired to TranscriptionService with {} entries", entries.len());
       }
   }
   ```
2. Re-run `cargo check` to verify the unused warning is resolved.
