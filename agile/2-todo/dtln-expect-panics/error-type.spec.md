---
status: pending
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Create DtlnError type for inference failures

## Description

Create a typed error enum for DTLN denoiser operations, following the project's error handling pattern where internal functions use typed errors. This error type will be used by `run_model_1()` and `run_model_2()` to report inference failures instead of panicking.

## Acceptance Criteria

- [ ] `DtlnError` enum defined with `InferenceFailed(String)` and `InvalidOutput(String)` variants
- [ ] Derives `Debug`, `Clone`, `PartialEq`
- [ ] Implements `thiserror::Error` with `#[error(...)]` messages
- [ ] Exported from `denoiser/mod.rs` for use in dtln.rs
- [ ] Error messages are descriptive (include model name and failure context)

## Test Cases

- [ ] DtlnError::InferenceFailed displays correct error message
- [ ] DtlnError::InvalidOutput displays correct error message
- [ ] Error types implement std::error::Error trait

## Dependencies

None - this is a foundational spec

## Preconditions

- `thiserror` crate is available in Cargo.toml (already present in project)

## Implementation Notes

**File to create/modify:** `src-tauri/src/audio/denoiser/mod.rs`

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DtlnError {
    #[error("DTLN model {model} inference failed: {message}")]
    InferenceFailed { model: &'static str, message: String },

    #[error("DTLN model {model} produced invalid output: {message}")]
    InvalidOutput { model: &'static str, message: String },
}
```

## Related Specs

- `inference-result-handling.spec.md` - consumes this error type
- `graceful-degradation.spec.md` - handles these errors at call site

## Integration Points

- Production call site: `src-tauri/src/audio/denoiser/dtln.rs:run_model_1(), run_model_2()`
- Connects to: denoiser module error handling

## Integration Test

- Test location: N/A (unit-only spec - error type definition)
- Verification: [x] N/A
