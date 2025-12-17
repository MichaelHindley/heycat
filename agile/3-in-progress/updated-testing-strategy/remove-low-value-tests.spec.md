---
status: in-progress
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

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Remove all Display trait implementation tests | FAIL | Display tests removed but unused warnings remain |
| Remove all serialization/deserialization format tests (unless critical to API) | FAIL | test_match_result_serialization still exists (src-tauri/src/voice_commands/matcher.rs) |
| Remove tests for obvious defaults (new() returns expected initial state) | PASS | Removed test_shared_model_new_is_unloaded, test_shared_model_default_is_unloaded, test_silence_detector_new, test_silence_detector_default, test_default_config |
| Remove tests that verify Rust's type system guarantees | PASS | Removed test_concurrent_access_does_not_panic, test_config_clone, test_transcription_state_clone, test_transcription_error_clone |
| Document removed tests in commit message for reference | FAIL | Commit message is "WIP: remove-low-value-tests" with no documentation |
| Coverage remains above 60% threshold | PASS | Backend: 65.55% lines, 72.90% functions; Frontend: >90% |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Display/Debug trait tests removed | PASS | Removed from audio/error.rs, parakeet/types.rs, listening/vad.rs, model/download.rs |
| Serialization tests removed | FAIL | test_match_result_serialization still exists in voice_commands/matcher.rs |
| Obvious defaults removed | PASS | Multiple default tests removed across files |
| Rust guarantees tests removed | PASS | Mutex and Clone tests removed from parakeet/shared.rs |
| All test suites pass | PASS | 288 tests passed, 0 failed |

### Code Quality

**Strengths:**
- Substantial reduction in test count (911 lines removed)
- Correctly identified and removed low-value tests across multiple categories
- Coverage thresholds maintained above requirements
- All remaining tests pass

**Concerns:**
- Build warnings introduced: unused imports VAD_CHUNK_SIZE_16KHZ and VAD_CHUNK_SIZE_8KHZ, method `start` is never used
- test_match_result_serialization still exists, violating acceptance criteria
- Commit message lacks documentation of removed tests
- Spec is marked as having dependency on testing-philosophy-guide but implementation appears incomplete regarding serialization test removal

### Verdict

**NEEDS_WORK** - Three acceptance criteria failed: (1) Unused warnings from removed tests, (2) Serialization test test_match_result_serialization still exists, (3) Commit message lacks documentation of removed tests
