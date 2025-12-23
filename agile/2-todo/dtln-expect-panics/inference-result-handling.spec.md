---
status: pending
created: 2025-12-23
completed: null
dependencies: ["error-type"]
---

# Spec: Convert run_model methods to return Result

## Description

Replace the 4 `.expect()` calls in `run_model_1()` and `run_model_2()` with proper `Result` returns using the `DtlnError` type. Update `process_frame()` to handle these errors and fall back to returning the unprocessed frame on failure.

## Acceptance Criteria

- [ ] `run_model_1()` returns `Result<(Vec<f32>, Tensor), DtlnError>` instead of `(Vec<f32>, Tensor)`
- [ ] `run_model_2()` returns `Result<(Vec<f32>, Tensor), DtlnError>` instead of `(Vec<f32>, Tensor)`
- [ ] Line 208: `.expect("Model 1 inference failed")` replaced with `.map_err()`
- [ ] Line 211: `.expect("Invalid mask output")` replaced with `.map_err()`
- [ ] Line 239: `.expect("Model 2 inference failed")` replaced with `.map_err()`
- [ ] Line 242: `.expect("Invalid output")` replaced with `.map_err()`
- [ ] `process_frame()` returns `Result<Vec<f32>, DtlnError>` and propagates errors
- [ ] No `.expect()` or `.unwrap()` calls remain in production paths of dtln.rs

## Test Cases

- [ ] `run_model_1()` returns Ok with valid output on successful inference
- [ ] `run_model_1()` returns Err(DtlnError::InferenceFailed) when model.run() fails
- [ ] `run_model_1()` returns Err(DtlnError::InvalidOutput) when output shape is wrong
- [ ] `run_model_2()` returns Ok with valid output on successful inference
- [ ] `run_model_2()` returns Err(DtlnError::InferenceFailed) when model.run() fails
- [ ] `run_model_2()` returns Err(DtlnError::InvalidOutput) when output shape is wrong
- [ ] `process_frame()` propagates errors from run_model_1
- [ ] `process_frame()` propagates errors from run_model_2

## Dependencies

- `error-type` spec must be completed (provides DtlnError type)

## Preconditions

- DtlnError type is defined and exported from denoiser module

## Implementation Notes

**File to modify:** `src-tauri/src/audio/denoiser/dtln.rs`

**Change run_model_1 signature (line ~194):**
```rust
fn run_model_1(&self, magnitude: &[f32]) -> Result<(Vec<f32>, Tensor), DtlnError> {
```

**Replace expects with map_err (lines 207-211):**
```rust
let result = self.model_1.run(tvec![...])
    .map_err(|e| DtlnError::InferenceFailed {
        model: "model_1",
        message: e.to_string()
    })?;

let mask_output = result[0].to_array_view::<f32>()
    .map_err(|e| DtlnError::InvalidOutput {
        model: "model_1",
        message: e.to_string()
    })?;
```

**Same pattern for run_model_2 (lines 224-242).**

**Update process_frame signature (line ~128):**
```rust
fn process_frame(&mut self, frame: &[f32]) -> Result<Vec<f32>, DtlnError> {
```

## Related Specs

- `error-type.spec.md` - provides DtlnError type (dependency)
- `graceful-degradation.spec.md` - handles errors at process()/flush() level

## Integration Points

- Production call site: `src-tauri/src/audio/denoiser/dtln.rs:process_frame():128`
- Connects to: DtlnDenoiser::process(), DtlnDenoiser::flush()

## Integration Test

- Test location: `src-tauri/src/audio/denoiser/tests.rs` (existing denoiser tests)
- Verification: [ ] Existing tests still pass after signature changes
