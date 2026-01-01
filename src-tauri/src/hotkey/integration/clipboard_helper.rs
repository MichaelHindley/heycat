//! Clipboard and paste simulation helpers.

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Simulate Cmd+V paste keystroke on macOS using CoreGraphics
#[cfg(target_os = "macos")]
pub fn simulate_paste() -> Result<(), String> {
    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        crate::debug!("Skipping paste - app is shutting down");
        return Ok(());
    }

    // Centralized synthesis ensures key-up always follows key-down and sequences don't interleave.
    crate::keyboard::synth::simulate_cmd_v_paste()?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_paste() -> Result<(), String> {
    Err("Paste simulation only supported on macOS".to_string())
}

/// Copy text to clipboard and auto-paste
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn copy_and_paste(app_handle: &Option<AppHandle>, text: &str) {
    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        crate::debug!("Skipping copy_and_paste - app is shutting down");
        return;
    }

    if let Some(ref handle) = app_handle {
        if let Err(e) = handle.clipboard().write_text(text) {
            crate::warn!("Failed to copy to clipboard: {}", e);
        } else {
            crate::debug!("Transcribed text copied to clipboard");
            if let Err(e) = simulate_paste() {
                crate::warn!("Failed to auto-paste: {}", e);
            } else {
                crate::debug!("Auto-pasted transcribed text");
            }
        }
    } else {
        crate::warn!("Clipboard unavailable: no app handle configured");
    }
}
