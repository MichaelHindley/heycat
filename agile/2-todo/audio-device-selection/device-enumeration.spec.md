---
status: pending
created: 2025-12-15
completed: null
dependencies: []
---

# Spec: Audio Device Enumeration Backend

## Description

Implement the Rust backend infrastructure to enumerate available audio input devices on the system using CPAL. This creates the foundation for all device selection features by exposing a Tauri command that returns a list of audio input devices with their properties.

## Acceptance Criteria

- [ ] `AudioInputDevice` struct defined with `name: String` and `is_default: bool` fields
- [ ] `list_input_devices()` function returns `Vec<AudioInputDevice>` using CPAL host enumeration
- [ ] Tauri command `list_audio_devices` exposed and callable from frontend via `invoke`
- [ ] Command returns empty array (not error) when no devices available
- [ ] Command returns device list sorted with default device first
- [ ] Unit tests cover: device listing, empty device case, default device identification

## Test Cases

- [ ] `test_list_input_devices_returns_vec` - Function returns a Vec (may be empty on CI)
- [ ] `test_audio_input_device_struct` - Struct serializes correctly to JSON for Tauri
- [ ] `test_list_devices_includes_default_flag` - At least one device has `is_default: true` when devices exist
- [ ] `test_list_devices_default_first` - Default device is first in returned list
- [ ] Integration: Frontend can call `list_audio_devices` and receive typed response

## Dependencies

None - this is the foundation spec.

## Preconditions

- CPAL crate already in `Cargo.toml` (version 0.15)
- Tauri command infrastructure exists (`src-tauri/src/commands/`)

## Implementation Notes

**Files to create/modify:**

1. **`src-tauri/src/audio/mod.rs`** - Add structs and public function:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AudioInputDevice {
       pub name: String,
       pub is_default: bool,
   }

   pub fn list_input_devices() -> Vec<AudioInputDevice> {
       // Use cpal::default_host().input_devices()
       // Map to AudioInputDevice structs
       // Sort with default device first
   }
   ```

2. **`src-tauri/src/commands/mod.rs`** - Add Tauri command:
   ```rust
   #[tauri::command]
   pub fn list_audio_devices() -> Vec<AudioInputDevice> {
       crate::audio::list_input_devices()
   }
   ```

3. **`src-tauri/src/lib.rs`** - Register command in `invoke_handler`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       commands::list_audio_devices,
   ])
   ```

4. **`src-tauri/src/audio/cpal_backend.rs`** - Add helper if needed for device iteration

**CPAL Usage Pattern:**
```rust
use cpal::traits::{DeviceTrait, HostTrait};

let host = cpal::default_host();
let default_device = host.default_input_device();
let default_name = default_device.map(|d| d.name().unwrap_or_default());

let devices: Vec<AudioInputDevice> = host
    .input_devices()
    .map(|devices| {
        devices
            .filter_map(|device| {
                device.name().ok().map(|name| AudioInputDevice {
                    is_default: Some(&name) == default_name.as_ref(),
                    name,
                })
            })
            .collect()
    })
    .unwrap_or_default();
```

**Error Handling:** Return empty vec on errors - don't propagate CPAL errors to frontend. Log errors internally.

## Related Specs

- `device-selection-backend.spec.md` - Uses device list to find specific device
- `device-selector-ui.spec.md` - Frontend consumes this command

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs` - Tauri command handler
- Connects to: Frontend via Tauri invoke, CPAL audio subsystem

## Integration Test

- Test location: Frontend test calling `invoke('list_audio_devices')`
- Verification: [ ] Integration test passes
