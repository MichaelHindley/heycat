---
status: pending
created: 2025-12-23
completed: null
dependencies: ["inference-result-handling"]
---

# Spec: Implement fallback behavior in process/flush

## Description

Update the public `process()` and `flush()` methods to handle `DtlnError` from `process_frame()` gracefully. On inference failure, log a warning and return the raw (unprocessed) audio samples instead of propagating the error or panicking. This ensures the audio stream continues uninterrupted.

## Acceptance Criteria

- [ ] `process()` catches errors from `process_frame()` and falls back to raw audio
- [ ] `flush()` catches errors from `process_frame()` and falls back to raw audio
- [ ] On error, a warning is logged with the error details
- [ ] Audio capture continues without interruption after inference errors
- [ ] Multiple consecutive errors are handled independently (each frame gets its own chance)
- [ ] Existing denoiser tests continue to pass

## Test Cases

- [ ] `process()` returns denoised samples when inference succeeds
- [ ] `process()` returns raw input samples when inference fails (not empty, not panic)
- [ ] `process()` logs warning when falling back to raw audio
- [ ] `flush()` returns buffered samples when inference fails (not empty, not panic)
- [ ] Multiple calls to `process()` with alternating success/failure each return appropriate output
- [ ] Audio callback in cpal_backend receives valid samples regardless of denoiser errors

## Dependencies

- `inference-result-handling` spec must be completed (process_frame returns Result)

## Preconditions

- `process_frame()` returns `Result<Vec<f32>, DtlnError>`
- DtlnError type is available

## Implementation Notes

**File to modify:** `src-tauri/src/audio/denoiser/dtln.rs`

**Update process() method (line ~92):**
```rust
pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
    // ... existing buffer logic ...

    while self.input_buffer.len() >= FRAME_SIZE {
        let frame: Vec<f32> = self.input_buffer[..FRAME_SIZE].to_vec();

        // Handle inference errors gracefully
        let processed = match self.process_frame(&frame) {
            Ok(denoised) => denoised,
            Err(e) => {
                crate::warn!("DTLN inference failed, using raw audio: {}", e);
                // Apply window to raw frame for overlap-add consistency
                frame.iter()
                    .zip(self.window.iter())
                    .map(|(&s, &w)| s * w)
                    .collect()
            }
        };

        // ... rest of overlap-add logic unchanged ...
    }
    output
}
```

**Similar pattern for flush() (line ~270).**

**Key insight:** The fallback applies the Hann window to maintain overlap-add consistency, preventing audio glitches at frame boundaries.

## Related Specs

- `error-type.spec.md` - provides DtlnError type
- `inference-result-handling.spec.md` - provides Result returns from process_frame (dependency)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs` (audio callback)
- Connects to: SharedDenoiser wrapper, audio capture pipeline

## Integration Test

- Test location: Manual test - start recording, verify audio continues if denoiser encounters issues
- Verification: [ ] Audio recording completes successfully even with simulated inference errors
