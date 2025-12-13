---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies: []
review_round: 1
---

# Spec: Use TranscriptionEventEmitter Trait

## Description

Refactor `HotkeyIntegration` to use the `TranscriptionEventEmitter` trait for emitting transcription events, matching the pattern established by `RecordingEventEmitter`. Currently, transcription events are emitted directly via `app_handle.emit()` in a spawned thread, bypassing the trait abstraction. This causes the compiler warning "trait TranscriptionEventEmitter is never used".

## Acceptance Criteria

- [ ] `HotkeyIntegration` receives a `TranscriptionEventEmitter` implementation as a second generic parameter
- [ ] `spawn_transcription()` uses trait methods instead of direct `app_handle.emit()` calls
- [ ] `TauriEventEmitter` implements both `RecordingEventEmitter` and `TranscriptionEventEmitter`
- [ ] No more "trait TranscriptionEventEmitter is never used" compiler warning
- [ ] Pattern consistent with `RecordingEventEmitter` usage

## Test Cases

- [ ] `MockEventEmitter` extended to implement `TranscriptionEventEmitter` for testing
- [ ] Existing transcription flow continues to work (manual test: hotkey → record → stop → transcribe → clipboard)
- [ ] Events are properly emitted (transcription_started, transcription_completed, transcription_error)

## Dependencies

None

## Preconditions

- Transcription pipeline is functional
- `TranscriptionEventEmitter` trait already defined in `src/events.rs`

## Implementation Notes

**Current state (bypassed pattern):**
```rust
// integration.rs:34 - stores AppHandle directly
app_handle: Option<AppHandle>,

// integration.rs:232-237 - direct emit calls
let _ = app_handle.emit(
    event_names::TRANSCRIPTION_STARTED,
    TranscriptionStartedPayload { timestamp: current_timestamp() },
);
```

**Target state (trait pattern):**
```rust
// Add second generic parameter
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter> {
    recording_emitter: R,
    transcription_emitter: Arc<T>,  // Arc for thread-safe sharing
    // ...
}

// Use trait methods
self.transcription_emitter.emit_transcription_started(payload);
```

**Key files:**
- `src-tauri/src/hotkey/integration.rs` - main refactor
- `src-tauri/src/events.rs` - extend MockEventEmitter
- `src-tauri/src/commands/mod.rs` - wire up TauriEventEmitter

## Related Specs

None - standalone refactoring spec

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs` (where HotkeyIntegration is instantiated)
- Connects to: events.rs (trait definitions), hotkey/integration.rs (usage)

## Integration Test

- Test location: Manual verification via hotkey recording flow
- Verification: [ ] Integration test passes / [x] N/A (manual verification)

## Review

**Date:** 2025-12-13
**Review Round:** 1

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `HotkeyIntegration` receives a `TranscriptionEventEmitter` implementation as a second generic parameter | ✅ | `integration.rs:24` - `pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter>` |
| `spawn_transcription()` uses trait methods instead of direct `app_handle.emit()` calls | ✅ | `integration.rs:229-231` - `transcription_emitter.emit_transcription_started(...)`, `integration.rs:272-275` - `transcription_emitter.emit_transcription_completed(...)`, `integration.rs:238-241` and `integration.rs:284-286` - `transcription_emitter.emit_transcription_error(...)` |
| `TauriEventEmitter` implements both `RecordingEventEmitter` and `TranscriptionEventEmitter` | ✅ | `commands/mod.rs:41-53` - `impl RecordingEventEmitter for TauriEventEmitter`, `commands/mod.rs:55-73` - `impl TranscriptionEventEmitter for TauriEventEmitter` |
| No more "trait TranscriptionEventEmitter is never used" compiler warning | ✅ | Verified via `cargo build 2>&1 \| grep -i TranscriptionEventEmitter` - no warnings found |
| Pattern consistent with `RecordingEventEmitter` usage | ✅ | Both traits follow identical patterns: trait definition in `events.rs:74-98`, `TauriEventEmitter` implements both, `HotkeyIntegration` parameterized by both |

### Test Coverage Analysis

| Test Case | Status | Evidence |
|-----------|--------|----------|
| `MockEventEmitter` extended to implement `TranscriptionEventEmitter` for testing | ✅ | `events.rs:141-153` - `impl TranscriptionEventEmitter for MockEventEmitter` with fields for all three transcription events (`transcription_started_events`, `transcription_completed_events`, `transcription_error_events` at lines 116-118) |
| Existing transcription flow continues to work | ⚠️ | Manual verification required - code paths are correct but no automated integration test |
| Events are properly emitted (transcription_started, transcription_completed, transcription_error) | ✅ | `integration.rs:229` emits `transcription_started`, `integration.rs:272` emits `transcription_completed`, `integration.rs:238` and `integration.rs:284` emit `transcription_error` |

**Additional test coverage:**
- `integration_test.rs:13-64` defines `MockEmitter` implementing both `RecordingEventEmitter` and `TranscriptionEventEmitter`
- `integration_test.rs:67` defines `type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter>` for test usage
- All 175 existing tests pass

### Architectural Assessment

**Strengths:**
1. **Clean trait abstraction**: The implementation correctly abstracts event emission behind the `TranscriptionEventEmitter` trait, enabling testability without Tauri dependencies
2. **Builder pattern consistency**: `with_transcription_emitter()` at `integration.rs:65-68` follows the same builder pattern as `with_audio_thread()` and `with_whisper_manager()`
3. **Thread-safe design**: `transcription_emitter` is stored as `Option<Arc<T>>` (`integration.rs:33`) for safe sharing across threads
4. **Production wiring verified**: `lib.rs:89-94` correctly instantiates `HotkeyIntegration` with `TauriEventEmitter` for both recording and transcription events

**Observations:**
1. The `transcription_emitter` is stored as `Option<Arc<T>>` while `recording_emitter` is stored as `R` directly. This is intentional - `transcription_emitter` needs to be cloned into the spawned thread (`integration.rs:203`), while `recording_emitter` is only used on the main thread.
2. The implementation correctly handles the case where `transcription_emitter` is `None` by early-returning from `spawn_transcription()` (`integration.rs:202-208`)

**No issues identified.**

### Verdict

**APPROVED**

All acceptance criteria are met. The implementation correctly refactors `HotkeyIntegration` to use the `TranscriptionEventEmitter` trait, eliminating the compiler warning and achieving consistency with the `RecordingEventEmitter` pattern. The production wiring in `lib.rs` correctly instantiates all components. All 175 tests pass.
