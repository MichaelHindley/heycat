---
status: in-progress
created: 2025-12-23
completed: null
dependencies: ["channel-mixing", "resampler-quality-upgrade", "audio-preprocessing", "audio-gain-normalization"]
review_round: 1
---

# Spec: Quality metrics and diagnostic tooling

## Description

Add comprehensive quality metrics and diagnostic tooling to the audio pipeline. This enables debugging audio quality issues, A/B comparison of processing stages, and provides visibility into pipeline health. Includes logging of key metrics, optional raw/processed audio capture, and quality warnings sent to the frontend.

## Acceptance Criteria

- [ ] Track and log per-recording metrics: input level (peak/RMS), output level, clipping events, AGC gain applied
- [ ] Add debug mode to save raw (pre-processing) audio alongside processed audio
- [ ] Emit quality warning events to frontend (e.g., "input too quiet", "clipping detected")
- [ ] Log sample count at each pipeline stage to detect data loss
- [ ] Include pipeline stage timing metrics (useful for performance tuning)
- [ ] Diagnostics can be enabled/disabled via settings (default: minimal logging)

## Test Cases

- [ ] Quiet recording triggers "input too quiet" warning event
- [ ] Clipping input triggers "clipping detected" warning event
- [ ] Debug mode saves raw audio file alongside processed audio
- [ ] Sample counts at pipeline input/output match expected ratios
- [ ] Metrics logged correctly for normal recording session
- [ ] Disabled diagnostics produce no additional logging/files

## Dependencies

- `channel-mixing` - diagnostics tracks channel mixing stage
- `resampler-quality-upgrade` - diagnostics tracks resampler stage
- `audio-preprocessing` - diagnostics tracks preprocessing stage
- `audio-gain-normalization` - diagnostics tracks AGC gain levels

## Preconditions

- All other pipeline specs are implemented
- Event emission infrastructure exists (app_handle.emit)

## Implementation Notes

- Extend existing `CallbackState::log_sample_diagnostics()` with more metrics
- Create `src-tauri/src/audio/diagnostics.rs` module for metric collection
- Metrics to track:
  - Input peak/RMS level (before processing)
  - Output peak/RMS level (after processing)
  - Clipping count (samples at or near ±1.0)
  - AGC current gain
  - Processing stage latencies
- For debug mode:
  - Save raw audio to separate file with `-raw` suffix
  - Use existing WAV encoding infrastructure
- Frontend events:
  - `recording_quality_warning` with payload: `{ type: "quiet" | "clipping", severity: "info" | "warning" }`

## Related Specs

