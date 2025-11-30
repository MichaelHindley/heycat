---
status: completed
created: 2025-11-26
completed: 2025-11-28
dependencies:
  - recording-state-manager
  - recording-coordinator
---

# Spec: Tauri IPC Commands

## Description

Implement Tauri commands that expose recording functionality to the frontend via `invoke()`. Commands provide start/stop recording and state query operations.

## Acceptance Criteria

- [x] Command: `start_recording() -> Result<(), String>`
- [x] Command: `stop_recording() -> Result<RecordingMetadata, String>`
- [x] Command: `get_recording_state() -> Result<RecordingStateInfo, String>`
- [x] All commands access `State<Mutex<RecordingManager>>`
- [ ] Commands emit events on state changes (via event emission spec) - Deferred to event-emission spec

## Test Cases

- [x] `start_recording` returns Ok when transitioning from Idle
- [x] `stop_recording` returns metadata with correct file path
- [x] `get_recording_state` returns current state enum value
- [x] Commands return descriptive error messages on failure
- [x] State query works during recording without blocking

## Dependencies

- [recording-state-manager.spec.md](recording-state-manager.spec.md) - State access
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Business logic

## Preconditions

- State manager and coordinator specs completed
- Event emission spec ready (for integration)

## Implementation Notes

- Add commands in `src-tauri/src/lib.rs`
- Register in `invoke_handler`: `tauri::generate_handler![start_recording, stop_recording, get_recording_state]`
- Use `#[tauri::command]` attribute on each function
- Return types must be Serde-serializable

## Related Specs

- [event-emission.spec.md](event-emission.spec.md) - Events emitted by commands
- [recording-state-hook.spec.md](recording-state-hook.spec.md) - Frontend consumer

## Review

**Reviewed:** 2025-11-28
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Command: `start_recording() -> Result<(), String>` | PASS | `src-tauri/src/commands/mod.rs:19-21` - Tauri command wrapper with correct signature, `src-tauri/src/commands/logic.rs:22-42` - Implementation with state validation and transition logic |
| Command: `stop_recording() -> Result<RecordingMetadata, String>` | PASS | `src-tauri/src/commands/mod.rs:25-27` - Tauri command wrapper, `src-tauri/src/commands/logic.rs:55-98` - Complete implementation with WAV encoding and metadata calculation |
| Command: `get_recording_state() -> Result<RecordingStateInfo, String>` | PASS | `src-tauri/src/commands/mod.rs:31-33` - Tauri command wrapper, `src-tauri/src/commands/logic.rs:107-116` - State query implementation |
| All commands access `State<Mutex<RecordingManager>>` | PASS | `src-tauri/src/lib.rs:33` - RecordingManager managed as Mutex state, `src-tauri/src/commands/mod.rs:16` - ProductionState type alias for Mutex<RecordingManager>, all command signatures accept State<ProductionState> |
| Commands emit events on state changes | DEFERRED | Explicitly deferred to event-emission.spec.md as noted in acceptance criteria |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `start_recording` returns Ok from Idle | PASS | `src-tauri/src/commands/tests.rs:55-61` - Direct test of successful transition |
| `stop_recording` returns metadata with file path | PASS | `src-tauri/src/commands/tests.rs:131-152` - Tests with sample data, validates file_path contains .wav |
| `get_recording_state` returns current state | PASS | `src-tauri/src/commands/tests.rs:22-30` (Idle state), `src-tauri/src/commands/tests.rs:32-40` (Recording state) |
| Commands return descriptive error messages | PASS | `src-tauri/src/commands/tests.rs:72-81` (start while recording), `src-tauri/src/commands/tests.rs:97-104` (stop while not recording) |
| State query works during recording | PASS | `src-tauri/src/commands/tests.rs:32-40` - Queries state after start_recording without blocking |

### Code Quality

**Strengths:**
- Excellent separation of concerns: Tauri wrappers in `mod.rs` (excluded from coverage) delegate to testable `logic.rs` functions
- Proper coverage exclusions: `mod.rs` marked with `#![cfg_attr(coverage_nightly, coverage(off))]`, tests file also excluded appropriately
- Comprehensive test suite with 17 tests covering:
  - Happy paths for all three commands
  - Error conditions (already recording, not recording, poisoned locks)
  - State transitions through full Recording -> Processing -> Idle cycle
  - Edge cases (zero samples, multiple cycles)
  - Serialization of RecordingStateInfo
- Thread-safe design with explicit Mutex lock error handling
- Type safety: RecordingStateInfo derives Serialize, Clone, Debug as needed
- Commands properly registered in `lib.rs:44-49` invoke_handler
- Clear documentation with error conditions and return values documented

**Design Decisions:**
- Event emission deferred to separate spec (event-emission.spec.md) - appropriate separation of concerns
- Empty file_path returned when no samples recorded (line 82) - reasonable default
- State transitions through Processing before Idle during stop (lines 65-90) - matches coordinator spec

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria fully met. Implementation demonstrates excellent code organization with proper coverage exclusions, comprehensive test coverage (17 tests), and correct Tauri command registration. Event emission appropriately deferred to dedicated spec. Ready for integration.
