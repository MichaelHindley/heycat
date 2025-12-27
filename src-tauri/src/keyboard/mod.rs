// Keyboard simulation module - provides cross-platform keyboard simulation
// Uses Core Graphics on macOS (for consistency with paste simulation)
// Uses enigo crate for Windows and Linux support

pub mod synth;

#[cfg(not(target_os = "macos"))]
use enigo::{Enigo, Key, Keyboard, Settings};

/// Keyboard simulator for sending key events
pub struct KeyboardSimulator {
    #[cfg(not(target_os = "macos"))]
    enigo: Enigo,
}

impl KeyboardSimulator {
    /// Create a new KeyboardSimulator
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create keyboard simulator: {}", e))?;
        Ok(Self { enigo })
    }

    /// Simulate an Enter/Return keypress
    ///
    /// On macOS, delegates to synth module which uses Session tap location for reliable
    /// cross-app event delivery. Includes shutdown checks and mutex serialization.
    ///
    /// Returns Ok(()) on success, Err with message on failure.
    #[cfg(target_os = "macos")]
    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        synth::simulate_enter_keypress()
    }

    #[cfg(not(target_os = "macos"))]
    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        // Small delay to ensure previous typing is complete
        std::thread::sleep(std::time::Duration::from_millis(50));

        self.enigo
            .key(Key::Return, enigo::Direction::Click)
            .map_err(|e| format!("Failed to simulate enter keypress: {}", e))
    }
}

impl Default for KeyboardSimulator {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyboardSimulator")
    }
}

#[cfg(test)]
#[path = "keyboard_test.rs"]
mod tests;