- All other specs in this feature (provides observability for entire pipeline)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs` (metrics collection throughout pipeline)
- Connects to: All pipeline stages, frontend via events

## Integration Test

- Test location: `src-tauri/src/audio/diagnostics.rs` (unit tests)
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Track and log per-recording metrics: input level (peak/RMS), output level, clipping events, AGC gain applied | FAIL | `RecordingDiagnostics` struct exists in `diagnostics.rs:121` with all metrics, but is NOT used in `cpal_backend.rs`. The `CallbackState` struct does not include `RecordingDiagnostics` and `record_input()`/`record_output()` are never called from production code. |
| Add debug mode to save raw (pre-processing) audio alongside processed audio | FAIL | `RecordingDiagnostics::raw_audio()` method exists at `diagnostics.rs:311`, but it is never called. No code saves raw audio to a file. The `debug_audio_enabled()` function exists but is never called from production paths. |
| Emit quality warning events to frontend (e.g., "input too quiet", "clipping detected") | FAIL | `QualityWarning` struct exists at `diagnostics.rs:54` and `check_warnings()` at `diagnostics.rs:263`, but: (1) No `recording_quality_warning` event constant defined in `events.rs`, (2) No `app_handle.emit()` call for warnings anywhere, (3) No frontend listener for this event. |
| Log sample count at each pipeline stage to detect data loss | DEFERRED | `CallbackState::log_sample_diagnostics()` at `cpal_backend.rs:470` logs input/output counts. However, the new `RecordingDiagnostics` module is not integrated, so per-stage tracking is incomplete. |
| Include pipeline stage timing metrics (useful for performance tuning) | FAIL | No timing metrics implemented anywhere. `RecordingDiagnostics` struct has no timing fields. |
| Diagnostics can be enabled/disabled via settings (default: minimal logging) | FAIL | Environment variables `HEYCAT_DIAGNOSTICS_VERBOSE` and `HEYCAT_DEBUG_AUDIO` exist but (1) not exposed via settings UI, (2) `RecordingDiagnostics` is not used in production so these env vars have no effect. |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Quiet recording triggers "input too quiet" warning event | MISSING | Unit test exists at `diagnostics.rs:459` testing `check_warnings()`, but no integration test verifying event emission to frontend |
| Clipping input triggers "clipping detected" warning event | MISSING | Unit test exists at `diagnostics.rs:473` testing `check_warnings()`, but no integration test verifying event emission to frontend |
| Debug mode saves raw audio file alongside processed audio | MISSING | No test for raw audio file saving |
| Sample counts at pipeline input/output match expected ratios | PASS | `diagnostics.rs:424-445` tests `record_input()`/`record_output()` metrics, but these are unit tests only |
| Metrics logged correctly for normal recording session | MISSING | No integration test - `log_summary()` at `diagnostics.rs:320` is never called from production code |
| Disabled diagnostics produce no additional logging/files | MISSING | No test for disabled state |

### Code Quality

**Strengths:**
- Clean, well-documented `RecordingDiagnostics` struct with comprehensive metrics tracking
- Good use of atomic operations for thread-safe counter updates
- Thorough unit tests for the `LevelMetrics` and `RecordingDiagnostics` classes
- Single-warning-per-session pattern prevents spam

**Concerns:**
- **CRITICAL: Code is entirely orphaned** - `RecordingDiagnostics` is defined and exported but never instantiated or used in production code
- **22 unused code warnings** in `cargo check` - confirms the new code is dead
- `CallbackState` in `cpal_backend.rs` already has its own sample counting (`input_sample_count`, `output_sample_count`) - the new module duplicates this without integration
- No event constant defined for `recording_quality_warning`
- No frontend event listener for quality warnings
- Pipeline timing metrics not implemented
- Raw audio saving not implemented (method exists but never called)

### Verdict

**NEEDS_WORK** - The `RecordingDiagnostics` module is complete in isolation but is completely disconnected from production code. Zero acceptance criteria are met because none of the new code is wired into `cpal_backend.rs` or the event emission system.

**How to Fix:**

1. **Integrate RecordingDiagnostics into CallbackState** (`cpal_backend.rs`):
   - Add `diagnostics: RecordingDiagnostics` field to `CallbackState` struct
   - Call `diagnostics.record_input()` before processing in `process_samples()`
   - Call `diagnostics.record_output()` after AGC in `process_samples()`
   - Call `diagnostics.log_summary(agc_gain)` in `stop()`

2. **Wire up event emission** (`cpal_backend.rs` and `events.rs`):
   - Add `RECORDING_QUALITY_WARNING` constant to `events.rs`
   - Pass `app_handle` to `CallbackState` (or use a callback pattern)
   - Call `app_handle.emit()` when `check_warnings()` returns warnings

3. **Add frontend listener** (`src/lib/eventBridge.ts` or relevant hook):
   - Add listener for `recording_quality_warning` event
   - Surface warnings to user (toast, status indicator, etc.)

4. **Implement raw audio saving**:
   - In `stop()`, if debug mode enabled, call `diagnostics.raw_audio()` and save to file
   - Use existing `encode_wav()` infrastructure

5. **Add pipeline timing metrics**:
   - Add timing fields to `RecordingDiagnostics`
   - Instrument processing stages in `process_samples()`

6. **Fix unused warnings**:
   - Remove `#[allow(dead_code)]` and ensure all public APIs are actually called
