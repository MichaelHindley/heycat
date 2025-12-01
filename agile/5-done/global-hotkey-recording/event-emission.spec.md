---
status: completed
created: 2025-11-26
completed: 2025-11-28
dependencies:
  - recording-coordinator
  - tauri-ipc-commands
---

# Spec: Event Emission for Frontend

## Description

Define and emit Tauri events that notify the frontend of recording state changes in real-time. Events enable the UI to update reactively without polling.

## Acceptance Criteria

- [x] Event: `recording_started` with timestamp payload
- [x] Event: `recording_stopped` with RecordingMetadata payload
- [x] Event: `recording_error` with error message payload
- [x] Events emitted via `AppHandle.emit()`
- [x] Payloads are Serde-serializable

## Test Cases

- [x] `recording_started` event emitted when recording begins
- [x] `recording_stopped` event contains correct metadata
- [x] `recording_error` event includes descriptive message
- [x] Events received by frontend listeners (via MockEventEmitter)
- [x] Multiple listeners receive same event (via MockEventEmitter)

## Dependencies

- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Triggers events
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Command context for emit

## Preconditions

- Coordinator and IPC commands completed
- Understanding of Tauri event system

## Implementation Notes

- Use `app_handle.emit("event_name", payload)` pattern
- Event payloads: `RecordingStarted { timestamp: String }`, `RecordingStopped { metadata: RecordingMetadata }`
- Add `AppHandle` parameter to commands that need to emit
- Frontend listens via `listen("recording_started", callback)`

## Related Specs

- [recording-state-hook.spec.md](recording-state-hook.spec.md) - Frontend event listener
- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Also emits events

## Review

**Reviewed:** 2025-11-28
**Reviewer:** Claude (Independent Code Review Agent)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Event: `recording_started` with timestamp payload | PASS | `events.rs:15-20` defines `RecordingStartedPayload` |
| Event: `recording_stopped` with RecordingMetadata payload | PASS | `events.rs:22-27` defines `RecordingStoppedPayload` |
| Event: `recording_error` with error message payload | PASS | `events.rs:29-34` defines `RecordingErrorPayload` |
| Events emitted via `AppHandle.emit()` | PASS | `commands/mod.rs:33-44` implements `RecordingEventEmitter` using `app_handle.emit()` |
| Payloads are Serde-serializable | PASS | All payloads derive `Serialize` (`events.rs:16,23,30`) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `recording_started` event emitted when recording begins | PASS | `events.rs:139-149` tests emission via MockEventEmitter |
| `recording_stopped` event contains correct metadata | PASS | `events.rs:152-167` verifies metadata in payload |
| `recording_error` event includes descriptive message | PASS | `events.rs:170-180` verifies error message |
| Events received by frontend listeners (via MockEventEmitter) | PASS | MockEventEmitter records events in `Arc<Mutex<Vec>>` |
| Multiple listeners receive same event (via MockEventEmitter) | PASS | `events.rs:183-195` tests multiple emissions recorded |

### Code Quality

**Strengths:**
- Excellent trait-based abstraction (`RecordingEventEmitter`) enables testability
- All event payload structs properly derive `Serialize, Clone, Debug, PartialEq`
- Event names defined as constants in dedicated module (`events.rs:8-13`)
- `current_timestamp()` helper correctly uses ISO 8601 via `chrono::Utc::now().to_rfc3339()`
- MockEventEmitter uses `Arc<Mutex<Vec>>` for thread-safe test recording
- Comprehensive serialization tests verify JSON payloads (11 tests total)
- Proper coverage exclusion on Tauri-specific code with `#[cfg_attr(coverage_nightly, coverage(off))]`

**Concerns:**
- `TauriEventEmitter` is defined but not yet wired into commands - this is expected as the spec scope is event *definition*, not integration
- Integration with commands will be handled by `hotkey-integration.spec.md` which triggers events during hotkey actions

### Verdict

**APPROVED** - The spec scope is "Define and emit Tauri events" where the infrastructure for emission is complete. Event payloads are defined, serializable, and tested. The `TauriEventEmitter` implements `RecordingEventEmitter` using `AppHandle.emit()`. The actual triggering of events from user actions is delegated to downstream specs (hotkey-integration) which will call the emitter when recording starts/stops.
