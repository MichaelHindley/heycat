---
status: pending
created: 2025-12-14
completed: null
dependencies:
  - wake-word-detector
  - listening-state-machine
---

# Spec: Continuous background audio capture

## Description

Configure the audio system for continuous capture during listening mode. Implement a circular buffer that feeds samples to the wake word detector without accumulating full recordings. Handle microphone availability changes gracefully.

## Acceptance Criteria

- [ ] Audio thread supports continuous capture mode (separate from recording mode)
- [ ] Fixed-size circular buffer implemented for wake word analysis window (~3 seconds)
- [ ] Samples routed to wake word detector, not main recording buffer
- [ ] Microphone unavailability detected and reported via event
- [ ] Listening pauses gracefully when mic unavailable, resumes when available
- [ ] Memory usage bounded by circular buffer size (~192KB for 3s @ 16kHz)

## Test Cases

- [ ] Continuous capture runs without memory growth
- [ ] Wake word detector receives samples in real-time
- [ ] Microphone disconnect triggers `listening_unavailable` event
- [ ] Microphone reconnect resumes listening automatically
- [ ] Recording mode takes priority over listening capture
- [ ] Listening resumes after recording completes

## Dependencies

- wake-word-detector (consumes samples)
- listening-state-machine (controls when pipeline is active)

## Preconditions

- Audio thread functional
- Wake word detector implemented

## Implementation Notes

- Implement `CircularBuffer` in `src-tauri/src/listening/buffer.rs`:
  ```rust
  struct CircularBuffer {
      buffer: Vec<f32>,
      write_pos: usize,
      capacity: usize, // ~48000 samples for 3s @ 16kHz
  }
  ```
- May need separate audio stream or shared stream with routing
- Use same sample rate as recording (16kHz) for simplicity
- Investigate cpal's ability to detect device changes

## Related Specs

- wake-word-detector.spec.md (receives samples from this pipeline)
- listening-state-machine.spec.md (controls pipeline activation)

## Integration Points

- Production call site: `src-tauri/src/audio/thread.rs`, `src-tauri/src/listening/buffer.rs`
- Connects to: listening/detector.rs, recording/state.rs

## Integration Test

- Test location: `src-tauri/src/listening/buffer_test.rs`
- Verification: [ ] Integration test passes
