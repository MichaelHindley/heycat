// Text input action - types text using macOS keyboard simulation

use crate::voice_commands::executor::{Action, ActionError, ActionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Default delay between key presses in milliseconds
pub const DEFAULT_TYPING_DELAY_MS: u64 = 10;

/// Check if Accessibility permission is granted on macOS
#[cfg(target_os = "macos")]
fn check_accessibility_permission() -> bool {
    // Link to the ApplicationServices framework for AXIsProcessTrusted
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility_permission() -> bool {
    // On non-macOS, we don't have this permission system
    true
}

/// Type a single character using CGEvent keyboard simulation
#[cfg(target_os = "macos")]
fn type_character(source: &CGEventSource, character: char) -> Result<(), ActionError> {
    // For Unicode characters, we use CGEventKeyboardSetUnicodeString
    // Encode the character to UTF-16
    let mut buf = [0u16; 2];
    let slice = character.encode_utf16(&mut buf);
    let chars: Vec<u16> = slice.to_vec();

    // Create a key down event with a dummy keycode (we'll set the unicode string)
    let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
        .map_err(|_| ActionError {
            code: "EVENT_ERROR".to_string(),
            message: "Failed to create keyboard event".to_string(),
        })?;

    // Set the unicode string for this event
    event.set_string_from_utf16_unchecked(&chars);

    // Post the key down event
    event.post(CGEventTapLocation::HID);

    // Small delay to ensure the event is processed
    thread::sleep(Duration::from_millis(1));

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn type_character(_source: &(), _character: char) -> Result<(), ActionError> {
    Err(ActionError {
        code: "UNSUPPORTED_PLATFORM".to_string(),
        message: "Text input is only supported on macOS".to_string(),
    })
}

/// Type a string of text with configurable delay between characters
#[cfg(target_os = "macos")]
fn type_text_with_delay(text: &str, delay_ms: u64) -> Result<(), ActionError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| ActionError {
            code: "EVENT_SOURCE_ERROR".to_string(),
            message: "Failed to create event source".to_string(),
        })?;

    for character in text.chars() {
        type_character(&source, character)?;

        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn type_text_with_delay(_text: &str, _delay_ms: u64) -> Result<(), ActionError> {
    Err(ActionError {
        code: "UNSUPPORTED_PLATFORM".to_string(),
        message: "Text input is only supported on macOS".to_string(),
    })
}

/// Action to type text into the currently focused application
pub struct TextInputAction;

impl TextInputAction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextInputAction {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Action for TextInputAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let text = parameters.get("text").ok_or_else(|| ActionError {
            code: "INVALID_PARAMETER".to_string(),
            message: "Missing 'text' parameter".to_string(),
        })?;

        // Empty text is a no-op, return success
        if text.is_empty() {
            return Ok(ActionResult {
                message: "No text to type".to_string(),
                data: Some(serde_json::json!({
                    "typed": "",
                    "length": 0
                })),
            });
        }

        // Check Accessibility permission first
        if !check_accessibility_permission() {
            return Err(ActionError {
                code: "PERMISSION_DENIED".to_string(),
                message: "Accessibility permission not granted. Please enable it in System Preferences > Security & Privacy > Privacy > Accessibility".to_string(),
            });
        }

        // Get optional delay parameter (default to DEFAULT_TYPING_DELAY_MS)
        let delay_ms = parameters
            .get("delay_ms")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TYPING_DELAY_MS);

        // Type the text (this is blocking, run in spawn_blocking if needed)
        type_text_with_delay(text, delay_ms)?;

        Ok(ActionResult {
            message: format!("Typed {} characters", text.chars().count()),
            data: Some(serde_json::json!({
                "typed": text,
                "length": text.chars().count()
            })),
        })
    }
}

#[cfg(test)]
#[path = "text_input_test.rs"]
mod tests;
