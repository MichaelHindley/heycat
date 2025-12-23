---
status: implemented
severity: critical
origin: manual
created: 2025-12-23
completed: null
parent_feature: "noise-suppression"
parent_spec: null
---

# Bug: Recordings are hyper-accelerated and compressed to under 1 second regardless of actual recording duration

**Created:** 2025-12-23
**Severity:** Major

## Problem Description

When recording audio, all recordings are hyper-accelerated and compressed. Even if recording for 5 seconds in real time, the resulting audio is under 1 second and sounds very strange/distorted due to the extreme speed-up.

**Expected:** Recording duration matches real-time recording length with normal audio playback speed.

**Actual:** Recordings are compressed to a fraction of their actual duration and play back hyper-accelerated.

## Steps to Reproduce

1. Start the application with noise suppression enabled
2. Record audio for ~5 seconds
3. Play back the recording
4. **Expected:** ~5 second recording at normal speed
5. **Actual:** <1 second recording, hyper-accelerated and distorted

## Root Cause

### PRIMARY: Ring Buffer Not Drained Before WAV Encoding

**Location:** `src-tauri/src/audio/mod.rs` and `src-tauri/src/commands/logic.rs`

The `AudioBuffer` has two storage layers:
1. **Ring buffer** - where `push_samples()` writes (audio callback)
2. **Accumulated Vec** - where `drain_samples()` moves samples, and `lock()` reads from

**The Bug:** In push-to-talk mode, `drain_samples()` is never called before WAV encoding!

Flow in push-to-talk mode:
1. Audio callback → `push_samples()` → ring buffer ✓
2. **NO periodic drain loop** (unlike listening mode which has coordinator loop)
3. On stop: `buffer.lock()` returns nearly empty `accumulated` ✗
4. WAV file is written with few/no samples ✗

Evidence from diagnostics:
- `output_sample_count` shows 19,542 samples pushed to ring buffer (correct)
- But WAV file is <1 second (samples never drained to accumulated)

The comment in `audio/mod.rs:121-124` explicitly warns:
```rust
/// Note: This only accesses accumulated samples, not samples still in ring buffer.
/// Call `drain_samples()` first to ensure all samples are accumulated.
```

But `stop_recording_impl()` in `logic.rs:186-193` calls `buffer.lock()` without draining first!

### Secondary Issues

1. **Denoiser buffers not flushed on recording stop**
   - `flush_residuals()` only flushes the resampler, not the denoiser
   - Up to 511 samples in `input_buffer` and 384 in `output_buffer` are lost

2. **Flushed resampler samples bypass denoiser**
   - Residual samples go directly to ring buffer, skipping denoising

## Fix Approach

### Step 1: CRITICAL - Drain ring buffer before WAV encoding
**File:** `src-tauri/src/commands/logic.rs`

Add `drain_samples()` call before accessing the buffer in `stop_recording_impl()`:
```rust
let buffer = manager.get_audio_buffer().map_err(|_| "No recorded audio available.")?;
let _ = buffer.drain_samples();  // CRITICAL: Move ring buffer to accumulated
let samples = buffer.lock().map_err(|_| "Failed to access audio buffer.")?;
```

### Step 2: Add flush() method to DtlnDenoiser
**File:** `src-tauri/src/audio/denoiser/dtln.rs`

Add method to extract remaining samples from denoiser buffers on recording stop.

### Step 3: Update flush_residuals() to flush denoiser
**File:** `src-tauri/src/audio/cpal_backend.rs`

Route flushed resampler samples through denoiser, then flush denoiser buffers.

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Record 5s in push-to-talk mode, verify WAV duration | WAV file ~5 seconds at normal speed | [ ] |
| Verify drain_samples() called before lock() | Log shows drained sample count matches output_sample_count | [ ] |
| Record with noise suppression, check denoiser flush | All denoiser buffer samples included in output | [ ] |
| Record without noise suppression (control) | Works correctly (baseline comparison) | [ ] |

## Integration Points

- `src-tauri/src/commands/logic.rs:stop_recording_impl()` - Main fix location
- `src-tauri/src/audio/mod.rs:AudioBuffer` - Ring buffer and accumulated storage
- `src-tauri/src/audio/cpal_backend.rs:flush_residuals()` - Denoiser flush integration
- `src-tauri/src/listening/coordinator.rs` - Already has drain loop (verify works correctly)

## Integration Test

1. Run app with noise suppression enabled
2. Use push-to-talk to record for 5+ seconds
3. Verify WAV file duration matches recording time
4. Play back to confirm normal speed audio
