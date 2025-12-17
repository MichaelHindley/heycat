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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_not_found_display() {
        let err = AudioDeviceError::DeviceNotFound {
            device_name: "My Mic".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Selected device 'My Mic' is not available"
        );
    }

    #[test]
    fn test_no_devices_available_display() {
        let err = AudioDeviceError::NoDevicesAvailable;
        assert_eq!(err.to_string(), "No audio input devices detected");
    }

    #[test]
    fn test_device_disconnected_display() {
        let err = AudioDeviceError::DeviceDisconnected;
        assert_eq!(err.to_string(), "Device disconnected during recording");
    }

    #[test]
    fn test_capture_error_display() {
        let err = AudioDeviceError::CaptureError {
            message: "Stream failed".to_string(),
        };
        assert_eq!(err.to_string(), "Audio capture failed: Stream failed");
    }

    #[test]
    fn test_serialize_device_not_found() {
        let err = AudioDeviceError::DeviceNotFound {
            device_name: "USB Mic".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"deviceNotFound\""));
        assert!(json.contains("\"deviceName\":\"USB Mic\""));
    }

    #[test]
    fn test_serialize_no_devices_available() {
        let err = AudioDeviceError::NoDevicesAvailable;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"noDevicesAvailable\""));
    }

    #[test]
    fn test_serialize_device_disconnected() {
        let err = AudioDeviceError::DeviceDisconnected;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"deviceDisconnected\""));
    }

    #[test]
    fn test_serialize_capture_error() {
        let err = AudioDeviceError::CaptureError {
            message: "test error".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"captureError\""));
        assert!(json.contains("\"message\":\"test error\""));
    }
}
