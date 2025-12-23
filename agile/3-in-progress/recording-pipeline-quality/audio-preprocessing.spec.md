---
status: in-progress
created: 2025-12-23
completed: null
dependencies: ["channel-mixing"]
review_round: 1
review_history:
  - round: 1
    date: 2025-12-23
    verdict: NEEDS_WORK
    failedCriteria: ["Highpass bypassed via configuration flag", "Pre-emphasis bypassed via configuration flag"]
    concerns: ["Configuration bypass flags not implemented - Acceptance criteria explicitly require filters to be bypassed via configuration flag, but no settings integration exists", "Unused method warnings will persist until configuration is wired up"]
---

# Spec: Voice-optimized preprocessing chain

## Description

Add a voice-optimized preprocessing stage to the audio pipeline. This includes:
1. **Highpass filter** (~80Hz) to remove low-frequency rumble (HVAC, traffic, handling noise)
2. **Pre-emphasis filter** (coefficient 0.97) to boost higher frequencies important for speech intelligibility

These filters run early in the pipeline (after channel mixing, before resampling) for maximum effectiveness. Pre-emphasis addresses "muffled" audio by boosting frequencies above ~300Hz where consonants and speech clarity reside.

## Acceptance Criteria

### Highpass Filter
- [ ] Implement highpass filter at ~80Hz cutoff to remove rumble
- [ ] Use IIR biquad filter for efficiency (Butterworth or similar)
- [ ] Filter operates on mono audio after channel mixing
- [ ] Filter state is preserved between callbacks (stateful IIR)
- [ ] Minimal latency added by filter (< 5ms)

### Pre-emphasis Filter
- [ ] Implement pre-emphasis filter: `y[n] = x[n] - alpha * x[n-1]` where alpha = 0.97
- [ ] Apply after highpass filter, before resampling
- [ ] Pre-emphasis state preserved between callbacks
- [ ] Boosts frequencies above ~300Hz for speech clarity

### Configuration
- [ ] Highpass filter can be bypassed via configuration flag
- [ ] Pre-emphasis filter can be bypassed via configuration flag (default: enabled)

## Test Cases

### Highpass Filter
- [ ] Highpass filter removes 50Hz test tone completely
- [ ] Highpass filter passes 200Hz test tone with minimal attenuation (< 1dB)
- [ ] No audible ringing or artifacts on transient signals
- [ ] Filter state resets correctly between recording sessions
- [ ] Bypassed highpass produces identical output to input

### Pre-emphasis Filter
- [ ] Pre-emphasis boosts 1kHz test tone relative to 100Hz (measurable gain difference)
- [ ] Pre-emphasis coefficient 0.97 produces expected frequency response curve
- [ ] Speech recordings sound clearer/crisper with pre-emphasis enabled
- [ ] Filter state resets correctly between recording sessions
- [ ] Bypassed pre-emphasis produces identical output to input

### Integration
- [ ] Combined filters (highpass + pre-emphasis) preserve speech quality
- [ ] Processing chain has minimal audible artifacts

## Dependencies

- `channel-mixing` - preprocessing receives mono audio from channel mixer

## Preconditions

- Audio capture pipeline is functional
- Channel mixing is implemented (spec dependency)

## Implementation Notes

### Module Structure
- Create `src-tauri/src/audio/preprocessing.rs` module
- Store filter states in `CallbackState` struct
- Insert preprocessing call after channel mixing, before resampling

### Highpass Filter
- Use `biquad` crate for efficient IIR filter implementation
- 2nd-order Butterworth, cutoff 80Hz, sample rate 16kHz (or device rate)
- Consider: could also operate at native sample rate before resampling (test both)

### Pre-emphasis Filter
- Simple first-order filter: `y[n] = x[n] - 0.97 * x[n-1]`
- No external crate needed (trivial implementation)
- Store single `prev_sample: f32` for state
- Standard ASR coefficient 0.97 boosts frequencies above ~300Hz
- Apply AFTER highpass (order: channel_mix → highpass → pre_emphasis → resample)

### Constants (add to audio_constants.rs)
```rust
pub const HIGHPASS_CUTOFF_HZ: f32 = 80.0;
pub const PRE_EMPHASIS_ALPHA: f32 = 0.97;
```

## Related Specs

- `channel-mixing.spec.md` - this spec depends on channel mixing
- `audio-gain-normalization.spec.md` - depends on this (gain applied to filtered signal)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Pipeline order: Channel mixer → **Highpass** → **Pre-emphasis** → Resampler → Denoiser → Buffer

## Integration Test

- Test location: `src-tauri/src/audio/preprocessing.rs` (unit tests) + integration test in cpal_backend
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Pre-Review Gates

**1. Build Warning Check:**
```
warning: unused import: `preprocessing::PreprocessingChain`
warning: methods `set_enabled`, `is_enabled`, `reset`, and `process_inplace` are never used (HighpassFilter)
warning: methods `set_enabled`, `is_enabled`, `reset`, and `process_inplace` are never used (PreEmphasisFilter)
warning: methods `set_highpass_enabled`, `set_pre_emphasis_enabled`, `reset`, and `process_inplace` are never used (PreprocessingChain)
```
**PARTIAL PASS** - Core `process()` method is used. Configuration methods exist but are not wired to production.

**2. Command Registration Check:** N/A (no new commands)

**3. Event Subscription Check:** N/A (no new events)

### Manual Review

