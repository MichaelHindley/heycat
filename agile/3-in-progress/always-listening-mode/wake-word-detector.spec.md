---
status: in-progress
created: 2025-12-14
completed: null
dependencies: []
---

# Spec: Core wake word detection engine

## Description

Implement the core wake word detection engine that analyzes streaming audio to detect the "Hey Cat" phrase. Uses on-device speech recognition via Parakeet with small-window batching for privacy-preserving detection. Emits events when the wake word is confidently detected.

> **MVP Note**: This implementation uses Parakeet (batch transcription model) with small-window batching (~1-2 seconds). CPU/latency optimization deferred to post-MVP.

## Acceptance Criteria

- [ ] `WakeWordDetector` struct created in `src-tauri/src/listening/detector.rs`
- [ ] Processes audio samples using Parakeet in small windows (~1-2 seconds)
- [ ] Detects "Hey Cat" phrase with configurable confidence threshold (default 0.8)
- [ ] Emits `wake_word_detected` event via Tauri event system
- [ ] Thread-safe implementation compatible with audio thread

## Test Cases

- [ ] Correctly detects "Hey Cat" spoken clearly
- [ ] Correctly detects "Hey Cat" with varying intonations
- [ ] Rejects similar phrases ("Hey Matt", "Pay Cat", "Hey")
- [ ] Handles background noise without false triggers
- [ ] Handles silence periods without errors
- [ ] Processes samples without blocking audio capture

## Dependencies

None

## Preconditions

- Parakeet TDT model available and loadable
- Audio capture system functional

## Implementation Notes

- Use Parakeet's `transcribe_file` or similar API with small audio windows
- Small-window batching: accumulate ~1-2 seconds of audio, run transcription, check for wake phrase
- Case-insensitive matching for "hey cat" variants
- Consider fuzzy matching for phonetic variations
- All code in unified `listening/` module

## Related Specs

- listening-audio-pipeline.spec.md (provides audio samples)
- listening-state-machine.spec.md (consumes detection events)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs`
- Connects to: audio thread (receives samples), event system (emits detection)

## Integration Test

- Test location: `src-tauri/src/listening/detector_test.rs`
- Verification: [ ] Integration test passes
