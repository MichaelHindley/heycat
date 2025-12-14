---
status: pending
created: 2025-12-14
completed: null
dependencies:
  - listening-audio-pipeline
---

# Spec: Automatic recording stop on silence

## Description

Implement simple energy-based silence detection to automatically stop recording when the user finishes speaking. Handle the case where the wake word was detected but no speech follows (false activation timeout). Distinguish between intentional pauses and end of dictation.

## Acceptance Criteria

- [ ] Silence detection based on audio amplitude (RMS calculation)
- [ ] Configurable silence threshold (default appropriate for speech)
- [ ] Configurable silence duration threshold (default 2 seconds)
- [ ] Recording auto-stops after silence threshold exceeded
- [ ] False activation timeout: cancel if no speech within 5 seconds of wake word
- [ ] Intentional pause detection: brief pauses (< 1s) don't trigger stop
- [ ] `recording_auto_stopped` event emitted with reason (silence vs timeout)
- [ ] Auto-stop triggers transcription pipeline (same as manual stop)

## Test Cases

- [ ] Recording stops after 2 seconds of silence following speech
- [ ] Brief pauses during speech don't stop recording
- [ ] No-speech timeout cancels recording without transcription
- [ ] Varying speech patterns handled correctly
- [ ] Background noise doesn't prevent silence detection
- [ ] Event payload includes correct stop reason

## Dependencies

- listening-audio-pipeline (provides audio samples for silence detection)

## Preconditions

- Audio pipeline delivering samples
- Recording state machine functional

## Implementation Notes

Simple energy-based silence detection approach:
```rust
// In src-tauri/src/listening/silence.rs

/// Calculate RMS (root mean square) of audio samples
fn calculate_rms(samples: &[f32]) -> f32 {
    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Configurable thresholds
struct SilenceDetector {
    silence_threshold: f32,      // RMS below this = silence
    silence_duration_ms: u32,    // How long silence before stop (default 2000)
    no_speech_timeout_ms: u32,   // Cancel if no speech after wake word (default 5000)
    consecutive_silent_frames: u32,
    has_detected_speech: bool,
}
```

- No external dependencies required
- Process samples in frames (~100ms windows)
- Track consecutive silent frames to measure duration
- Different handling for "no speech at all" vs "speech then silence"

## Related Specs

- listening-audio-pipeline.spec.md (provides samples)
- cancel-commands.spec.md (alternative stop mechanism)

## Integration Points

- Production call site: `src-tauri/src/listening/silence.rs`
- Connects to: audio pipeline, recording state machine

## Integration Test

- Test location: `src-tauri/src/listening/silence_test.rs`
- Verification: [ ] Integration test passes
