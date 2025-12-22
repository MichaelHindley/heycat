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
