---
status: completed
created: 2025-12-17
completed: 2025-12-17
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
| Escape key listener registered when recording starts | PASS | `integration.rs:377` calls `register_escape_listener()` in `handle_toggle()` when starting recording |
| Escape key listener unregistered when recording stops (normal or cancelled) | PASS | `integration.rs:397` calls `unregister_escape_listener()` in `handle_toggle()` when stopping recording |
| Does not interfere with other Escape key usage when not recording | PASS | Listener is only registered during recording (tested in `test_escape_callback_does_not_fire_when_not_recording`) |
| Uses existing `ShortcutBackend` abstraction for testability | PASS | `integration.rs:270` uses `Arc<dyn ShortcutBackend + Send + Sync>` for DI |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Escape key callback fires when pressed during recording | PASS | `integration_test.rs:655` `test_escape_callback_fires_during_recording` |
| Escape key callback does not fire when not recording | PASS | `integration_test.rs:713` `test_escape_callback_does_not_fire_when_not_recording` |
| Listener properly cleaned up after recording stops | PASS | `integration_test.rs:683` `test_escape_listener_unregistered_when_recording_stops` |
| Multiple start/stop cycles work correctly | PASS | `integration_test.rs:742` `test_escape_listener_multiple_cycles` |

### Code Quality

**Strengths:**
- Clean builder pattern for optional configuration (`with_shortcut_backend`, `with_escape_callback`)
- Graceful degradation when backend or callback not configured
- Comprehensive test coverage with mock backend
- Proper cleanup in all code paths
- Production wiring now complete in `lib.rs:154-177`

**Concerns:**
- None identified. Previous concern about missing production wiring has been addressed. The `lib.rs` now creates `escape_backend` (TauriShortcutBackend) and `escape_callback`, and passes them via `.with_shortcut_backend()` and `.with_escape_callback()` to the HotkeyIntegration builder.

### Pre-Review Gate Results

**Build Warning Check:** PASS - No unused/dead_code warnings for escape listener code
**Deferral Check:** PASS - Placeholder callback in lib.rs:157 references "double-tap detection will be added in a later spec" which is tracked by `double-tap-detection.spec.md` in the same feature

### Data Flow Verification

```
[Recording starts via hotkey]
     |
     v
[handle_toggle()] integration.rs:325
     | calls register_escape_listener()
     v
[register_escape_listener()] integration.rs:1133
     | backend.register(ESCAPE_SHORTCUT, callback)
     v
[TauriShortcutBackend.register()] tauri_backend.rs
     | registers with tauri_plugin_global_shortcut
     v
[Escape key pressed]
     | triggers callback
     v
[escape_callback] lib.rs:158
     | currently logs "Escape key pressed during recording"
     v
[Recording stops (manual or cancel)]
     |
     v
[unregister_escape_listener()] integration.rs:1173
     | backend.unregister(ESCAPE_SHORTCUT)
     v
[Escape listener removed]
```

### Verdict

**APPROVED** - All acceptance criteria are met and verified. The escape key listener is properly registered when recording starts and unregistered when recording stops. The implementation uses the existing ShortcutBackend abstraction for testability. All test cases pass. Production wiring is complete in lib.rs. The placeholder callback is appropriate as double-tap detection is tracked by a subsequent spec in this feature.
