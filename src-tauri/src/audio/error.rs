// Audio device error types for frontend communication
// These provide structured error information for user-facing error dialogs

use serde::Serialize;

/// Specific error types for audio device failures, emitted to the frontend
/// These provide structured error information for user-facing error dialogs
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AudioDeviceError {
    /// Selected device is not available (disconnected or not found)
    #[serde(rename_all = "camelCase")]
    DeviceNotFound { device_name: String },
    /// No audio input devices detected on the system
    NoDevicesAvailable,
    /// Device disconnected during active recording
    DeviceDisconnected,
    /// Generic capture error with details (includes permission errors on macOS)
    #[serde(rename_all = "camelCase")]
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
            AudioDeviceError::DeviceDisconnected => {
                write!(f, "Device disconnected during recording")
            }
            AudioDeviceError::CaptureError { message } => {
                write!(f, "Audio capture failed: {}", message)
            }
        }
    }
}

impl std::error::Error for AudioDeviceError {}

// Tests removed per docs/TESTING.md:
// - Display trait tests: "if it compiles, it works"
// - Serialization tests: "Derive macros handle this correctly"
