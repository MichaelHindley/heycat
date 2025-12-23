---
status: pending
severity: minor
origin: code-review
created: 2025-12-23
completed: null
parent_feature: null
parent_spec: null
---

# Bug: ONNX Model Inference Panics in DTLN Denoiser

**Created:** 2025-12-23
**Owner:** Claude
**Severity:** Minor
**Discovered:** Rust code review

## Problem Description

The DTLN denoiser uses `.expect()` on ONNX model inference results, which will panic and crash the entire application if inference fails. This is a violation of Rust error handling best practices - audio processing failures should degrade gracefully, not crash the app.

**Affected File:** `src-tauri/src/audio/denoiser/dtln.rs`

**Problematic Lines:**
- Line 208: `.expect("Model 1 inference failed")`
- Line 211: `.expect("Invalid mask output")`
- Line 239: `.expect("Model 2 inference failed")`
- Line 242: `.expect("Invalid output")`

**Code Context:**
```rust
// Lines 207-208 (run_model_1)
let result = self.model_1.run(tvec![...])
    .expect("Model 1 inference failed");

// Line 211
let mask_output = result[0].to_array_view::<f32>().expect("Invalid mask output");

// Lines 238-239 (run_model_2)
let result = self.model_2.run(tvec![...])
    .expect("Model 2 inference failed");

// Line 242
let output = result[0].to_array_view::<f32>().expect("Invalid output");
```

## Steps to Reproduce

1. Load the DTLN denoiser with noise suppression enabled
2. Start audio recording
3. Feed malformed or unexpected input tensor shapes to the ONNX model
4. **Expected:** Error is logged, denoiser is bypassed, audio continues unprocessed
5. **Actual:** Application panics and crashes

## Root Cause

The `run_model_1()` and `run_model_2()` methods use `.expect()` instead of proper `Result` error handling. This was likely done for simplicity during initial development, but violates the project's error handling patterns where typed errors are used internally.

The codebase otherwise follows the pattern of:
- Internal functions return `Result<T, CustomError>`
- Tauri command boundaries convert to `String`

These inference methods break that pattern by panicking instead of returning errors.

## Fix Approach

1. **Create a `DtlnError` type** in `denoiser/mod.rs` or a new `error.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum DtlnError {
       #[error("Model inference failed: {0}")]
       InferenceFailed(String),
       #[error("Invalid model output: {0}")]
       InvalidOutput(String),
   }
   ```

2. **Change method signatures** to return `Result`:
   ```rust
   fn run_model_1(&self, magnitude: &[f32]) -> Result<(Vec<f32>, Tensor), DtlnError>
   fn run_model_2(&self, time_domain: &[f32]) -> Result<(Vec<f32>, Tensor), DtlnError>
   ```

3. **Update `process_frame()`** to propagate errors or fall back to unprocessed audio

4. **Update `process()`** and `flush()`** to handle errors gracefully - either skip denoising for that frame or log and continue with raw audio

## Acceptance Criteria

- [ ] No `.expect()` calls remain in dtln.rs production code
- [ ] `run_model_1()` returns `Result<(Vec<f32>, Tensor), DtlnError>`
- [ ] `run_model_2()` returns `Result<(Vec<f32>, Tensor), DtlnError>`
- [ ] `process_frame()` handles inference errors gracefully
- [ ] On inference failure, audio continues (either unprocessed or with previous frame)
- [ ] Tests added to verify error handling behavior
- [ ] Existing denoiser tests still pass

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Normal audio processing | Denoised output produced, no errors | [ ] |
| Inference returns error | Error logged, raw audio returned, no panic | [ ] |
| Invalid output shape | Error logged, raw audio returned, no panic | [ ] |
| Multiple consecutive errors | Each error handled independently, processing continues | [ ] |

## Related Files

- `src-tauri/src/audio/denoiser/dtln.rs` - Main file to modify
- `src-tauri/src/audio/denoiser/mod.rs` - May need DtlnError type
- `src-tauri/src/audio/denoiser/shared.rs` - SharedDenoiser wrapper
- `src-tauri/src/audio/cpal_backend.rs` - Consumer of denoiser (check error handling)

## Notes

- This bug was discovered during a comprehensive Rust code review
- See `Rustreview.md` in project root for full review context
- The codebase has no unsafe code and this fix maintains that property
