---
status: in-progress
created: 2025-12-15
completed: null
dependencies:
  - device-enumeration
---

# Spec: Device Selection Backend Support

## Description

Modify the audio capture backend to accept an optional device name parameter when starting capture. This enables the frontend to specify which audio input device to use instead of always using the system default. Includes fallback behavior when the specified device is unavailable.

## Acceptance Criteria

- [ ] `AudioCaptureBackend` trait's `start()` method accepts optional `device_name: Option<String>` parameter
- [ ] `CpalBackend::start()` implementation finds and uses specified device by name
- [ ] `find_device_by_name(name: &str) -> Option<Device>` helper function implemented
- [ ] When device not found, falls back to default device (not error)
- [ ] `AudioCommand::Start` enum variant updated to include `device_name: Option<String>`
- [ ] `AudioThread` passes device name through to backend
- [ ] Tauri command `start_recording` updated to accept optional device name parameter
- [ ] Listening pipeline passes stored device preference when starting audio
- [ ] Unit tests cover: device found, device not found (fallback), default device usage

## Test Cases

- [ ] `test_find_device_by_name_found` - Returns Some(device) when device exists
- [ ] `test_find_device_by_name_not_found` - Returns None for non-existent device
- [ ] `test_start_with_specific_device` - Capture starts successfully with named device
- [ ] `test_start_with_missing_device_fallback` - Falls back to default when device missing
- [ ] `test_start_with_none_uses_default` - Passing None uses default device
- [ ] `test_audio_command_start_includes_device` - Command serialization includes device field

## Dependencies

- `device-enumeration` - Needs device listing infrastructure

## Preconditions

- `device-enumeration` spec completed
- Existing `AudioCaptureBackend` trait in `src-tauri/src/audio/mod.rs`
- Existing `CpalBackend` implementation in `src-tauri/src/audio/cpal_backend.rs`

## Implementation Notes

**Files to modify:**

1. **`src-tauri/src/audio/mod.rs`** - Update trait:
   ```rust
   pub trait AudioCaptureBackend: Send + 'static {
       fn start(
           &mut self,
           sender: Sender<AudioData>,
           device_name: Option<String>,  // NEW PARAMETER
       ) -> Result<(), AudioError>;
       // ... rest unchanged
   }
   ```

2. **`src-tauri/src/audio/cpal_backend.rs`** - Add device finder and update start:
   ```rust
   use cpal::traits::{DeviceTrait, HostTrait};

   fn find_device_by_name(name: &str) -> Option<cpal::Device> {
       let host = cpal::default_host();
       host.input_devices()
           .ok()?
           .find(|d| d.name().map(|n| n == name).unwrap_or(false))
   }

   impl AudioCaptureBackend for CpalBackend {
       fn start(
           &mut self,
           sender: Sender<AudioData>,
           device_name: Option<String>,
       ) -> Result<(), AudioError> {
           let device = device_name
               .as_ref()
               .and_then(|name| find_device_by_name(name))
               .or_else(|| cpal::default_host().default_input_device())
               .ok_or(AudioError::NoInputDevice)?;

           // Log which device is being used
           log::info!("Starting audio capture with device: {:?}", device.name());

           // ... rest of existing start implementation using `device`
       }
   }
   ```

3. **`src-tauri/src/audio/thread.rs`** - Update command enum:
   ```rust
   pub enum AudioCommand {
       Start {
           sender: Sender<AudioData>,
           device_name: Option<String>,  // NEW FIELD
       },
       Stop,
       // ... other variants
   }
   ```

   Update handler:
   ```rust
   AudioCommand::Start { sender, device_name } => {
       backend.start(sender, device_name)?;
   }
   ```

4. **`src-tauri/src/commands/logic.rs`** - Update recording start:
   ```rust
   pub fn start_recording(
       state: &AppState,
       device_name: Option<String>,  // NEW PARAMETER
   ) -> Result<(), AppError> {
       // Pass device_name when sending Start command
   }
   ```

5. **`src-tauri/src/commands/mod.rs`** - Update Tauri command:
   ```rust
   #[tauri::command]
   pub fn start_recording(
       state: State<'_, AppState>,
       device_name: Option<String>,
   ) -> Result<(), String> {
       logic::start_recording(&state, device_name)
           .map_err(|e| e.to_string())
   }
   ```

6. **`src-tauri/src/listening/pipeline.rs`** - Pass device when starting:
   ```rust
   // When starting listening mode, pass the stored device preference
   // This will require reading from settings or accepting as parameter
   ```

**Fallback Strategy:**
1. If `device_name` is `Some(name)` → try to find device by name
2. If device not found → log warning, use default device
3. If no default device → return `AudioError::NoInputDevice`

**Important:** Don't error when specified device is missing - graceful degradation is key for Bluetooth devices that may disconnect.

## Related Specs

- `device-enumeration.spec.md` - Provides device listing (dependency)
- `device-settings-persistence.spec.md` - Frontend stores the device preference
- `device-reconnection.spec.md` - Handles device availability changes

## Integration Points

- Production call site: `src-tauri/src/commands/logic.rs:start_recording()`
- Production call site: `src-tauri/src/listening/pipeline.rs` - Listening mode start
- Connects to: CPAL audio device selection, AudioThread command handling

## Integration Test

- Test location: Manual test with device selection in UI
- Verification: [ ] Integration test passes

**Manual Integration Test Steps:**
1. Start app with USB microphone connected
2. Select USB microphone in settings
3. Start recording - verify USB mic audio captured
4. Disconnect USB mic, start recording - verify falls back to default
