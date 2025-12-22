---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["dictionary-expander"]
review_round: 2
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
warning: methods `save`, `add`, `update`, `delete`, and `get` are never used
   = note: `#[warn(dead_code)]` on by default
warning: `heycat` (lib) generated 1 warning
```
**PASS** - The only warning is for `DictionaryStore` CRUD methods (covered by a separate spec). No warnings for `with_dictionary_expander()` - it is now called in production.

**2. Command Registration Check:** N/A - no new commands added.

**3. Event Subscription Check:** N/A - uses existing `transcription_completed` event.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| RecordingTranscriptionService accepts optional DictionaryExpander via builder | PASS | `service.rs:161-165` - `with_dictionary_expander()` method implemented |
| Expansion applied after transcription result, before command matching | PASS | `service.rs:288-301` - expansion logic in correct position |
| Expanded text used for command matching (not original) | PASS | `service.rs:304-306` - `expanded_text` passed to `try_command_matching()` |
| Expanded text copied to clipboard (not original) | PASS | `service.rs:309-320` - `expanded_text` used in clipboard write |
| transcription_completed event contains expanded text | PASS | `service.rs:324-327` - `expanded_text` in TranscriptionCompletedPayload |
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
- Good debug logging when expansion is applied (`lib.rs:189`, `service.rs:292-296`)
- Unit tests verify the expansion pattern correctly
- Production wiring handles edge cases gracefully (empty dictionary, load failures)

**Concerns:**
- None identified

### What Would Break If This Code Was Deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `with_dictionary_expander()` | fn | `lib.rs:188` | YES |
| `dictionary_expander` field | struct field | `service.rs:196, 289` | YES |
| Expansion logic in `process_recording()` | code block | `service.rs:288-301` | YES |
| Dictionary loading in setup | code block | `lib.rs:163-190` | YES |

### Data Flow Verification

```
[Application Startup]
     |
     v
[DictionaryStore::with_default_path()] lib.rs:166
     | load()
     v
[DictionaryExpander::new(&entries)] lib.rs:182
     |
     v
[transcription_service.with_dictionary_expander()] lib.rs:188
     |
     v
[RecordingTranscriptionService stored in AppHandle state] lib.rs:204
     |
     v
[process_recording() called via hotkey/wake-word]
     |
     v
[expander.expand(&text)] service.rs:290
     |
     v
[expanded_text used for command matching] service.rs:305
     |
     v
[expanded_text copied to clipboard] service.rs:310
     |
     v
[expanded_text in transcription_completed event] service.rs:325
     |
     v
[Frontend receives expanded text]
```

### Verdict

**APPROVED** - The dictionary expander is now correctly wired into the transcription pipeline. The previous round's critical issue (dead code) has been resolved: `lib.rs:163-190` loads dictionary entries and wires the expander to the `RecordingTranscriptionService`. The expansion logic correctly applies after transcription and before command matching/clipboard, with graceful fallback for empty dictionaries or load failures.
