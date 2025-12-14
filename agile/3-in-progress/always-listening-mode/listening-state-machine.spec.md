---
status: pending
created: 2025-12-14
completed: null
dependencies: []
---

# Spec: Add Listening state to recording system

## Description

Extend the existing recording state machine to include a `Listening` state for always-on wake word detection. Add Tauri commands to enable/disable listening mode and manage transitions between Idle, Listening, and Recording states.

## Acceptance Criteria

- [ ] `RecordingState` enum extended with `Listening` variant
- [ ] State transitions: `Idle` ↔ `Listening` ↔ `Recording` implemented
- [ ] `enable_listening` Tauri command starts listening mode
- [ ] `disable_listening` Tauri command stops listening mode
- [ ] `get_listening_status` Tauri command returns current listening state
- [ ] Manual recording (hotkey) works while in Listening state
- [ ] Wake word detection triggers transition from Listening → Recording
- [ ] Recording completion returns to Listening (if enabled) or Idle

## State Machine Edge Cases

- [ ] Hotkey press during Listening → Recording (listening suspended, not disabled)
- [ ] Wake word ignored if already Recording
- [ ] Recording completion returns to Listening (if `listening_enabled` flag true) or Idle
- [ ] `listening_enabled` flag persists across Recording state
- [ ] Microphone release during Listening → `listening_unavailable` event, auto-resume when available

## State Transition Matrix

```
From State    | Event              | To State    | Notes
--------------|--------------------|--------------|--------------------------
Idle          | enable_listening   | Listening   | Start audio capture
Listening     | disable_listening  | Idle        | Stop audio capture
Listening     | wake_word_detected | Recording   | listening_enabled stays true
Listening     | hotkey_pressed     | Recording   | listening_enabled stays true
Recording     | stop_recording     | Listening*  | *if listening_enabled, else Idle
Recording     | wake_word_detected | Recording   | Ignored (already recording)
Listening     | mic_unavailable    | Listening** | **listening_unavailable event
Processing    | complete           | Listening*  | *if listening_enabled, else Idle
```

## Test Cases

- [ ] Enable listening from Idle state succeeds
- [ ] Disable listening returns to Idle state
- [ ] Manual recording interrupts Listening, returns after completion
- [ ] Wake word triggers Recording from Listening state
- [ ] Cannot enable listening while already Recording
- [ ] State persists correctly across enable/disable cycles
- [ ] `listening_enabled` flag preserved during Recording state

## Dependencies

None

## Preconditions

- Existing RecordingManager and state machine functional

## Implementation Notes

- Modify `src-tauri/src/recording/state.rs` to add Listening variant
- Add new commands in `src-tauri/src/commands/mod.rs`
- `listening_enabled` should be a boolean flag in ListeningManager, not a state variant
- Coordinate with hotkey integration for seamless transitions

## Related Specs

- wake-word-detector.spec.md (triggers state transition)
- listening-audio-pipeline.spec.md (activated by this state)

## Integration Points

- Production call site: `src-tauri/src/recording/state.rs`, `src-tauri/src/listening/mod.rs`
- Connects to: commands/mod.rs, hotkey/integration.rs

## Integration Test

- Test location: `src-tauri/src/recording/state_test.rs`
- Verification: [ ] Integration test passes
