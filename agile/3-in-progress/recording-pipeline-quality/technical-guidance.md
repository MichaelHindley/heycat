---
last-updated: 2025-12-23
status: validated
---

# Technical Guidance: Recording Pipeline Quality

## Architecture Overview

This feature improves the audio recording pipeline to address quality issues (muffled sound, low volume, artifacts) while maintaining cross-platform compatibility. The approach adds standard audio processing stages used in professional voice recording systems.

### Current Pipeline (heycat)
```
Microphone → cpal → [Resampler] → [DTLN Denoiser] → Ring Buffer → WAV
```

### Target Pipeline
```
Microphone → cpal (fixed buffer) → Channel Mixer → Highpass → Pre-emphasis → Resampler → Denoiser → AGC → Diagnostics → Ring Buffer → WAV
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use cpal (not native CoreAudio) | Cross-platform requirement (Windows/Linux support) | 2025-12-23 |
| Add pre-emphasis filter | VoiceInk investigation showed muffled audio needs high-freq boost | 2025-12-23 |
| Fixed buffer size 256 | Balance between latency and stability | 2025-12-23 |
| IIR biquad for highpass | Efficient, low latency, standard choice | 2025-12-23 |
| AGC with envelope tracking | Prevents "pumping" artifacts on transients | 2025-12-23 |
| Keep sinc resampler | Higher quality than FFT for speech | 2025-12-23 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | VoiceInk uses AVAudioRecorder (native macOS) | Explains quality difference; we must optimize cpal instead |
| 2025-12-23 | VoiceInk has no denoiser, captures clean audio | Our DTLN may contribute to artifacts; keep it optional |
| 2025-12-23 | VoiceInk normalizes by dividing by max absolute value | Simple effective approach; our AGC is more sophisticated |
| 2025-12-23 | cpal uses platform default buffer sizes | May cause glitches; fixed 256 samples recommended |
| 2025-12-23 | Current pipeline lacks pre-emphasis | Causes muffled speech; add 0.97 coefficient filter |

## VoiceInk Comparison

### Architecture Differences

| Aspect | VoiceInk | heycat |
|--------|----------|--------|
| Platform | macOS only (Swift) | Cross-platform (Rust) |
| Capture API | AVAudioRecorder | cpal abstraction |
| Device Control | Direct CoreAudio APIs | cpal DeviceTrait |
| Noise Suppression | None | DTLN ONNX denoiser |
| Sample Format | 16-bit PCM direct | f32 → 16-bit |
| Buffer Config | OS-managed | Platform defaults |
| Resampling | AVAudioConverter | rubato FFT/sinc |

### VoiceInk Recording Settings (Reference)
```swift
AVFormatIDKey: Int(kAudioFormatLinearPCM)
AVSampleRateKey: 16000.0
AVNumberOfChannelsKey: 1
AVLinearPCMBitDepthKey: 16
AVLinearPCMIsFloatKey: false
AVLinearPCMIsBigEndianKey: false
AVLinearPCMIsNonInterleaved: false
```

### Key Takeaways
1. VoiceInk captures "clean" audio without processing - quality comes from native API
2. Our approach adds DSP stages to compensate for cpal abstraction overhead
3. Pre-emphasis is critical for speech clarity (VoiceInk may get this from native codec)

## Open Questions

- [x] Should we maintain cross-platform support? **Yes** (confirmed)
- [x] Is denoiser causing artifacts? **Partially** - improves slightly when disabled
- [x] What specific quality issues? **Muffled, low volume, artifacts**
- [ ] Should dithering be added to WAV encoding? (low priority)

## Files to Modify

### Core Pipeline
- `src-tauri/src/audio/cpal_backend.rs` - Buffer size, channel mixing, filter integration
- `src-tauri/src/audio/preprocessing.rs` - **NEW**: Highpass + pre-emphasis filters
- `src-tauri/src/audio/agc.rs` - **NEW**: Automatic gain control
- `src-tauri/src/audio/diagnostics.rs` - **NEW**: Quality metrics

### Configuration
- `src-tauri/src/audio_constants.rs` - New constants (HIGHPASS_CUTOFF_HZ, PRE_EMPHASIS_ALPHA, PREFERRED_BUFFER_SIZE, AGC_* params)
- `src-tauri/src/audio/mod.rs` - Export new modules

### Dependencies
- `src-tauri/Cargo.toml` - Add `biquad` crate for IIR filters

## Processing Order

The filter chain order is critical for optimal results:

1. **Channel Mixing** - Convert stereo to mono first (all subsequent stages expect mono)
2. **Highpass Filter** - Remove DC offset and rumble before any frequency-dependent processing
3. **Pre-emphasis** - Boost high frequencies before resampling (preserves detail)
4. **Resampling** - Convert to 16kHz for VAD/transcription
5. **Denoiser** - DTLN operates on clean, pre-emphasized signal
6. **AGC** - Normalize levels after denoising (amplifies clean signal)
7. **Diagnostics** - Track final output quality

## Constants Reference

```rust
// Preprocessing
pub const HIGHPASS_CUTOFF_HZ: f32 = 80.0;
pub const PRE_EMPHASIS_ALPHA: f32 = 0.97;

// Buffer configuration
pub const PREFERRED_BUFFER_SIZE: u32 = 256;

// AGC parameters
pub const AGC_TARGET_RMS_DBFS: f32 = -12.0;
pub const AGC_ATTACK_MS: f32 = 10.0;
pub const AGC_RELEASE_MS: f32 = 150.0;
pub const AGC_MAX_GAIN_DB: f32 = 20.0;
pub const AGC_SOFT_LIMIT_DBFS: f32 = -3.0;
```

## References

- [VoiceInk Source](file:///Users/michaelhindley/Documents/git/VoiceInk/VoiceInk/Recorder.swift) - Reference implementation
- [rubato crate](https://crates.io/crates/rubato) - High-quality resampling
- [biquad crate](https://crates.io/crates/biquad) - IIR filter implementation
- [cpal BufferSize docs](https://docs.rs/cpal/latest/cpal/enum.BufferSize.html) - Buffer configuration
