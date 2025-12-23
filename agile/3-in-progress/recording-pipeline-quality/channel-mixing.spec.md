---
status: completed
created: 2025-12-23
completed: 2025-12-23
dependencies: []
review_round: 1
---

# Spec: Proper stereo-to-mono conversion for multi-channel devices

## Description

Implement proper stereo-to-mono channel mixing for audio input devices that provide multi-channel audio. Currently, the audio callback receives multi-channel data but the resampler is configured for single-channel processing, which may result in dropped channels or incorrect mixing.

## Acceptance Criteria

- [ ] Detect the number of channels from the audio device configuration
- [ ] When stereo (2 channels), mix to mono by averaging: `mono = (left + right) / 2`
- [ ] Apply -3dB gain compensation when summing channels to prevent clipping
- [ ] Handle devices with more than 2 channels (select first 2 or average all)
- [ ] Preserve mono input unchanged (no processing overhead)
- [ ] Channel mixing happens before resampling in the callback pipeline

## Test Cases

- [ ] Mono input (1 channel) passes through unchanged
- [ ] Stereo input (2 channels) is correctly mixed to mono
- [ ] Stereo sine wave at 0dB results in mono output at approximately -3dB
- [ ] Multi-channel input (4+ channels) is handled without panics
- [ ] Mixed output maintains correct sample count (input_samples / channels)

## Dependencies

None (foundational spec)

## Preconditions

- Audio capture pipeline is functional
- Access to `cpal_backend.rs` audio callback code

## Implementation Notes

- Modify `CallbackState::process_samples()` to accept raw device samples before channel mixing
- Add channel count to `CallbackState` struct
- Create `mix_to_mono(samples: &[f32], channels: usize) -> Vec<f32>` utility function
- The cpal `config.channels()` provides the channel count

## Related Specs

- `resampler-quality-upgrade.spec.md` - depends on this (resampler receives mono)
- `audio-preprocessing.spec.md` - depends on this (preprocessing operates on mono)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Connects to: Audio callback → Channel mixer → Resampler pipeline

## Integration Test

- Test location: `src-tauri/src/audio/cpal_backend.rs` (integration test module)
- Verification: [x] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Pre-Review Gates (Automated)

**1. Build Warning Check:**
```
warning: associated function `with_worktree_context` is never used (wav.rs:63)
warning: associated items `with_default_path` and `get` are never used (store.rs:86)
warning: associated function `with_config` is never used (coordinator.rs:49)
warning: associated function `new` is never used (voice_commands/mod.rs:20)
warning: associated function `with_default_path` is never used (registry.rs:103)
```
**PASS** - All warnings are pre-existing from other modules (wav.rs, store.rs, coordinator.rs, voice_commands). No new unused code warnings from channel-mixing implementation.

**2. Command Registration Check:** N/A - No new Tauri commands added.

**3. Event Subscription Check:** N/A - No new events added.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Detect the number of channels from the audio device configuration | PASS | `cpal_backend.rs:549` - `let channel_count = config.channels();` |
| When stereo (2 channels), mix to mono by averaging: `mono = (left + right) / 2` | PASS | `cpal_backend.rs:184-188` - Averages channels per frame |
| Apply -3dB gain compensation when summing channels to prevent clipping | PASS | `cpal_backend.rs:182` - `const GAIN_COMPENSATION: f32 = 0.7071067811865476;` (1/sqrt(2)) |
| Handle devices with more than 2 channels (select first 2 or average all) | PASS | `cpal_backend.rs:186` - Averages ALL channels in the frame |
| Preserve mono input unchanged (no processing overhead) | PASS | `cpal_backend.rs:171-174` - Early return for mono (returns samples.to_vec()) |
| Channel mixing happens before resampling in the callback pipeline | PASS | `cpal_backend.rs:240-246` - Step 1 is channel mixing, Step 2 is resampling |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Mono input (1 channel) passes through unchanged | PASS | `cpal_backend.rs:1142-1149` - `test_mix_to_mono_preserves_mono` |
| Stereo input (2 channels) is correctly mixed to mono | PASS | `cpal_backend.rs:1152-1167` - `test_mix_to_mono_stereo_mixing` |
| Stereo sine wave at 0dB results in mono output at approximately -3dB | PASS | `cpal_backend.rs:1170-1186` - `test_mix_to_mono_gain_compensation` |
| Multi-channel input (4+ channels) is handled without panics | PASS | `cpal_backend.rs:1189-1216` - `test_mix_to_mono_multichannel` |
| Mixed output maintains correct sample count (input_samples / channels) | PASS | `cpal_backend.rs:1219-1235` - `test_mix_to_mono_sample_count` |

### Manual Review (6 Questions)

**1. Is the code wired up end-to-end?**
- [x] `mix_to_mono()` is called from `CallbackState::process_samples()` at line 243
- [x] `process_samples()` is called from all three audio stream callbacks (F32, I16, U16) at lines 651, 677, 694, 719, 739, 767
- [x] `channel_count` is stored in `CallbackState` at line 631, read from device config at line 549
- [x] Logging confirms multi-channel detection at lines 558-563

**2. What would break if this code was deleted?**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `mix_to_mono()` | fn | `cpal_backend.rs:243` | YES - audio callback pipeline |
| `channel_count` field | struct field | `cpal_backend.rs:242-243` | YES - used in process_samples |
| `GAIN_COMPENSATION` | const | `cpal_backend.rs:188` | YES - applied to every multi-channel sample |

**3. Where does the data flow?**
```
[Audio Device Callback] F32/I16/U16 stream
     |
     v
[process_samples()] src-tauri/src/audio/cpal_backend.rs:230
     | Step 1: mix_to_mono() at line 243
     v
[Mono Samples]
     | Step 2: Resampling at line 249
     v
[Resampled Samples]
     | Step 3: Denoising at line 301
     v
[Ring Buffer] at line 325
```

**4. Are there any deferrals?**
No deferrals found in the implementation.

**5. Automated check results**
```
Frontend tests: 344 passed (38 test files)
Backend tests: 469 passed; 0 failed; 9 ignored
Build warnings: 5 pre-existing (none from new code)
```

**6. Frontend-Only Integration Check:** N/A - This is a backend-only spec.

### Code Quality

**Strengths:**
- Clean separation of concerns: `mix_to_mono()` is a pure function with clear documentation
- Proper -3dB gain compensation (0.707 = 1/sqrt(2)) prevents clipping on coherent signals
- Efficient mono passthrough - early return avoids unnecessary processing
- Comprehensive test coverage for all acceptance criteria
- Well-documented processing pipeline order in `process_samples()` comments
- Informative logging when multi-channel devices are detected

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria are met with evidence. The implementation correctly detects channel count from the audio device, mixes multi-channel to mono with -3dB gain compensation, handles 4+ channels by averaging all, preserves mono unchanged, and performs mixing before resampling. All tests pass (344 frontend, 469 backend). No new code warnings or deferrals.
