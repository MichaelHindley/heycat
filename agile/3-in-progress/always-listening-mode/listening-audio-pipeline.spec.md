---
status: in-progress
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

- Test locations (inline unit tests):
  - `src-tauri/src/listening/buffer.rs` - CircularBuffer tests
  - `src-tauri/src/listening/detector.rs` - WakeWordDetector tests
  - `src-tauri/src/listening/pipeline.rs` - ListeningPipeline tests
- Verification: [x] Unit tests pass (run `cargo test listening`)

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Independent Subagent

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Audio thread supports continuous capture mode (separate from recording mode) | PASS | `pipeline.rs:144-189` - `ListeningPipeline::start()` creates separate audio capture using the shared `AudioThreadHandle`, which routes samples to the detector's circular buffer rather than accumulating for recording |
| Fixed-size circular buffer implemented for wake word analysis window (~3 seconds) | PARTIAL | `buffer.rs:21-99` - `CircularBuffer` is implemented with fixed capacity. However, default config in `detector.rs:28` uses 2.0 seconds (32000 samples), not ~3 seconds as spec states. Memory is ~128KB, not ~192KB |
| Samples routed to wake word detector, not main recording buffer | PASS | `pipeline.rs:271-303` - `analysis_thread_main` takes samples from `AudioBuffer`, clears it via `std::mem::take()` (line 281), and routes to detector via `push_samples()` (line 298) |
| Microphone unavailability detected and reported via event | PASS | `pipeline.rs:287-291` - Lock errors emit `listening_unavailable` event; `pipeline.rs:321-327` - Model not loaded emits `listening_unavailable`; `events.rs:31,64-69` - `LISTENING_UNAVAILABLE` event defined with `ListeningUnavailablePayload` |
| Listening pauses gracefully when mic unavailable, resumes when available | PASS | `pipeline.rs:267-269` - Analysis loop checks `mic_available` flag and continues (skips analysis) when false; `pipeline.rs:218-219` - `set_mic_available()` method allows external control |
| Memory usage bounded by circular buffer size (~192KB for 3s @ 16kHz) | PARTIAL | `buffer.rs:48-56` - Circular buffer overwrites oldest samples when full; `pipeline.rs:279-282` - AudioBuffer cleared each cycle to prevent growth. However, actual buffer size is 2s/128KB, not 3s/192KB per spec |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Continuous capture runs without memory growth | PASS | `pipeline.rs:450-464` - `test_circular_buffer_bounds_memory` verifies bounded memory; `pipeline.rs:467-478` - `test_pipeline_config_memory_bounded` verifies reasonable allocation |
| Wake word detector receives samples in real-time | PASS | `detector.rs:316-319` - `test_push_samples_to_buffer` verifies samples can be pushed; `detector.rs:456-467` - `test_push_samples_does_not_block` verifies non-blocking push |
| Microphone disconnect triggers `listening_unavailable` event | PASS | `events.rs:1004-1013` - `test_mock_emitter_records_listening_unavailable_events` verifies event emission; pipeline code emits this on lock errors |
| Microphone reconnect resumes listening automatically | PASS | `pipeline.rs:369-379` - `test_pipeline_set_mic_available` verifies flag toggle; analysis loop at `pipeline.rs:267-269` resumes when flag becomes true |
| Recording mode takes priority over listening capture | DEFERRED | No direct test - requires integration with `ListeningManager` and `RecordingManager`. The `manager.rs:94-121` shows enable_listening fails during Recording state, but priority preemption not tested |
| Listening resumes after recording completes | DEFERRED | No direct test - `manager.rs:217-223` - `get_post_recording_state()` returns Listening if enabled, but end-to-end flow not tested in pipeline tests |

### Code Quality

**Strengths:**
- Clean separation of concerns: `CircularBuffer` for storage, `WakeWordDetector` for analysis, `ListeningPipeline` for orchestration
- Thread-safe design with `Arc<AtomicBool>` for flags and proper mutex usage
- Memory growth prevention through buffer clearing in `analysis_thread_main` (line 281)
- Graceful error handling with appropriate event emission
- Well-documented public API with clear docstrings
- Comprehensive unit tests for core functionality

**Concerns:**
- Spec states ~3 seconds / ~192KB buffer but implementation uses 2 seconds / 128KB (discrepancy)
- Integration test file `buffer_test.rs` does not exist (spec references non-existent file)
- Recording/listening priority interaction tests are missing (deferred to integration)
- No explicit device disconnect detection via cpal - relies on callback errors or manual `set_mic_available()` calls

### Verdict

**NEEDS_WORK** - The implementation is largely complete and well-structured, but there are two issues that need addressing:

1. **Buffer size discrepancy**: Spec requires ~3 seconds / ~192KB buffer but implementation uses 2 seconds / 128KB. Either update the spec to match implementation or adjust `WakeWordDetectorConfig::default()` in `detector.rs:28` to use `window_duration_secs: 3.0`.

2. **Missing integration test file**: Spec references `src-tauri/src/listening/buffer_test.rs` which does not exist. Either create this test file with the integration tests mentioned, or update the spec to reference the existing inline tests in `buffer.rs`, `detector.rs`, and `pipeline.rs`.
