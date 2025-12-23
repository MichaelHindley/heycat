---
status: in-progress
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Configure cpal audio buffer size for reduced latency and glitches

## Description

Configure cpal's audio stream with an explicit buffer size instead of relying on platform defaults. Currently, `build_input_stream()` is called with `None` for the buffer size, letting the OS choose. This can cause variable latency, potential dropouts, and audio glitches.

Setting a fixed buffer size (256 samples) provides more consistent timing and may reduce artifacts on some systems.

## Acceptance Criteria

- [ ] Request specific buffer size (256 samples) via `StreamConfig::buffer_size`
- [ ] Handle fallback gracefully if platform rejects the requested size
- [ ] Add `PREFERRED_BUFFER_SIZE` constant to `audio_constants.rs`
- [ ] Log actual buffer size used (may differ from requested)
- [ ] No increase in audio dropouts or CPU usage
- [ ] Configuration flag to adjust buffer size (for troubleshooting)

## Test Cases

- [ ] Audio capture works with requested buffer size (256)
- [ ] Audio capture falls back gracefully if 256 is rejected
- [ ] Buffer size is logged at stream creation
- [ ] No audible glitches in test recordings
- [ ] Performance: callback processing completes within buffer period

## Dependencies

None (can be implemented independently)

## Preconditions

- Audio capture pipeline is functional
- cpal 0.15+ supports `BufferSize::Fixed`

## Implementation Notes

### Changes to cpal_backend.rs

Currently (around line 523):
```rust
device.build_input_stream(
    &config.into(),
    // ...
)
```

Should become:
```rust
let mut stream_config: cpal::StreamConfig = config.into();
stream_config.buffer_size = cpal::BufferSize::Fixed(PREFERRED_BUFFER_SIZE);

device.build_input_stream(
    &stream_config,
    // ...
)
```

### Buffer Size Selection

| Buffer Size | Latency @ 16kHz | Latency @ 48kHz | Notes |
|-------------|-----------------|-----------------|-------|
| 128 | 8ms | 2.7ms | Very low latency, higher CPU |
| 256 | 16ms | 5.3ms | Good balance (recommended) |
| 512 | 32ms | 10.7ms | Lower CPU, higher latency |

### Constant (add to audio_constants.rs)
```rust
/// Preferred audio buffer size for consistent timing.
/// 256 samples = ~16ms at 16kHz, ~5ms at 48kHz.
/// Smaller values reduce latency but increase CPU usage.
pub const PREFERRED_BUFFER_SIZE: u32 = 256;
```

### Error Handling

cpal may reject the requested buffer size on some platforms. If `BufferSize::Fixed` fails:
1. Log warning with details
2. Fall back to `BufferSize::Default`
3. Continue with platform-chosen buffer

## Related Specs

- All other specs benefit from consistent timing
- Independent of other specs (no dependencies)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:build_input_stream()` (multiple locations for different sample formats)
- Connects to: Audio callback processing

## Integration Test

- Test location: Manual A/B testing with existing recordings
- Verification: [ ] Integration test passes
