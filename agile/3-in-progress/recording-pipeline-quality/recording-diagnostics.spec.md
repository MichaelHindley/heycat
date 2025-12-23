---
status: in-progress
created: 2025-12-23
completed: null
dependencies: ["channel-mixing", "resampler-quality-upgrade", "audio-preprocessing", "audio-gain-normalization"]
review_round: 2
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
| Track and log per-recording metrics: input level (peak/RMS), output level, clipping events, AGC gain applied | PASS | `RecordingDiagnostics` integrated into `CallbackState` at `cpal_backend.rs:243`. `record_input()` called at line 270, `record_output()` at line 370. `log_summary(agc_gain_db)` called at line 499 includes all metrics. |
| Add debug mode to save raw (pre-processing) audio alongside processed audio | FAIL | `raw_audio()` method exists at `diagnostics.rs:311` and raw audio is captured when `HEYCAT_DEBUG_AUDIO` is set (line 190-194), but `raw_audio()` is never called to save the file. No file saving implementation. |
| Emit quality warning events to frontend (e.g., "input too quiet", "clipping detected") | FAIL | Warnings are generated and LOGGED at `cpal_backend.rs:502-508`, but NOT emitted as events. No `RECORDING_QUALITY_WARNING` constant in `events.rs`, no `app_handle.emit()` call, no frontend listener. |
| Log sample count at each pipeline stage to detect data loss | PASS | `log_sample_diagnostics()` at `cpal_backend.rs:479` logs input/output counts and ratio error. `log_summary()` at `diagnostics.rs:320` logs comprehensive metrics including sample counts. |
| Include pipeline stage timing metrics (useful for performance tuning) | FAIL | No timing metrics implemented. `RecordingDiagnostics` struct has no timing fields. No `Instant` or duration tracking. |
| Diagnostics can be enabled/disabled via settings (default: minimal logging) | PASS | Environment variables `HEYCAT_DIAGNOSTICS_VERBOSE` (line 23-25) and `HEYCAT_DEBUG_AUDIO` (line 28-30) control behavior. Default is minimal logging (summary only). |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Quiet recording triggers "input too quiet" warning event | PARTIAL | Unit test at `diagnostics.rs:459` tests `check_warnings()` logic. Warnings logged in production but NOT emitted as events. |
| Clipping input triggers "clipping detected" warning event | PARTIAL | Unit test at `diagnostics.rs:473` tests `check_warnings()` logic. Warnings logged in production but NOT emitted as events. |
| Debug mode saves raw audio file alongside processed audio | MISSING | No test. Raw audio is captured to buffer but never saved to file. |
| Sample counts at pipeline input/output match expected ratios | PASS | Unit tests at `diagnostics.rs:424-445` test `record_input()`/`record_output()`. Production code logs ratio at `cpal_backend.rs:492-494`. |
| Metrics logged correctly for normal recording session | PASS | `log_summary()` called from production at `cpal_backend.rs:499`. Comprehensive logging verified. |
| Disabled diagnostics produce no additional logging/files | PASS | Default behavior (no env vars) produces only summary log. Verbose mode requires `HEYCAT_DIAGNOSTICS_VERBOSE=1`. |

### Code Quality

**Strengths:**
- `RecordingDiagnostics` is now fully integrated into `CallbackState` and production code paths
- Clean separation of concerns - metrics collection in `diagnostics.rs`, integration in `cpal_backend.rs`
- Thread-safe design with atomics and mutexes for hot-path metrics
- Single-warning-per-session pattern prevents log spam
- Good unit test coverage for metrics and warning logic
- Environment variable control allows debugging without code changes

**Concerns:**
- **14 unused warnings** in `cargo check` - some are for preprocessing/AGC (separate specs), but `raw_audio()`, `from_samples()`, and several other methods are still unused
- Quality warnings are logged but not emitted to frontend - breaks the feedback loop to users
- Raw audio saving is half-implemented (captured but never saved)
- Pipeline timing metrics not implemented at all

### Automated Check Results

```
warning: unused import: `preprocessing::PreprocessingChain`
warning: unused import: `agc::AutomaticGainControl`
warning: unused imports: `QualityWarning` and `RecordingDiagnostics`
warning: associated function `from_samples` is never used
warning: method `raw_audio` is never used
```

Note: Some warnings are from other specs (preprocessing, AGC imports). The `QualityWarning` import warning and `raw_audio` method unused indicate incomplete integration.

### Verdict

**NEEDS_WORK** - The core metrics tracking and logging is now integrated and working. However, 3 acceptance criteria are still not met:

1. **Quality warning events not emitted to frontend** (AC #3)
   - Warnings are generated and logged but users never see them
   - Fix: Add `RECORDING_QUALITY_WARNING` event, emit from `check_warnings()` results, add frontend listener

2. **Raw audio debug mode incomplete** (AC #2)
   - Raw audio is captured to buffer but never saved to disk
   - Fix: In `stop()`, if debug enabled, call `raw_audio()` and save via `encode_wav()`

3. **Pipeline timing metrics missing** (AC #5)
   - No timing instrumentation at all
   - Fix: Add `Instant` tracking around processing stages, log stage durations
