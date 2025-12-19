// Text input action - types text using macOS keyboard simulation

use crate::keyboard_capture::permissions::check_accessibility_permission;
use crate::voice_commands::executor::{Action, ActionError, ActionErrorCode, ActionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventTapLocation};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Default delay between key presses in milliseconds
pub const DEFAULT_TYPING_DELAY_MS: u64 = 10;

/// Type a single character using CGEvent keyboard simulation
#[cfg(target_os = "macos")]
fn type_character(source: &CGEventSource, character: char) -> Result<(), ActionError> {
    // For Unicode characters, we use CGEventKeyboardSetUnicodeString
    // Encode the character to UTF-16
    let mut buf = [0u16; 2];
    let slice = character.encode_utf16(&mut buf);
    let chars: Vec<u16> = slice.to_vec();

    // Defensive check: ensure UTF-16 encoding produced at least one code unit
    // This should never fail for valid Rust chars, but guard against edge cases
    if chars.is_empty() {
        return Err(ActionError {
            code: ActionErrorCode::EncodingError,
            message: format!("Failed to encode character '{}' to UTF-16", character),
        });
    }

    // Create a key down event with a dummy keycode (we'll set the unicode string)
    let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
        .map_err(|_| ActionError {
            code: ActionErrorCode::EventError,
            message: "Failed to create keyboard event".to_string(),
        })?;

    // Set the unicode string for this event (safe: chars is non-empty and valid UTF-16)
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
        code: ActionErrorCode::UnsupportedPlatform,
        message: "Text input is only supported on macOS".to_string(),
    })
}

/// Type a string of text with configurable delay between characters
#[cfg(target_os = "macos")]
fn type_text_with_delay(text: &str, delay_ms: u64) -> Result<(), ActionError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| ActionError {
            code: ActionErrorCode::EventSourceError,
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
        code: ActionErrorCode::UnsupportedPlatform,
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
            code: ActionErrorCode::InvalidParameter,
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

        // Check Accessibility permission first (blocking call, but quick)
        let has_permission = tokio::task::spawn_blocking(check_accessibility_permission)
            .await
            .map_err(|e| ActionError {
                code: ActionErrorCode::TaskPanic,
                message: format!("Permission check task panicked: {}", e),
            })?;

        if !has_permission {
            return Err(ActionError {
                code: ActionErrorCode::PermissionDenied,
                message: "Accessibility permission not granted. Please enable it in System Preferences > Security & Privacy > Privacy > Accessibility".to_string(),
            });
        }

        // Get optional delay parameter (default to DEFAULT_TYPING_DELAY_MS)
        let delay_ms = parameters
            .get("delay_ms")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TYPING_DELAY_MS);

        // Clone text for the blocking task
        let text_owned = text.clone();
        let char_count = text.chars().count();

        // Run blocking keyboard simulation on a dedicated thread pool
        // This prevents blocking the tokio async runtime
        tokio::task::spawn_blocking(move || {
            type_text_with_delay(&text_owned, delay_ms)
        })
        .await
        .map_err(|e| ActionError {
            code: ActionErrorCode::TaskPanic,
            message: format!("Text input task panicked: {}", e),
        })??;

        Ok(ActionResult {
            message: format!("Typed {} characters", char_count),
            data: Some(serde_json::json!({
                "typed": text,
                "length": char_count
            })),
        })
    }
}

#[cfg(test)]
#[path = "text_input_test.rs"]
mod tests;
