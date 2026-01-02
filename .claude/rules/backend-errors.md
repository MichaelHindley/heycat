---
paths: "src-tauri/src/**/*.rs"
---

# Backend Error Handling Pattern

## Custom Error Types

Use enum-based error types with manual `Display` and `Error` implementations (no `thiserror`):

```rust
// GOOD: Manual Display + Error impl (audio/error.rs)
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AudioDeviceError {
    DeviceNotFound { device_name: String },
    NoDevicesAvailable,
    DeviceDisconnected,
    CaptureError { message: String },
}

impl std::fmt::Display for AudioDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioDeviceError::DeviceNotFound { device_name } => {
                write!(f, "Selected device '{}' is not available", device_name)
            }
            AudioDeviceError::NoDevicesAvailable => {
                write!(f, "No audio input devices detected")
            }
            // ... other variants
        }
    }
}

impl std::error::Error for AudioDeviceError {}
```

## Result<T, String> for Commands

Tauri commands return `Result<T, String>` since Tauri serializes errors as strings:

```rust
#[tauri::command]
pub fn start_recording(
    state: State<'_, ProductionState>,
) -> Result<(), String> {
    start_recording_impl(state.as_ref())
        .map_err(|e| e.to_string())
}
```

Internal functions can use typed errors that convert to `String` at the command boundary.

## Error Marker Constants

Use marker constants to identify error categories without fragile string matching:

```rust
// In logic.rs
/// Error identifier for microphone access failures.
/// Used to detect microphone-related errors without fragile string matching.
pub const MICROPHONE_ERROR_MARKER: &str = "[MICROPHONE_ACCESS_ERROR]";

// When creating the error
Err(format!(
    "{} Could not access the microphone. Please check permissions.",
    MICROPHONE_ERROR_MARKER
))

// When checking the error
if err_msg.contains(MICROPHONE_ERROR_MARKER) {
    emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error);
}
```

## Logging Macros

Use `crate::*!` logging macros throughout the backend (re-exported from `tauri_plugin_log`):

```rust
// In lib.rs (crate root)
pub use tauri_plugin_log::log::{debug, error, info, trace, warn};

// In any module file (not lib.rs)
crate::debug!("Current state: {:?}", state);
crate::info!("Recording started at {}Hz", sample_rate);
crate::warn!("Device not found, using default");
crate::error!("Failed to start recording: {}", e);
```

**Note:** The `lib.rs` file uses macros directly without `crate::` prefix since they are defined there.

## Error Handling in State Operations

For mutex lock errors, provide user-friendly messages:

```rust
let manager = state.lock().map_err(|_| {
    crate::error!("Failed to acquire recording state lock");
    "Unable to access recording state. Please try again or restart the application."
})?;
```

## Serializable Errors for Frontend

When errors need to be sent to the frontend, use `#[serde(tag = "type")]` for discriminated union serialization:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RecordingStateError {
    InvalidTransition { from: RecordingState, to: RecordingState },
    NoAudioBuffer,
}
```

This serializes as `{ "type": "invalidTransition", "from": "Idle", "to": "Processing" }`.

## Anti-Patterns

### Using thiserror

```rust
// BAD: thiserror adds macro complexity
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Something went wrong: {0}")]
    SomethingWrong(String),
}

// GOOD: Manual impl is explicit and simple
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Something went wrong: {}", self.0)
    }
}
impl std::error::Error for MyError {}
```

### String matching for error types

```rust
// BAD: Fragile string matching
if err_msg.contains("microphone") && err_msg.contains("access") {
    // Handle microphone error
}

// GOOD: Use marker constants
if err_msg.contains(MICROPHONE_ERROR_MARKER) {
    // Handle microphone error
}
```

### Panicking on lock errors

```rust
// BAD: Panic on lock failure
let manager = state.lock().unwrap();

// GOOD: Convert to user-friendly error
let manager = state.lock().map_err(|_| {
    "Unable to access recording state."
})?;
```