#### 1. Is the code wired up end-to-end?

- [x] Core preprocessing is called in production: `cpal_backend.rs:254` - `pp.process(&mono_samples)`
- [x] PreprocessingChain is instantiated per recording session: `cpal_backend.rs:629`
- [ ] Configuration bypass flags are NOT wired to settings - `set_highpass_enabled()` and `set_pre_emphasis_enabled()` exist but are never called

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `PreprocessingChain::new()` | struct | `cpal_backend.rs:629` | YES |
| `PreprocessingChain::process()` | fn | `cpal_backend.rs:254` | YES |
| `HighpassFilter::process()` | fn | Called by chain.process() | YES |
| `PreEmphasisFilter::process()` | fn | Called by chain.process() | YES |
| `set_*_enabled()` methods | fn | - | TEST-ONLY |
| `reset()` methods | fn | - | NOT NEEDED (new instance per session) |
| `process_inplace()` methods | fn | - | TEST-ONLY (unused variant) |

The `set_enabled`, `is_enabled` methods are TEST-ONLY because they implement the spec requirement "can be bypassed via configuration flag" but no configuration flag exists in settings.

#### 3. Where does the data flow?

```
[Audio Callback]
     |
     v
[Channel Mixer] mix_to_mono() at cpal_backend.rs:246
     |
     v
[Preprocessing] pp.process(&mono_samples) at cpal_backend.rs:254
     |
     v
[Resampler/Denoiser/Buffer]
```

Data flow is complete for the core processing path.

#### 4. Are there any deferrals?

No TODOs, FIXMEs, or deferrals found in the implementation.

#### 5. Automated check results

```
Tests: 12 passed (test_highpass_*, test_pre_emphasis_*, test_chain_*, test_inplace_*)
Warnings: Configuration methods unused (set_enabled, reset, process_inplace)
```

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Highpass filter at ~80Hz cutoff | PASS | `HIGHPASS_CUTOFF_HZ: f32 = 80.0` in audio_constants.rs:217 |
| IIR biquad filter (Butterworth) | PASS | Uses `biquad` crate with `Q_BUTTERWORTH_F32` in preprocessing.rs:31 |
| Filter operates on mono after channel mixing | PASS | Pipeline order verified in cpal_backend.rs:246-254 |
| Filter state preserved between callbacks | PASS | `Arc<Mutex<PreprocessingChain>>` in CallbackState |
| Minimal latency (< 5ms) | PASS | IIR filters have essentially zero latency (sample-by-sample) |
| Pre-emphasis formula y[n] = x[n] - 0.97*x[n-1] | PASS | preprocessing.rs:180, verified by test_pre_emphasis_formula |
| Pre-emphasis after highpass, before resample | PASS | Chain applies highpass then pre_emphasis in process() |
| Pre-emphasis state preserved | PASS | `prev_sample` stored in struct, in Arc<Mutex<>> |
| Highpass bypassed via configuration flag | FAIL | Method exists but not wired to settings |
| Pre-emphasis bypassed via configuration flag | FAIL | Method exists but not wired to settings |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Highpass removes 50Hz | PASS | test_highpass_removes_low_frequency |
| Highpass passes 200Hz (<1dB loss) | PASS | test_highpass_passes_speech_frequencies |
| No ringing/artifacts | DEFERRED | Requires subjective audio testing |
| Filter state resets between sessions | PASS | New instance per session (cpal_backend.rs:629) |
| Bypassed highpass = input | PASS | test_highpass_bypass |
| Pre-emphasis boosts 1kHz vs 100Hz | PASS | test_pre_emphasis_boosts_high_frequencies |
| Pre-emphasis coefficient verified | PASS | test_pre_emphasis_formula |
| Speech clarity with pre-emphasis | DEFERRED | Requires subjective audio testing |
| Bypassed pre-emphasis = input | PASS | test_pre_emphasis_bypass |
| Combined filters preserve quality | PASS | test_chain_applies_both_filters |
| Minimal audible artifacts | DEFERRED | Requires subjective audio testing |

### Code Quality

**Strengths:**
- Well-documented module with clear docstrings
- Comprehensive unit test coverage (12 tests)
- Correct use of `biquad` crate for efficient IIR filtering
- Proper state management via Arc<Mutex<>>
- Constants properly defined in audio_constants.rs
- Both `process()` and `process_inplace()` variants provided

**Concerns:**
- Configuration bypass flags not implemented - Acceptance criteria explicitly require filters to be bypassed via configuration flag, but no settings integration exists
- Unused method warnings will persist until configuration is wired up

### Verdict

**NEEDS_WORK** - Configuration bypass flags required by acceptance criteria are not implemented

The core preprocessing functionality (highpass + pre-emphasis) is correctly implemented and integrated into the audio pipeline. However, the spec explicitly requires:
- "Highpass filter can be bypassed via configuration flag"
- "Pre-emphasis filter can be bypassed via configuration flag (default: enabled)"

The methods to enable/disable filters exist (`set_highpass_enabled`, `set_pre_emphasis_enabled`) but there is no settings integration to actually bypass the filters from the UI or backend settings.

**To fix:**
1. Add settings keys (e.g., `audio.highpassEnabled`, `audio.preEmphasisEnabled`) to the settings system
2. Wire up the settings to call `set_highpass_enabled()` and `set_pre_emphasis_enabled()` when creating the PreprocessingChain
3. This will also resolve the "unused method" warnings
