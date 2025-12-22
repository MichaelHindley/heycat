---
status: in-review
created: 2025-12-22
completed: null
dependencies: []
review_round: 1
---

# Spec: Add atomic counters to track input/output sample counts in CallbackState

## Description

Add atomic counters to `CallbackState` in `cpal_backend.rs` to track the total number of input samples received from the device and output samples produced by the resampler. Log the sample counts and ratio at the end of each recording to diagnose the progressive speedup issue.

## Acceptance Criteria

- [ ] `CallbackState` has `input_sample_count: Arc<AtomicUsize>` field
- [ ] `CallbackState` has `output_sample_count: Arc<AtomicUsize>` field
- [ ] `process_samples()` increments input counter with `f32_samples.len()`
- [ ] `process_samples()` increments output counter with `samples_to_add.len()`
- [ ] Sample counts and ratio logged when recording stops (info level)
- [ ] Log format includes: input samples, output samples, actual ratio, expected ratio

## Test Cases

- [ ] After recording, logs show input/output sample counts
- [ ] Ratio logged matches expected (16000 / device_rate) within 1%
- [ ] Counters reset to 0 for each new recording

## Dependencies

None

## Preconditions

Device requires resampling (doesn't support 16kHz natively)

## Implementation Notes

**File:** `src-tauri/src/audio/cpal_backend.rs`

1. Add to `CallbackState` struct (line ~86-94):
```rust
input_sample_count: Arc<AtomicUsize>,
output_sample_count: Arc<AtomicUsize>,
```

2. In `process_samples()` (line ~101):
```rust
self.input_sample_count.fetch_add(f32_samples.len(), Ordering::Relaxed);
// ... after resampling ...
self.output_sample_count.fetch_add(samples_to_add.len(), Ordering::Relaxed);
```

3. Initialize counters in `start()` (line ~275-283):
```rust
input_sample_count: Arc::new(AtomicUsize::new(0)),
output_sample_count: Arc::new(AtomicUsize::new(0)),
```

4. Log in `stop()` or via a mechanism to access counters before CallbackState is dropped

## Related Specs

- `flush-residual-samples.spec.md` - depends on this spec

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:start()` creates CallbackState
- Connects to: Audio thread, recording state

## Integration Test

N/A - diagnostic logging verified via manual testing and log inspection

- Test location: N/A (debug/diagnostic feature)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `CallbackState` has `input_sample_count: Arc<AtomicUsize>` field | PASS | cpal_backend.rs:98 |
| `CallbackState` has `output_sample_count: Arc<AtomicUsize>` field | PASS | cpal_backend.rs:100 |
| `process_samples()` increments input counter with `f32_samples.len()` | PASS | cpal_backend.rs:112 |
| `process_samples()` increments output counter with `samples_to_add.len()` | PASS | cpal_backend.rs:177 |
| Sample counts and ratio logged when recording stops (info level) | PASS | cpal_backend.rs:258 calls `crate::info!()` |
| Log format includes: input samples, output samples, actual ratio, expected ratio | PASS | cpal_backend.rs:258-260 format string includes all |
| Counters reset to 0 for each new recording | PASS | cpal_backend.rs:371-372 new CallbackState with AtomicUsize::new(0) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| After recording, logs show input/output sample counts | N/A | Diagnostic verified via manual log inspection |
| Ratio logged matches expected (16000 / device_rate) within 1% | N/A | Manual verification |
| Counters reset to 0 for each new recording | PASS | cpal_backend.rs:371-372 (new CallbackState per recording) |
| test_resampler_flushes_partial_chunk | FAIL | cpal_backend.rs:478 - panics "First chunk should produce output" |
| test_sample_ratio_consistency | FAIL | cpal_backend.rs:522 - panics "Ratio error 9.82% exceeds 1%" |
| test_no_residual_after_flush | FAIL | cpal_backend.rs:577 - panics "Output samples should be counted after flush" |

### Pre-Review Gate Results

```
Build Warning Check: PASS (no new warnings - existing warning is in dictionary/store.rs:218, unrelated)
Command Registration Check: N/A (no new Tauri commands)
Event Subscription Check: N/A (no new events)
```

### Code Quality

**Strengths:**
- Clean separation of concerns with `log_sample_diagnostics()` method
- Atomic counters allow safe concurrent access from audio callback
- Informative log format with ratio error percentage for quick diagnosis
- Counters naturally reset via fresh CallbackState per recording

**Concerns:**
- Tests are failing: 3 tests in `resampler_tests` module fail with panics
- The tests appear to be from the companion `flush-residual-samples.spec.md` spec but are committed together
- While this spec is for diagnostics only, the failing tests affect the broader test suite

### Verdict

**NEEDS_WORK** - Tests are failing

1. **What failed**: Pre-Review Gate - 3 tests fail in `audio::cpal_backend::resampler_tests`
2. **Why it failed**: `test_resampler_flushes_partial_chunk`, `test_sample_ratio_consistency`, and `test_no_residual_after_flush` all panic with assertion failures
3. **How to fix**: Fix the failing tests in `src-tauri/src/audio/cpal_backend.rs:478-611`. The FftFixedIn resampler may have different behavior than expected - investigate why:
   - First chunk produces no output (latency/warmup issue with FFT resampler)
   - Ratio error is 9.82% instead of expected <1%
   - Output samples not counted after flush (likely related to first issue)
