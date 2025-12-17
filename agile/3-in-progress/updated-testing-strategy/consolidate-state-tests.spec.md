---
status: completed
created: 2025-12-17
completed: 2025-12-17
dependencies:
  - testing-philosophy-guide
---

# Spec: Consolidate RecordingManager state tests into behavior scenarios

## Description

Replace the 59 granular state transition tests in `state_test.rs` with ~5-10 behavior-focused tests that cover real usage scenarios. Focus on testing what users/callers actually need, not every permutation of the state machine.

## Acceptance Criteria

- [ ] Reduce test count from 59 to 5-10 behavior tests
- [ ] Cover: complete recording flow, abort scenarios, listening mode, error recovery
- [ ] Remove tests for: Display trait, individual invalid transitions, concurrent mutex access
- [ ] All remaining tests verify observable behavior, not internal state
- [ ] Coverage remains above 60% threshold

## Test Cases

Target test structure:
- [ ] `test_complete_recording_flow` - Idle -> Recording -> Processing -> Idle with data
- [ ] `test_listening_mode_flow` - Idle -> Listening -> Recording -> Processing -> Listening
- [ ] `test_abort_discards_recording` - Recording + abort = no data retained
- [ ] `test_error_recovery` - Invalid operations don't corrupt state
- [ ] `test_last_recording_persists` - Can access previous recording after new one starts

## Dependencies

testing-philosophy-guide - need guidelines before refactoring

## Preconditions

- Current tests pass
- TESTING.md guide exists

## Implementation Notes

Tests to REMOVE (low value):
- `test_new_manager_starts_idle` / `test_default_manager_starts_idle` (redundant)
- `test_default_state_is_idle` (obvious)
- All individual invalid transition tests (59 -> covered by error recovery test)
- `test_recording_state_error_display` (Display trait)
- `test_error_is_std_error` (trait implementation)
- `test_concurrent_access_with_mutex` (Rust guarantees this)
- `test_match_result_serialization` (serialization format)

Tests to CONSOLIDATE:
- All valid transition tests -> one flow test
- All buffer availability tests -> part of flow tests
- All sample rate tests -> part of flow tests

File: `src-tauri/src/recording/state_test.rs`

## Related Specs

- testing-philosophy-guide.spec.md
- remove-low-value-tests.spec.md

## Integration Points

N/A - test refactoring only

## Integration Test

- Verification: [ ] `cargo test` passes with reduced test count
- Verification: [ ] Coverage >= 60%

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Reduce test count from 59 to 5-10 behavior tests | PASS | Reduced from 57 to 6 tests (git diff shows -748 lines). Tests now at: state_test.rs:1-206 |
| Cover: complete recording flow, abort scenarios, listening mode, error recovery | PASS | All flows covered: test_complete_recording_flow (lines 7-39), test_abort_discards_recording (lines 78-109), test_listening_mode_flow (lines 43-74), test_error_recovery (lines 113-144) |
| Remove tests for: Display trait, individual invalid transitions, concurrent mutex access | PASS | No Display trait tests, no individual transition tests (replaced with single error_recovery test), no mutex concurrency tests remain |
| All remaining tests verify observable behavior, not internal state | PASS | All 6 tests verify behavior through public API: state transitions, buffer access, data retention. No private field inspection |
| Coverage remains above 60% threshold | PASS | recording/state.rs: 76.23% lines, 73.33% functions (both exceed 60%) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| test_complete_recording_flow | PASS | state_test.rs:7-39 |
| test_listening_mode_flow | PASS | state_test.rs:43-74 |
| test_abort_discards_recording | PASS | state_test.rs:78-109 |
| test_error_recovery | PASS | state_test.rs:113-144 |
| test_last_recording_persists | PASS | state_test.rs:149-186 |
| test_audio_duration_calculation | PASS | state_test.rs:189-205 |

### Code Quality

**Strengths:**
- Each test clearly documents its purpose with doc comments explaining behavior
- Tests follow real-world usage patterns (wake-word flow, user cancel, invalid operations)
- Proper cleanup verification (abort tests check no data retained)
- Error recovery test ensures state remains valid after invalid operations
- Last recording persistence test covers multi-session scenarios

**Concerns:**
- Build warnings present (unused imports VAD_CHUNK_SIZE_16KHZ/VAD_CHUNK_SIZE_8KHZ, unused method `start`) - these are unrelated to this spec but should be addressed

### Verdict

**APPROVED** - Successfully consolidated 57 granular state machine tests into 6 behavior-focused tests while maintaining 76% coverage. All acceptance criteria met. Tests now verify observable behavior from caller perspective rather than internal state transitions.
