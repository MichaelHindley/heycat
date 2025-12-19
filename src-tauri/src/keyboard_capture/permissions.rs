// Accessibility permission handling for macOS
// CGEventTap requires Accessibility permission (not Input Monitoring)
// This module provides functions to check and guide users to enable the permission

use std::process::Command;

// FFI bindings for Accessibility permission checking
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Check if the current process has Accessibility permission
    /// Returns true if the app is trusted (has Accessibility permission)
    fn AXIsProcessTrusted() -> bool;
}

/// Check if the application has Accessibility permission
///
/// Returns true if Accessibility is enabled for this app in System Settings.
/// Unlike Input Monitoring, there's no programmatic way to request this permission -
/// the user must manually enable it.
pub fn check_accessibility_permission() -> bool {
    // SAFETY: AXIsProcessTrusted is a safe C function that just checks permission state
    unsafe { AXIsProcessTrusted() }
}

/// Open System Settings to the Accessibility pane
///
/// Opens the Privacy & Security > Accessibility section where users can
/// grant permission to this app.
///
/// Returns Ok(()) if the settings were opened successfully, or an error message.
pub fn open_accessibility_settings() -> Result<(), String> {
    let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to open System Settings: {}", e))?;

    Ok(())
}

/// Error returned when Accessibility permission is not granted
#[derive(Debug, Clone)]
pub struct AccessibilityPermissionError {
    pub message: String,
}

impl std::fmt::Display for AccessibilityPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AccessibilityPermissionError {}

impl AccessibilityPermissionError {
    pub fn new() -> Self {
        Self {
            message: "Accessibility permission required. Please grant permission in System Settings > Privacy & Security > Accessibility, then restart the app.".to_string(),
        }
    }
}

impl Default for AccessibilityPermissionError {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_permission_returns_bool() {
        // This test verifies that the function can be called and returns a boolean
        // The actual return value depends on system state
        let result = check_accessibility_permission();
        // Just verify it's a valid boolean (not a crash or undefined behavior)
        assert!(result == true || result == false);
    }

    #[test]
    fn test_accessibility_permission_error_display() {
        let error = AccessibilityPermissionError::new();
        let display = format!("{}", error);
        assert!(display.contains("Accessibility permission required"));
        assert!(display.contains("System Settings"));
    }

    #[test]
    fn test_accessibility_permission_error_default() {
        let error = AccessibilityPermissionError::default();
        assert!(error.message.contains("Accessibility permission required"));
    }
}
