// Keyboard simulation module - provides cross-platform keyboard simulation
// Uses enigo crate for macOS, Windows, and Linux support

use enigo::{Enigo, Key, Keyboard, Settings};

/// Keyboard simulator for sending key events
pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    /// Create a new KeyboardSimulator
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create keyboard simulator: {}", e))?;
        Ok(Self { enigo })
    }

    /// Simulate an Enter/Return keypress
    ///
    /// Includes a small delay to ensure previous typing is complete before sending the key.
    /// Returns Ok(()) on success, Err with message on failure.
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
