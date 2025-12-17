---
status: in-review
created: 2025-12-17
completed: null
dependencies:
  - testing-philosophy-guide
---

# Spec: Identify and remove low-value tests

## Description

Audit all test files and remove tests that provide minimal value: Display trait tests, serialization format tests, obvious defaults, Rust-guaranteed behavior (mutex safety), and redundant coverage.

## Acceptance Criteria

- [ ] Remove all Display trait implementation tests
- [ ] Remove all serialization/deserialization format tests (unless critical to API)
- [ ] Remove tests for obvious defaults (new() returns expected initial state)
- [ ] Remove tests that verify Rust's type system guarantees
- [ ] Document removed tests in commit message for reference
- [ ] Coverage remains above 60% threshold

## Test Cases

Categories of tests to remove:

**Display/Debug trait tests:**
- [ ] `test_recording_state_error_display`
- [ ] Any `format!("{}", error)` assertion tests

**Serialization tests:**
- [ ] `test_match_result_serialization`
- [ ] Tests that just verify JSON contains expected strings

**Obvious defaults:**
- [ ] `test_default_state_is_idle`
- [ ] `test_new_manager_starts_idle` (when `default` test exists)

**Rust guarantees:**
- [ ] `test_concurrent_access_with_mutex`
- [ ] `test_error_is_std_error`

## Dependencies

testing-philosophy-guide - need clear criteria for what's "low value"

## Preconditions

- TESTING.md defines low-value test categories
- Other consolidation specs identify overlapping removals

## Implementation Notes

Approach:
1. Run `cargo test -- --list` and `bun test --list` to inventory all tests
2. Categorize each test by value (behavior vs implementation detail)
3. Remove tests that fall into low-value categories
4. Verify coverage threshold still met
5. Document reasoning in commit

This spec may overlap with consolidate-state-tests and consolidate-hook-tests. Coordinate to avoid duplicate work - those specs handle consolidation, this one handles pure removal of tests that shouldn't exist at all.

## Related Specs

- testing-philosophy-guide.spec.md
- consolidate-state-tests.spec.md
- consolidate-hook-tests.spec.md

## Integration Points

N/A - test cleanup only

## Integration Test

- Verification: [ ] All test suites pass
- Verification: [ ] Coverage >= 60%
- Verification: [ ] No regressions in actual behavior coverage

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Pre-Review Automated Checks

**Build Warning Check:**
```
warning: method `start` is never used
  --> src/audio/thread.rs:72:12
```
**FAIL** - Unused method `start` at src/audio/thread.rs:72 (dead code introduced by removing tests)

**Command Registration Check:**
Not applicable - spec does not add Tauri commands

**Event Subscription Check:**
Not applicable - spec does not add events

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Remove all Display trait implementation tests | PASS | Display tests removed from audio/error.rs, parakeet/types.rs, listening/vad.rs, model/download.rs |
| Remove all serialization/deserialization format tests (unless critical to API) | PASS | test_match_result_serialization removed (only comment remains at matcher_test.rs:188) |
| Remove tests for obvious defaults (new() returns expected initial state) | PASS | Removed test_shared_model_new_is_unloaded, test_shared_model_default_is_unloaded, test_silence_detector_new, test_silence_detector_default, test_default_config |
| Remove tests that verify Rust's type system guarantees | PASS | Removed test_concurrent_access_does_not_panic, test_config_clone, test_transcription_state_clone, test_transcription_error_clone |
| Document removed tests in commit message for reference | FAIL | Commit message "WIP: remove-low-value-tests: fix unused imports and serialization test" lacks detailed documentation of all removed tests |
| Coverage remains above 60% threshold | PASS | Backend: 65.55% lines, 72.90% functions; Frontend: >90% |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Display/Debug trait tests removed | PASS | Removed from audio/error.rs, parakeet/types.rs, listening/vad.rs, model/download.rs |
| Serialization tests removed | PASS | test_match_result_serialization removed (comment at matcher_test.rs:188) |
| Obvious defaults removed | PASS | Multiple default tests removed across files |
| Rust guarantees tests removed | PASS | Mutex and Clone tests removed from parakeet/shared.rs |
| All test suites pass | PASS | 287 tests passed, 0 failed |

### Manual Review Questions

**1. Is the code wired up end-to-end?**
N/A - This spec removes code rather than adding it. However, dead code was exposed:
- AudioThreadHandle::start method at src/audio/thread.rs:72 is now unreachable (not called from production)

**2. What would break if this code was deleted?**
Dead code identified:

| Code | Type | Production Call Site | Reachable from main/UI? |
|------|------|---------------------|-------------------------|
| AudioThreadHandle::start | method | None found | NO - Dead code exposed by test removal |

**3. Where does the data flow?**
N/A - Test removal spec

**4. Are there any deferrals?**
Pre-existing TODOs found (not introduced by this spec):
- src/parakeet/utils.rs:24-25 - TODO: Remove when parakeet-rs fixes issue upstream
- src/hotkey/integration_test.rs:347 - Comment about metadata being empty "for now"

These are pre-existing and not related to this spec's changes.

**5. Automated check results:**
See Pre-Review Automated Checks section above.

### Code Quality

**Strengths:**
- Substantial reduction in test count (911 lines removed)
- Successfully identified and removed low-value tests across all specified categories
- Coverage thresholds maintained well above requirements (65.55% lines, 72.90% functions)
- All remaining tests pass (287 passing)
- Correctly removed Display, serialization, default, and Rust guarantee tests

**Concerns:**
- Dead code exposed: AudioThreadHandle::start method at src/audio/thread.rs:72 is never used (should be removed or marked with #[allow(dead_code)] if needed for future use)
- Commit message lacks comprehensive documentation of removed tests - should list categories and counts

### Verdict

**NEEDS_WORK** - Dead code warning at src/audio/thread.rs:72 must be resolved (remove unused start method or add #[allow(dead_code)] with justification)
