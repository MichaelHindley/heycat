// Audio device enumeration module
// Provides types and functions for listing available audio input devices

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};


/// Represents an audio input device with its properties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioInputDevice {
    /// Human-readable name of the device
    pub name: String,
    /// Whether this is the system's default input device
    pub is_default: bool,
}

/// List all available audio input devices
///
/// Returns a vector of audio input devices sorted with the default device first.
/// Returns an empty vector if no devices are available or if an error occurs.
/// Errors are logged but not propagated to the caller.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn list_input_devices() -> Vec<AudioInputDevice> {
    let host = cpal::default_host();
    crate::debug!("Listing input devices for host: {:?}", host.id());

    // Get the default device name for comparison
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    crate::debug!("Default input device: {:?}", default_name);

    // Get all input devices
    let devices = match host.input_devices() {
        Ok(devices) => devices,
        Err(e) => {
            crate::warn!("Failed to enumerate input devices: {}", e);
            return Vec::new();
        }
    };

    // Map devices to AudioInputDevice structs
    let mut device_list: Vec<AudioInputDevice> = devices
        .filter_map(|device| {
            device.name().ok().map(|name| {
                let is_default = default_name.as_ref() == Some(&name);
                AudioInputDevice { name, is_default }
            })
        })
        .collect();

    // Sort with default device first
    device_list.sort_by(|a, b| b.is_default.cmp(&a.is_default));

    crate::debug!("Found {} input devices", device_list.len());
    device_list
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed per docs/TESTING.md:
    // - test_audio_input_device_struct_serializes_correctly: Serialization derives
    // - test_audio_input_device_clone: Type system guarantee
    // - test_audio_input_device_debug: Debug trait derives
    // - test_list_input_devices_returns_vec: Always-true assertion

    #[test]
    fn test_list_devices_default_first() {
        // Create a mock list and verify sorting logic
        let mut devices = vec![
            AudioInputDevice {
                name: "Device A".to_string(),
                is_default: false,
            },
            AudioInputDevice {
                name: "Device B".to_string(),
                is_default: true,
            },
            AudioInputDevice {
                name: "Device C".to_string(),
                is_default: false,
            },
        ];

        // Apply the same sorting logic used in list_input_devices
        devices.sort_by(|a, b| b.is_default.cmp(&a.is_default));

        // Default device should be first
        assert!(devices[0].is_default);
        assert_eq!(devices[0].name, "Device B");
    }
}
