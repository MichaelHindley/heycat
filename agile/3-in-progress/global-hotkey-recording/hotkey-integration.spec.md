---
status: completed
created: 2025-11-26
completed: 2025-11-28
dependencies:
  - global-hotkey
  - recording-coordinator
  - tauri-ipc-commands
---

# Spec: Hotkey-to-Recording Integration

## Description

Connect the global hotkey to the recording coordinator for toggle behavior. First press starts recording, second press stops. Handles rapid toggle presses with debouncing.

## Acceptance Criteria

- [ ] Hotkey callback toggles recording (Idle → Recording → Idle)
- [ ] First press transitions to Recording state and emits `recording_started` event
- [ ] Second press transitions to Idle state and emits `recording_stopped` event
- [ ] Emit events on toggle (via event emission)
- [ ] Handle rapid toggle presses gracefully (debounce ~200ms)

> **Note:** State transitions use `RecordingManager` directly rather than `RecordingCoordinator` because cpal audio streams are not `Send + Sync`, making them incompatible with Tauri's threaded callback model. Audio capture will be managed separately when that spec is implemented.

## Test Cases

- [ ] Toggle from Idle starts recording
- [ ] Toggle from Recording stops and saves
- [ ] Rapid double-press doesn't cause race condition
- [ ] Events emitted on each toggle
- [ ] Error state prevents toggle until reset

## Dependencies

- [global-hotkey.spec.md](global-hotkey.spec.md) - Hotkey registration
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Recording logic
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Command interface

## Preconditions

- Global hotkey and coordinator specs completed
- Event emission spec completed

## Implementation Notes

- Register hotkey callback during app setup in `lib.rs`
- Use `Instant::now()` for debounce timing
- Access state via `app.state::<Mutex<RecordingManager>>()`
- Callback runs on separate thread - use `app_handle.clone()` for emit

## Related Specs

- [global-hotkey.spec.md](global-hotkey.spec.md) - Provides hotkey callback
- [event-emission.spec.md](event-emission.spec.md) - Events for UI update

## Review

**Reviewed:** 2025-11-28 (Round 2)
**Reviewer:** Claude (Independent Code Review Agent)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Hotkey callback toggles recording (Idle → Recording → Idle) | **PASS** | `integration.rs:64-109` - State transitions implemented correctly |
| First press transitions to Recording state and emits `recording_started` event | **PASS** | `integration.rs:66-74` - Transitions to Recording (line 67-69), emits event (line 70-73) |
| Second press transitions to Idle state and emits `recording_stopped` event | **PASS** | `integration.rs:76-103` - Transitions Recording→Processing→Idle (lines 78-99), emits event (lines 101-102) |
| Emit events on toggle (via event emission) | **PASS** | `integration.rs:70-73,101-102` - Uses `RecordingEventEmitter` trait |
| Handle rapid toggle presses gracefully (debounce ~200ms) | **PASS** | `integration.rs:13,27,48-56` - 200ms debounce with `Instant::now()` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Toggle from Idle starts recording | **PASS** | `integration_test.rs:46-60` |
| Toggle from Recording stops and saves | **PASS** | `integration_test.rs:63-82` |
| Rapid double-press doesn't cause race condition | **PASS** | `integration_test.rs:85-105` |
| Events emitted on each toggle | **PASS** | `integration_test.rs:126-142` |
| Error state prevents toggle until reset | **PASS** | `integration_test.rs:145-164` |

**Additional test coverage:** 7 extra edge-case tests (total: 12 tests, 236 lines of test code)

### Code Quality

**Strengths:**
- Comprehensive test suite covering all edge cases (debounce timing, rapid toggles, Processing state blocking)
- Trait-based event emitter design enables testing without Tauri runtime (`MockEmitter` in tests)
- Proper thread-safety with `Mutex<RecordingManager>` - correctly uses `Send + Sync` types for threaded callback
- Configurable debounce duration for testing (production: 200ms, tests: 0-100ms for fast execution)
- Clean state machine logic with explicit `.expect()` messages explaining invariants
- Proper implementation of debounce using `Instant::now()` as specified in implementation notes
- Correct integration wiring in `lib.rs:40,43,46-47,54-62` - hotkey callback, state, and events properly connected
- Empty `file_path` in metadata (`integration.rs:91`) is intentional per comment "WAV encoding handled by IPC commands"

**Concerns:**
- None identified.

### Verdict

**APPROVED** - Implementation fully satisfies all updated acceptance criteria. The spec was clarified in Round 2 to use `RecordingManager` directly (not `RecordingCoordinator`) due to thread-safety constraints with cpal audio streams. The implementation correctly follows this architectural decision, properly transitions states, emits events, handles debouncing, and has comprehensive test coverage.
