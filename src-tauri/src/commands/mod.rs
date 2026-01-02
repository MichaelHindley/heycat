//! Tauri IPC commands module
//!
//! This module contains Tauri-specific command wrappers and is excluded from coverage.
//! The actual logic is in logic.rs which is fully tested.
//!
//! ## Module Organization
//!
//! - `recording`: Recording commands (start, stop, list, delete)
//! - `transcription`: Transcription commands
//! - `audio`: Audio device commands
//! - `hotkey`: Hotkey management commands
//! - `dictionary`: Dictionary management commands
//! - `window_context`: Window context commands
//! - `common`: Shared utilities (TauriEventEmitter)
//! - `logic`: Core command logic (testable)

#![cfg_attr(coverage_nightly, coverage(off))]

pub mod audio;
pub mod common;
pub mod dictionary;
pub mod hotkey;
pub mod logic;
pub mod recording;
pub mod transcription;
pub mod window_context;

// Re-export TauriEventEmitter from common module for backward compatibility
pub use common::TauriEventEmitter;

// Re-export state type aliases from app::state for backward compatibility
pub use crate::app::state::{
    AudioMonitorState, AudioThreadState, HotkeyIntegrationState, HotkeyServiceState,
    KeyboardCaptureState, ProductionState, TranscriptionServiceState, TursoClientState,
};

// Worktree commands
use tauri::State;

/// Get the settings file name for the current worktree context
#[tauri::command]
pub fn get_settings_file_name(
    worktree_state: State<'_, crate::worktree::WorktreeState>,
) -> String {
    worktree_state.settings_file_name()
}

/// Show the main window, close the splash window, and give main focus
///
/// Called by the frontend when the app is ready to be displayed (e.g., after
/// initialization completes). This enables a seamless splash-to-app transition.
///
/// Includes error recovery with retry logic for splash window operations.
#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    // Show the main window first (before closing splash) for smoother UX
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;

    crate::info!("Main window shown and focused");

    // Close the splash window with retry logic
    if let Some(splash) = app_handle.get_webview_window("splash") {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        const RETRY_DELAY_MS: u64 = 50;

        loop {
            attempts += 1;
            match splash.close() {
                Ok(()) => {
                    crate::debug!("Splash window closed");
                    break;
                }
                Err(e) => {
                    if attempts >= MAX_ATTEMPTS {
                        // Log warning but don't fail - main window is already visible
                        crate::warn!(
                            "Failed to close splash window after {} attempts: {}",
                            attempts,
                            e
                        );
                        break;
                    }
                    crate::debug!(
                        "Splash close attempt {} failed, retrying: {}",
                        attempts,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
