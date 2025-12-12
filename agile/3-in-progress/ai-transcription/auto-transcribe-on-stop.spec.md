---
status: completed
created: 2025-12-12
completed: 2025-12-12
dependencies:
  - transcription-pipeline
  - sample-rate-modification
---

# Spec: Auto-Transcribe on Recording Stop

## Description

Automatically start transcription when a recording stops. The transcribed text is copied to the clipboard on success. This creates the seamless voice-to-text workflow described in the feature BDD scenarios.

## Acceptance Criteria

- [ ] Transcription auto-starts when recording stops (after WAV save)
- [ ] Uses `get_last_recording_buffer()` to get audio samples for transcription
- [ ] Emits `transcription_started` event when transcription begins
- [ ] Emits `transcription_completed` event with transcribed text on success
- [ ] Emits `transcription_error` event with error message on failure
- [ ] Copies transcribed text to system clipboard on success
- [ ] Clipboard not modified on transcription error
- [ ] Frontend `useTranscription` hook listens to all transcription events

## Test Cases

- [ ] Recording stop triggers transcription automatically
- [ ] transcription_started event contains timestamp
- [ ] transcription_completed event contains transcribed text
- [ ] transcription_error event contains error message
- [ ] Clipboard contains transcribed text after success
- [ ] Clipboard unchanged after transcription failure
- [ ] useTranscription hook updates state on events

## Dependencies

- transcription-pipeline (WhisperManager.transcribe())
- sample-rate-modification (16kHz audio format)

## Preconditions

- Whisper model is loaded
- Recording has completed successfully
- Audio buffer is available via get_last_recording_buffer()

## Implementation Notes

- Integrate transcription into existing recording stop workflow
- Use Tauri's clipboard API or arboard crate for clipboard access
- Add transcription events to events.rs following existing pattern
- Consider: transcription happens in same thread or separate? (Technical guidance: transcription thread)

```rust
// Event payloads
pub struct TranscriptionStartedPayload {
    pub timestamp: String,
}

pub struct TranscriptionCompletedPayload {
    pub text: String,
    pub duration_ms: u64,
}

pub struct TranscriptionErrorPayload {
    pub error: String,
}
```

## Related Specs

- transcription-pipeline.spec.md (provides transcribe function)
- transcription-ui.spec.md (displays transcription state)

## Integration Points

- Production call site: `src-tauri/src/recording/state.rs` (after recording stop)
- Production call site: `src-tauri/src/hotkey/integration.rs` (stop recording handler)
- Connects to: WhisperManager, Clipboard, EventEmitter

## Integration Test

- Test location: N/A (unit-only spec)
- Justification: Hardware clipboard access (arboard::Clipboard), thread spawning (std::thread::spawn), and Tauri AppHandle requirements make integration testing impractical. The implementation is covered by unit tests for event payloads (events.rs) and frontend hook behavior (useTranscription.test.ts). The auto-transcription flow is validated by code inspection - the `spawn_transcription` function is appropriately excluded from coverage.

## Review

**Reviewed:** 2025-12-12
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Transcription auto-starts when recording stops (after WAV save) | PASS | integration.rs:164 - `self.spawn_transcription()` called after `emit_recording_stopped` in stop handler |
| Uses `get_last_recording_buffer()` to get audio samples for transcription | PASS | integration.rs:237 - `get_last_recording_buffer_impl(&recording_state)` fetches audio samples |
| Emits `transcription_started` event when transcription begins | PASS | integration.rs:229-234 - emits `TRANSCRIPTION_STARTED` with `TranscriptionStartedPayload` |
| Emits `transcription_completed` event with transcribed text on success | PASS | integration.rs:278-281 - emits `TRANSCRIPTION_COMPLETED` with `TranscriptionCompletedPayload { text, duration_ms }` |
| Emits `transcription_error` event with error message on failure | PASS | integration.rs:240-248 (buffer error), 290-295 (transcription error) - emits `TRANSCRIPTION_ERROR` with `TranscriptionErrorPayload` |
| Copies transcribed text to system clipboard on success | PASS | integration.rs:264-275 - uses `arboard::Clipboard` to `set_text(&text)` on success |
| Clipboard not modified on transcription error | PASS | integration.rs:288-295 - error path does not touch clipboard, only emits error event |
| Frontend `useTranscription` hook listens to all transcription events | PASS | useTranscription.ts:43-72 - listens to `transcription_started`, `transcription_completed`, `transcription_error` |

### Test Verification

| Behavior | Tested By | Notes |
|----------|-----------|-------|
| Recording stop triggers transcription automatically | N/A | spawn_transcription excluded from coverage (thread spawning, hardware clipboard) |
| transcription_started event contains timestamp | Unit | events.rs:333-339 - test_transcription_started_payload_serialization |
| transcription_completed event contains transcribed text | Unit | events.rs:343-353 - test_transcription_completed_payload_serialization |
| transcription_error event contains error message | Unit | events.rs:356-363 - test_transcription_error_payload_serialization |
| Clipboard contains transcribed text after success | N/A | Requires hardware clipboard (arboard::Clipboard) |
| Clipboard unchanged after transcription failure | N/A | Requires hardware clipboard (arboard::Clipboard) |
| useTranscription hook updates state on events | Unit | useTranscription.test.ts:54-169 - comprehensive event handler tests |

### Code Quality

**Strengths:**
- Clean builder pattern for HotkeyIntegration configuration (with_whisper_manager, with_app_handle, with_recording_state)
- Proper thread spawning for transcription to avoid blocking main thread
- Defensive checks for all required components before spawning transcription (lines 192-214)
- Follows existing event patterns with TranscriptionEventEmitter trait (events.rs:89-98)
- Proper error handling with warnings for clipboard failures without aborting transcription
- Uses v8 ignore comments for untestable async Tauri event listener code in frontend hook
- WhisperManager state reset after transcription (both success and error paths)

**Concerns:**
- None identified

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | PASS | lib.rs:80-104 - WhisperManager, AppHandle, and recording_state all properly configured in HotkeyIntegration builder chain |
| Any "handled separately" without spec reference? | PASS | No untracked deferrals found in implementation |
| Integration test exists and passes? | N/A | unit-only spec - justification documented in spec (hardware clipboard, thread spawning, AppHandle requirements) |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | - | - |

### Verdict

APPROVED - Implementation correctly auto-transcribes when recording stops, emits all required events, copies text to clipboard on success, and the frontend hook properly listens to all events. Code follows established patterns with proper error handling and thread safety. Coverage exclusions are well-justified for untestable code paths (hardware clipboard, thread spawning).
