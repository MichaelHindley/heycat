---
status: completed
created: 2025-11-26
completed: 2025-11-28
dependencies:
  - audio-capture
  - wav-encoding
  - recording-state-manager
---

# Spec: Recording Coordinator

## Description

Implement orchestration logic that coordinates between audio capture, WAV encoding, and state management for the full recording lifecycle. Handles start/stop operations and returns recording metadata.

## Acceptance Criteria

- [x] `start_recording()`: Initialize capture stream, transition to Recording state
- [x] `stop_recording()`: Stop stream, encode WAV, save file, return metadata
- [x] Return `RecordingMetadata` struct (duration, file_path, sample_count)
- [x] Clear audio buffer after successful save
- [x] Handle errors at each step gracefully with descriptive messages

## Test Cases

- [x] Full start→stop cycle produces WAV file at expected path
- [x] Metadata contains correct duration and sample count
- [x] Error during capture rolls back state to Idle
- [x] Error during encoding preserves audio buffer for retry
- [x] Concurrent start calls rejected when already recording

## Dependencies

- [audio-capture.spec.md](audio-capture.spec.md) - Audio primitives
- [wav-encoding.spec.md](wav-encoding.spec.md) - File encoding
- [recording-state-manager.spec.md](recording-state-manager.spec.md) - State management

## Preconditions

- All Layer 1 specs and Spec 2.1 completed
- State manager properly initialized

## Implementation Notes

- Create new module: `src-tauri/src/recording/coordinator.rs`
- RecordingMetadata struct: `{ duration_secs: f64, file_path: String, sample_count: usize }`
- Calculate duration: `sample_count / sample_rate`
- Use `?` operator for error propagation with context

## Related Specs

- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Exposes coordinator
- [event-emission.spec.md](event-emission.spec.md) - Emits events on state change
- [transcription-buffer.spec.md](transcription-buffer.spec.md) - Buffer access

## Review

**Reviewed:** 2025-11-28
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `start_recording()`: Initialize capture stream, transition to Recording state | PASS | coordinator.rs:89-113 - Validates Idle state, transitions to Recording, starts capture with rollback on failure |
| `stop_recording()`: Stop stream, encode WAV, save file, return metadata | PASS | coordinator.rs:118-160 - Stops capture, transitions to Processing, encodes WAV, returns metadata |
| Return `RecordingMetadata` struct (duration, file_path, sample_count) | PASS | coordinator.rs:38-47 - Struct defined with all required fields; coordinator.rs:155-159 - Instance returned |
| Clear audio buffer after successful save | PASS | coordinator.rs:150-153 - Transition to Idle clears buffer; state.rs:99-101 - Buffer cleared during Processing→Idle |
| Handle errors at each step gracefully with descriptive messages | PASS | coordinator.rs:10-36 - CoordinatorError enum with Display impl; coordinator.rs:107-110, 144-145 - Error propagation with context |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Full start→stop cycle produces WAV file at expected path | PASS | coordinator_test.rs:206-227 - test_full_cycle_produces_metadata |
| Metadata contains correct duration and sample count | PASS | coordinator_test.rs:220-222, 230-251 - Verified in both tests, including custom sample rate |
| Error during capture rolls back state to Idle | PASS | coordinator_test.rs:181-189 - test_capture_error_rollback |
| Error during encoding preserves audio buffer for retry | PASS | coordinator_test.rs:254-267 - test_encoding_error_preserves_buffer |
| Concurrent start calls rejected when already recording | PASS | coordinator_test.rs:169-178 - test_concurrent_start_rejected |

### Code Quality

**Strengths:**
- **Excellent error handling pattern**: Each error type is properly wrapped (CaptureError, StateError, EncodingError) with descriptive Display implementations
- **Proper state rollback**: When capture fails during start, state is rolled back to Idle using the new `reset_to_idle()` method (line 108)
- **Buffer lifecycle management**: Buffer is preserved during encoding errors (stays in Processing state) but cleared after successful save (transition to Idle)
- **Type safety**: Generic over backend and file writer allows for full testability without runtime dependencies
- **Duration calculation**: Correctly uses `sample_count / sample_rate` formula (line 148)
- **Comprehensive test coverage**: 15 tests covering all success paths, error paths, state transitions, and edge cases
- **Mock implementations**: Well-structured MockAudioBackend and MockFileWriter with builder pattern for configurability

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria fully met with excellent implementation quality. The coordinator properly orchestrates the recording lifecycle with robust error handling, correct state management, and comprehensive test coverage. The addition of `reset_to_idle()` in state.rs ensures clean rollback on capture failures.
