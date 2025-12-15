---
status: in-progress
created: 2025-12-15
completed: null
dependencies: []
---

# Spec: Unify VAD configuration across components

## Description

Create a unified VAD configuration that is shared between WakeWordDetector (listening) and SilenceDetector (recording). Currently, different thresholds are used (0.3 vs 0.5) without documented rationale, causing inconsistent behavior.

## Acceptance Criteria

- [ ] Create `VadConfig` struct in `src-tauri/src/listening/vad.rs`
- [ ] Document threshold rationale in code comments
- [ ] `WakeWordDetector` uses `VadConfig`
- [ ] `SilenceDetector` uses `VadConfig`
- [ ] Extract VAD initialization to factory function (eliminate duplication)
- [ ] Single threshold value OR documented reason for difference
- [ ] Both listening and recording VAD work correctly

## Test Cases

- [ ] Unit test: VadConfig defaults are sensible
- [ ] Unit test: VAD initializes with custom config
- [ ] Unit test: VAD factory produces working detector
- [ ] Integration test: Wake word VAD detects speech
- [ ] Integration test: Silence VAD detects end of speech

## Dependencies

None - can be done independently

## Preconditions

- Understanding of why current thresholds differ (investigate before implementing)

## Implementation Notes

```rust
// src-tauri/src/listening/vad.rs

/// VAD configuration shared across listening and recording components.
///
/// Threshold rationale:
/// - 0.4 provides good balance between sensitivity and false positive rejection
/// - Lower values (0.3) are more sensitive but may trigger on background noise
/// - Higher values (0.5) are more precise but may miss soft speech
pub struct VadConfig {
    /// Speech probability threshold (0.0-1.0)
    /// Default: 0.4 - balanced for typical indoor environments
    pub speech_threshold: f32,

    /// Audio sample rate in Hz
    pub sample_rate: u32,

    /// Chunk size for VAD processing (must match Silero model)
    pub chunk_size: usize,

    /// Minimum speech frames before considering speech detected
    pub min_speech_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            speech_threshold: 0.4,
            sample_rate: 16000,
            chunk_size: 512,  // Required by Silero VAD
            min_speech_frames: 2,
        }
    }
}

/// Factory function for creating VAD detector
pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(config.chunk_size)
        .build()
        .map_err(VadError::InitializationFailed)
}
```

Key changes:
- `listening/detector.rs:229-239` - Use `create_vad()` factory
- `listening/silence.rs:80-84` - Use `create_vad()` factory
- Remove duplicate initialization code

## Related Specs

- `extract-duplicate-code.spec.md` - Related (both reduce duplication)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs`, `src-tauri/src/listening/silence.rs`
- Connects to: `WakeWordDetector`, `SilenceDetector`

## Integration Test

- Test location: `src-tauri/src/listening/vad_test.rs`
- Verification: [ ] Integration test passes
