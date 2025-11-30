---
status: completed
created: 2025-11-30
completed: 2025-11-30
dependencies:
  - audio-capture.spec.md
  - hotkey-integration.spec.md
---

# Spec: Audio Thread Integration

## Description

Wire together the audio capture infrastructure with the hotkey integration using a dedicated audio thread. This resolves the "audio capture handled separately" gap identified in the audit report.

## Background

The original hotkey-integration spec deferred audio capture with the note:
> "Audio capture will be managed separately when that spec is implemented."

This spec implements that missing integration using a dedicated audio thread pattern to handle cpal's thread-safety constraints (cpal::Stream is not Send+Sync).

## Acceptance Criteria

- [x] Create `AudioThreadHandle` that is `Send + Sync`
- [x] Audio thread owns `CpalBackend` and processes Start/Stop commands
- [x] `HotkeyIntegration` can accept an `AudioThreadHandle` via builder pattern
- [x] When hotkey toggles to Recording, audio capture starts
- [x] When hotkey toggles to Idle, audio capture stops
- [x] Buffer receives audio samples during recording
- [x] All existing tests continue to pass
- [x] New integration tests verify audio thread interaction

## Test Cases

- [x] `test_audio_thread_handle_is_send_sync` - Verifies handle can be shared across threads
- [x] `test_spawn_and_shutdown` - Thread can be spawned and cleanly shut down
- [x] `test_start_stop_commands` - Commands are accepted by the thread
- [x] `test_toggle_without_audio_thread_still_works` - Regression test for no-audio-thread case
- [x] `test_full_cycle_with_audio_thread` - End-to-end toggle cycle with real audio thread

## Dependencies

- audio-capture.spec.md (CpalBackend, AudioCaptureBackend trait)
- hotkey-integration.spec.md (HotkeyIntegration, state management)

## Preconditions

- CpalBackend fully implemented
- HotkeyIntegration managing state transitions
- RecordingManager creating AudioBuffer on Recording transition

## Implementation Notes

### Architecture

```
┌─────────────────┐   mpsc channel   ┌─────────────────────┐
│  Main Thread    │ ───────────────▶ │  Audio Thread       │
│                 │   Start(buffer)  │                     │
│  HotkeyInteg.   │   Stop           │  CpalBackend        │
│  AudioThreadH.  │   Shutdown       │  (owns cpal::Stream)│
└─────────────────┘                  └─────────────────────┘
                                              │
                                              ▼
                                      AudioBuffer (shared)
                                      Arc<Mutex<Vec<f32>>>
```

### Key Files

- `src-tauri/src/audio/thread.rs:1-130` - AudioThreadHandle, AudioCommand, audio_thread_main
- `src-tauri/src/hotkey/integration.rs:17-22` - HotkeyIntegration.audio_thread field
- `src-tauri/src/hotkey/integration.rs:36-40` - with_audio_thread() builder method
- `src-tauri/src/hotkey/integration.rs:82-92` - Start audio on Idle→Recording
- `src-tauri/src/hotkey/integration.rs:103-105` - Stop audio on Recording→Processing
- `src-tauri/src/lib.rs:44-47` - Spawn audio thread and wire to HotkeyIntegration

### Thread Safety Resolution

- `cpal::Stream` is NOT `Send + Sync` - cannot cross thread boundaries
- `AudioBuffer` (`Arc<Mutex<Vec<f32>>>`) IS `Send + Sync` - can be shared
- Solution: Spawn dedicated thread that owns CpalBackend, communicate via mpsc channel
- `Sender<AudioCommand>` IS `Send + Sync` - can be stored in HotkeyIntegration

## Related Specs

- [audio-capture.spec.md](audio-capture.spec.md) - CpalBackend implementation
- [hotkey-integration.spec.md](hotkey-integration.spec.md) - State management
- [recording-state-manager.spec.md](recording-state-manager.spec.md) - AudioBuffer lifecycle

## Integration Points

- Production call site: `lib.rs:44` - AudioThreadHandle::spawn()
- Production call site: `lib.rs:46` - with_audio_thread() injection
- Connects to: HotkeyIntegration, CpalBackend, RecordingManager

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs:263-285`
- Test: `test_full_cycle_with_audio_thread` - Spawns real audio thread, verifies full toggle cycle
- Verification: [x] Integration test passes

## Smoke Test (Manual)

1. Build and run: `bun run tauri dev`
2. Press Cmd+Shift+R to start recording
3. Speak into microphone for 2-3 seconds
4. Press Cmd+Shift+R to stop recording
5. Check `~/Library/Application Support/heycat/recordings/` for WAV file
6. Verify WAV file has non-zero size and plays back audio

## Notes

This spec addresses the critical gap identified in the audit report where all audio infrastructure was built and tested but never wired into the running application. The dedicated audio thread pattern was chosen over the simpler "inject CpalBackend directly" approach after discovering that cpal::Stream's lack of Send+Sync prevented sharing via Arc<Mutex<>>.
