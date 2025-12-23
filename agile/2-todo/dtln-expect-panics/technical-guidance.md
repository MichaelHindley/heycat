---
last-updated: 2025-12-23
status: complete
---

# Technical Guidance: DTLN Expect Panics

## Root Cause Analysis

The DTLN denoiser (`src-tauri/src/audio/denoiser/dtln.rs`) uses `.expect()` on ONNX model inference results instead of proper error handling. This violates the project's error handling pattern where internal functions return typed errors.

**Problematic Code:**
- Line 208: `.expect("Model 1 inference failed")` - panics if tract model.run() fails
- Line 211: `.expect("Invalid mask output")` - panics if output tensor has wrong shape
- Line 239: `.expect("Model 2 inference failed")` - panics if tract model.run() fails
- Line 242: `.expect("Invalid output")` - panics if output tensor has wrong shape

**Why It Exists:** Likely added for simplicity during initial development, when the focus was on getting denoising working rather than production hardening.

**Impact:** If the ONNX runtime encounters unexpected input (malformed tensor shapes, resource exhaustion, etc.), the entire application crashes instead of gracefully degrading to unprocessed audio.

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/audio/denoiser/dtln.rs` | Main denoiser implementation with problematic expects |
| `src-tauri/src/audio/denoiser/mod.rs` | Module root, will contain DtlnError type |
| `src-tauri/src/audio/denoiser/shared.rs` | SharedDenoiser wrapper (no changes needed) |
| `src-tauri/src/audio/cpal_backend.rs` | Consumer - calls denoiser.process() in audio callback |

## Fix Approach

1. **Create DtlnError type** - Define error enum with InferenceFailed and InvalidOutput variants
2. **Convert internal methods to Result** - Change run_model_1/run_model_2/process_frame to return Result
3. **Add graceful fallback** - In process()/flush(), catch errors and return raw audio with warning log

The fix follows the existing project pattern seen in `audio/thread.rs` (AudioThreadError) and `listening/pipeline.rs` (PipelineError).

## Regression Risk

| Risk | Mitigation |
|------|------------|
| Changed return types break callers | Only internal methods change; public process()/flush() keep same signature |
| Fallback audio has different timing | Apply Hann window to raw frames for overlap-add consistency |
| Existing tests fail | Tests use normal inference path which will still return Ok |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | Identified 4 expect() calls in dtln.rs | All in run_model_1/run_model_2 methods |
| 2025-12-23 | Confirmed no unsafe code | Fix maintains memory safety |
| 2025-12-23 | Checked error patterns in codebase | thiserror is already used (e.g., listening/manager.rs) |

## Open Questions

- [x] Should errors propagate to caller or be handled internally? **Decision: Handle internally in process()/flush() to maintain API compatibility**
- [x] What audio to return on failure? **Decision: Raw input with Hann window for overlap-add consistency**
