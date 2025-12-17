---
status: completed
created: 2025-12-17
completed: 2025-12-17
dependencies:
  - escape-key-listener
  - double-tap-detection
---

# Spec: Cancel recording without transcription

## Description

Implement the cancel flow that stops recording, discards audio data, and returns to idle state without triggering transcription. This is triggered by the double-tap detection.

## Acceptance Criteria

- [ ] Recording stops immediately on cancel
- [ ] Audio buffer cleared without encoding/saving WAV
- [ ] No `spawn_transcription()` called
- [ ] State transitions: `Recording -> Idle` (bypasses `Processing`)
- [ ] Silence detection stopped if active
- [ ] `recording_cancelled` event emitted with reason

## Test Cases

- [ ] Cancel during recording clears buffer
- [ ] Cancel does not create WAV file
- [ ] Cancel does not trigger transcription
- [ ] Cancel emits `recording_cancelled` event
- [ ] State is `Idle` after cancel
- [ ] Silence detection thread stopped on cancel

## Dependencies

- escape-key-listener (Escape key must be registered)
- double-tap-detection (triggers cancel flow)

## Preconditions

- Recording is in progress (`RecordingState::Recording`)
- Audio thread is capturing audio

## Implementation Notes

- Add `cancel_recording()` method to `HotkeyIntegration`
- Different from `stop_recording()` - does not encode or transcribe
- Call `audio_thread.stop()` but discard result
- Transition directly to `Idle` state
- Emit `recording_cancelled` event with `{ reason: "double-tap-escape" }`

## Related Specs

- double-tap-detection.spec.md (triggers this flow)
- cancel-ui-feedback.spec.md (consumes cancel event)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: `RecordingManager`, `AudioThread`, event emitters

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Pre-Review Gate Results

```
Build Warning Check:
warning: method `with_escape_callback` is never used
warning: `heycat` (lib) generated 1 warning

(Note: with_escape_callback is the builder-pattern variant; set_escape_callback is used in production)

Command Registration Check: PASS (no new commands added)
Event Subscription Check: DEFERRED - recording_cancelled event listener handled by cancel-ui-feedback.spec.md
```

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Recording stops immediately on cancel | PASS | lib.rs:192 calls `cancel_recording()` from escape callback |
| Audio buffer cleared without encoding/saving WAV | PASS | integration.rs:1304-1306 calls `abort_recording(Idle)` which clears buffer |
| No `spawn_transcription()` called | PASS | Transitions directly to Idle, bypassing Processing state |
| State transitions: Recording -> Idle | PASS | integration.rs:1306 transitions to Idle via `abort_recording` |
| Silence detection stopped if active | PASS | integration.rs:1292 calls `stop_silence_detection()` |
| `recording_cancelled` event emitted with reason | PASS | integration.rs:1319 emits event with reason and timestamp |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Cancel during recording clears buffer | PASS | integration_test.rs:952 |
| Cancel does not create WAV file | PASS | integration_test.rs:952 (implicit) |
| Cancel does not trigger transcription | PASS | integration_test.rs:977 |
| Cancel emits recording_cancelled event | PASS | integration_test.rs:1000 |
| State is Idle after cancel | PASS | integration_test.rs:952 |
| Silence detection thread stopped on cancel | PASS | integration_test.rs:1114 |

### Code Quality

**Strengths:**
- Well-structured `cancel_recording()` method with proper error handling
- Comprehensive test coverage (12 cancel-related tests passing)
- Proper event payload structure with reason and timestamp
- Good documentation on the method
- Clean data flow from escape callback to cancel logic

**Concerns:**
- None identified

### What Would Break If This Code Was Deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `cancel_recording()` | fn | lib.rs:192 | YES |
| `emit_recording_cancelled()` | fn | integration.rs:1319 | YES |
| `RecordingCancelledPayload` | struct | integration.rs:1319 | YES |
| `RECORDING_CANCELLED` | const | commands/mod.rs:80 | YES |
| `set_escape_callback()` | fn | lib.rs:199 | YES |

### Data Flow Verification

```
[Double-tap Escape Key]
     |
     v
[DoubleTapDetector] src-tauri/src/hotkey/double_tap.rs
     | triggers callback when 2 taps within 300ms
     v
[Escape Callback] src-tauri/src/lib.rs:189-195
     | calls integration.cancel_recording()
     v
[cancel_recording()] src-tauri/src/hotkey/integration.rs:1268
     | 1. unregister_escape_listener()
     | 2. stop_silence_detection()
     | 3. audio_thread.stop()
     | 4. abort_recording(Idle)
     v
[emit_recording_cancelled()] src-tauri/src/hotkey/integration.rs:1319
     | emits "recording_cancelled" event
     v
[Frontend Listener] DEFERRED to cancel-ui-feedback.spec.md
```

### Deferrals Found

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| Frontend listener for recording_cancelled | N/A | cancel-ui-feedback.spec.md (pending) |

### Verdict

**APPROVED** - The cancel recording flow is fully implemented and wired up end-to-end. The escape callback at lib.rs:189-195 correctly calls `cancel_recording()` when double-tap is detected. All 12 related tests pass. The frontend listener for the `recording_cancelled` event is properly deferred to the pending `cancel-ui-feedback.spec.md`.
