---
status: pending
created: 2025-12-16
completed: null
dependencies: []
priority: P1
---

# Spec: Create centralized audio constants module

## Description

Magic numbers are scattered throughout the audio processing code:
- `512` chunk size (5+ locations)
- `0.3` / `0.5` VAD thresholds
- `8000` samples (0.5s at 16kHz)
- `150ms` analysis interval
- `2.0s` window duration

Create a centralized `audio_constants.rs` module to hold all audio-related constants with documentation explaining their purpose and constraints.

## Acceptance Criteria

- [ ] Create `src-tauri/src/audio_constants.rs` module
- [ ] Define named constants for all magic numbers
- [ ] Add documentation explaining each constant's purpose
- [ ] Note constraints (e.g., chunk size depends on sample rate)
- [ ] Update all files to use constants instead of magic numbers
- [ ] Export constants from main lib.rs

## Test Cases

- [ ] Test constants have correct values
- [ ] Test chunk size calculation matches formula
- [ ] Verify no remaining magic numbers in audio code

## Dependencies

- sample-rate-validation.spec.md (uses OPTIMAL_CHUNK_DURATION_MS)

## Preconditions

None

## Implementation Notes

**New file:** `src-tauri/src/audio_constants.rs`

```rust
//! Centralized constants for audio processing.
//!
//! All audio-related magic numbers should be defined here with
//! documentation explaining their purpose and constraints.

/// Sample rate used throughout the audio pipeline (Hz).
/// Silero VAD only supports 8000 or 16000 Hz.
pub const DEFAULT_SAMPLE_RATE: u32 = 16000;

/// Optimal chunk duration for VAD processing (milliseconds).
/// Silero VAD works best with 32ms windows.
pub const OPTIMAL_CHUNK_DURATION_MS: u32 = 32;

/// Chunk size for VAD at 16kHz: 16000 * 32 / 1000 = 512 samples.
pub const VAD_CHUNK_SIZE_16KHZ: usize = 512;

/// Chunk size for VAD at 8kHz: 8000 * 32 / 1000 = 256 samples.
pub const VAD_CHUNK_SIZE_8KHZ: usize = 256;

/// VAD speech threshold for wake word detection.
/// Lower threshold = more sensitive to speech.
pub const VAD_THRESHOLD_WAKE_WORD: f32 = 0.3;

/// VAD speech threshold for silence/end-of-speech detection.
/// Higher threshold = more confident speech is present.
pub const VAD_THRESHOLD_SILENCE: f32 = 0.5;

/// Default analysis interval for wake word pipeline (milliseconds).
/// How often we analyze accumulated audio for wake word.
pub const ANALYSIS_INTERVAL_MS: u64 = 150;

/// Minimum new samples before analysis (at 16kHz).
/// 8000 samples = 0.5 seconds of audio.
pub const MIN_NEW_SAMPLES_FOR_ANALYSIS: usize = 8000;

/// Wake word detection window duration (seconds).
/// How much audio history to analyze for wake word.
pub const WAKE_WORD_WINDOW_SECS: f32 = 2.0;

/// Fingerprint overlap threshold for duplicate detection.
/// Audio segments with >50% overlap are considered duplicates.
pub const FINGERPRINT_OVERLAP_THRESHOLD: f32 = 0.5;

/// Transcription timeout for wake word detection (seconds).
pub const TRANSCRIPTION_TIMEOUT_SECS: u64 = 10;

/// Calculate chunk size for a given sample rate.
pub const fn chunk_size_for_sample_rate(sample_rate: u32) -> usize {
    (sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
}
```

**Files to update:**
- `src-tauri/src/listening/vad.rs` - Use VAD_THRESHOLD_*, chunk_size_for_sample_rate
- `src-tauri/src/listening/detector.rs` - Use all detector constants
- `src-tauri/src/listening/silence.rs` - Use silence detection constants
- `src-tauri/src/listening/pipeline.rs` - Use ANALYSIS_INTERVAL_MS
- `src-tauri/src/listening/buffer.rs` - Reference constants in capacity comments
- `src-tauri/src/lib.rs` - Export the module

## Related Specs

- sample-rate-validation.spec.md (uses OPTIMAL_CHUNK_DURATION_MS)
- unified-vad-config.spec.md (completed - defines some thresholds)

## Integration Points

- N/A - This is a utility module with constants only
- Connects to: All audio processing modules

## Integration Test

- N/A (compile-time verification via usage)
- Verification: [x] N/A
