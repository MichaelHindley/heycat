---
status: in-review
created: 2025-12-17
completed: null
dependencies: []
review_round: 1
---

# Spec: Register Escape key listener during recording

## Description

Register a global shortcut for the Escape key that is only active while recording is in progress. The listener should be registered when recording starts and unregistered when recording stops (either normally or via cancellation).

## Acceptance Criteria

- [ ] Escape key listener registered when recording starts
- [ ] Escape key listener unregistered when recording stops (normal or cancelled)
- [ ] Does not interfere with other Escape key usage when not recording
- [ ] Uses existing `ShortcutBackend` abstraction for testability

## Test Cases

- [ ] Escape key callback fires when pressed during recording
- [ ] Escape key callback does not fire when not recording
- [ ] Listener properly cleaned up after recording stops
- [ ] Multiple start/stop cycles work correctly

## Dependencies

None

## Preconditions

- Existing hotkey infrastructure in `src-tauri/src/hotkey/`
- `ShortcutBackend` trait available for registration

## Implementation Notes

- Add Escape key registration to `HotkeyIntegration` or `HotkeyService`
- Register in `handle_toggle()` when starting recording
- Unregister in `handle_toggle()` when stopping recording
- Store callback handle for cleanup

## Related Specs

- double-tap-detection.spec.md (consumes Escape key events)
- cancel-recording-flow.spec.md (triggered by double-tap)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (handle_toggle)
- Connects to: `HotkeyService`, `ShortcutBackend`

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Escape key listener registered when recording starts | PASS | `integration.rs:376` calls `register_escape_listener()` in `handle_toggle()` when starting recording |
| Escape key listener unregistered when recording stops (normal or cancelled) | PASS | `integration.rs:396` calls `unregister_escape_listener()` in `handle_toggle()` when stopping recording |
| Does not interfere with other Escape key usage when not recording | PASS | Listener is only registered during recording (tested in `test_escape_callback_does_not_fire_when_not_recording`) |
| Uses existing `ShortcutBackend` abstraction for testability | PASS | `integration.rs:270` uses `Arc<dyn ShortcutBackend + Send + Sync>` for DI |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Escape key callback fires when pressed during recording | PASS | `integration_test.rs:651` `test_escape_callback_fires_during_recording` |
| Escape key callback does not fire when not recording | PASS | `integration_test.rs:708` `test_escape_callback_does_not_fire_when_not_recording` |
| Listener properly cleaned up after recording stops | PASS | `integration_test.rs:681` `test_escape_listener_unregistered_when_recording_stops` |
| Multiple start/stop cycles work correctly | PASS | `integration_test.rs:739` `test_escape_listener_multiple_cycles` |

### Code Quality

**Strengths:**
- Clean builder pattern for optional configuration (`with_shortcut_backend`, `with_escape_callback`)
- Graceful degradation when backend or callback not configured
- Comprehensive test coverage with mock backend
- Proper cleanup in all code paths

**Concerns:**
- **CRITICAL:** Build warnings show `methods 'register_escape_shortcut' and 'unregister_escape_shortcut' are never used` and `methods 'with_shortcut_backend' and 'with_escape_callback' are never used`. These methods are only called from tests, not production code. The `lib.rs` production setup does NOT call `.with_shortcut_backend()` or `.with_escape_callback()`, so the escape listener will never be registered in the actual application.

### Verdict

**NEEDS_WORK** - The escape listener infrastructure is correctly implemented and tested, but it is NOT wired up in production. The `HotkeyIntegration` setup in `src-tauri/src/lib.rs:152-176` does not call `.with_shortcut_backend()` or `.with_escape_callback()`, so the escape key listener will never be registered when the application runs. This is confirmed by the `dead_code` compiler warnings. Fix by wiring up the shortcut backend and escape callback in the production code path in `lib.rs`.
