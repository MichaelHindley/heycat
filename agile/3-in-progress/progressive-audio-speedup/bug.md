---
status: pending
severity: major
origin: manual
created: 2025-12-22
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Progressive Audio Speedup

**Created:** 2025-12-22
**Owner:** Claude
**Severity:** Major

## Problem Description

Audio progressively speeds up with each recording when resampling is active. First recording is fine but subsequent recordings play back faster. Occurs when device requires resampling (48kHz to 16kHz). Root cause: Rubato FFT resampler residual sample handling in cpal_backend.rs.

## Steps to Reproduce

1. Launch the app with an audio device that doesn't support 16kHz natively (most devices use 48kHz)
2. Make a recording using the hotkey - first recording plays back at normal speed
3. Make a second recording - audio plays back slightly faster
4. Make additional recordings - each subsequent recording plays progressively faster
5. After 5-10 recordings, audio is noticeably sped up with metallic/robotic sound

## Root Cause

The rubato `FftFixedIn` resampler in `cpal_backend.rs` has residual sample handling issues:

1. **Residual samples discarded**: When recording stops, samples remaining in `resample_buf` (< RESAMPLE_CHUNK_SIZE of 1024) are discarded instead of flushed through the resampler
2. **Sample count drift**: The FFT-based resampler may produce slightly more/fewer samples than the mathematical ratio predicts due to windowing and phase accumulation
3. **No flush mechanism**: `stop()` at line 350-361 drops the stream without flushing the resampler's internal state

Key code locations:
- `cpal_backend.rs:119` - resample_buf accumulates samples
- `cpal_backend.rs:123-146` - chunk processing loop leaves residuals
- `cpal_backend.rs:350-361` - stop() with no residual flush

## Fix Approach

1. Add diagnostic logging to track input/output sample counts and verify resample ratio
2. Flush residual samples when recording stops (zero-pad final chunk or use rubato partial processing)
3. Verify sample ratio consistency across multiple recordings

## Acceptance Criteria

- [ ] Bug no longer reproducible after 10+ consecutive recordings
- [ ] Root cause addressed - residual samples properly flushed
- [ ] Sample count ratio verified within 0.1% of expected
- [ ] Diagnostic logging added to track sample counts
- [ ] Transcription quality remains consistent across recordings

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| 10 consecutive recordings with resampling | All recordings play at same speed | [ ] |
| Sample ratio logging verification | output/input = 16000/device_rate (within 0.1%) | [ ] |
| Transcription quality consistency | Same quality across all recordings | [ ] |
