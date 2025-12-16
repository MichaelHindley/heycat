---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
priority: P0
---

# Spec: Add sample rate validation in VAD creation

## Description

The VAD (Silero) only supports 8kHz or 16kHz sample rates, and the chunk size (512) assumes 16kHz (32ms window). Currently there is **no runtime validation** - passing the wrong sample rate causes silent failure. The chunk size should also adapt based on sample rate.

Add explicit validation in `create_vad()` to fail fast with a clear error when an unsupported sample rate is used, and auto-calculate the correct chunk size.

## Acceptance Criteria

- [ ] `create_vad()` returns error for sample rates other than 8000 or 16000
- [ ] Error message clearly states supported sample rates
- [ ] Chunk size is calculated from sample rate (32ms window): `sample_rate * 32 / 1000`
- [ ] VadConfig fields `chunk_size` becomes derived, not user-specified
- [ ] Add constant for optimal chunk duration (32ms)
- [ ] Update all callers if VadConfig API changes

## Test Cases

- [ ] Test `create_vad()` with 8000 Hz succeeds with chunk_size=256
- [ ] Test `create_vad()` with 16000 Hz succeeds with chunk_size=512
- [ ] Test `create_vad()` with 44100 Hz returns clear error
- [ ] Test `create_vad()` with 0 Hz returns clear error
- [ ] Test error message mentions "8000 or 16000"

## Dependencies

None

## Preconditions

- Existing VAD module with `VadConfig` and `create_vad()` function

## Implementation Notes

**File:** `src-tauri/src/listening/vad.rs`

**Current state:**
- Lines 51-128: `VadConfig` struct with `sample_rate` and `chunk_size` fields
- Lines 139-145: `create_vad()` doesn't validate sample rate
- Chunk size hardcoded as 512 in 5+ places (vad.rs:70, silence.rs:150, detector.rs:539)

**Proposed changes:**

```rust
pub const OPTIMAL_CHUNK_DURATION_MS: u32 = 32;

impl VadConfig {
    pub fn chunk_size_for_sample_rate(sample_rate: u32) -> usize {
        (sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
    }
}

pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    // Validate sample rate
    match config.sample_rate {
        8000 | 16000 => {},
        other => return Err(VadError::ConfigurationInvalid(
            format!("Unsupported sample rate: {}. Must be 8000 or 16000 Hz.", other)
        )),
    }

    let chunk_size = VadConfig::chunk_size_for_sample_rate(config.sample_rate);

    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(chunk_size)
        .build()
        .map_err(|e| VadError::InitializationFailed(e.to_string()))
}
```

**Also add new error variant:**
```rust
pub enum VadError {
    InitializationFailed(String),
    ConfigurationInvalid(String),  // NEW
}
```

## Related Specs

- unified-vad-config.spec.md (completed)
- audio-constants-module.spec.md (should define OPTIMAL_CHUNK_DURATION_MS there)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs:271`
- Production call site: `src-tauri/src/listening/silence.rs:90`
- Connects to: WakeWordDetector, SilenceDetector

## Integration Test

- Test location: `src-tauri/src/listening/vad.rs` (test module)
- Verification: [ ] Integration test passes
