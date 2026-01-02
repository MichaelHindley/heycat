//! Windows/Linux-specific initialization.
//!
//! Uses rdev backend for hotkey registration with key release detection.

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::app::state::{HotkeyIntegrationState, HotkeyServiceState, ProductionState};
use crate::events::{self, RecordingErrorPayload};
use crate::hotkey::{RecordingMode, ShortcutBackendExt};

/// Register hotkey with press and release callbacks using rdev backend.
///
/// Returns true if registration was successful.
pub fn register_hotkey_with_release(
    service: &HotkeyServiceState,
    shortcut: &str,
    integration: HotkeyIntegrationState,
    recording_state: ProductionState,
    app_handle: AppHandle,
    recording_mode: RecordingMode,
) -> bool {
    crate::info!(
        "Registering global hotkey: {} (initial mode: {:?})...",
        shortcut,
        recording_mode
    );

    let integration_press = integration.clone();
    let state_press = recording_state.clone();
    let app_handle_press = app_handle.clone();

    let integration_release = integration.clone();
    let state_release = recording_state.clone();
    let app_handle_release = app_handle.clone();

    // Try rdev backend (Windows/Linux)
    if let Some(ext_backend) = service
        .backend
        .as_any()
        .downcast_ref::<crate::hotkey::RdevShortcutBackend>()
    {
        if let Err(e) = ext_backend.register_with_release(
            shortcut,
            Box::new(move || {
                // Clone for the async task - the callback must return immediately
                // to avoid blocking the rdev event loop
                let integration = integration_press.clone();
                let state = state_press.clone();
                let app_handle = app_handle_press.clone();

                // Spawn the heavy work on Tauri's async runtime
                tauri::async_runtime::spawn(async move {
                    match integration.lock() {
                        Ok(mut guard) => {
                            let mode = guard.recording_mode();
                            crate::debug!("Hotkey pressed (mode: {:?})", mode);
                            match mode {
                                RecordingMode::Toggle => {
                                    guard.handle_toggle(&state);
                                }
                                RecordingMode::PushToTalk => {
                                    guard.handle_hotkey_press(&state);
                                }
                            }
                        }
                        Err(e) => {
                            crate::error!("Failed to acquire integration lock: {}", e);
                            let _ = app_handle.emit(
                                events::event_names::RECORDING_ERROR,
                                RecordingErrorPayload {
                                    message: "Internal error: please restart the application"
                                        .to_string(),
                                },
                            );
                        }
                    }
                });
            }),
            Box::new(move || {
                // Clone for the async task - the callback must return immediately
                // to avoid blocking the rdev event loop
                let integration = integration_release.clone();
                let state = state_release.clone();
                let app_handle = app_handle_release.clone();

                // Spawn the heavy work on Tauri's async runtime
                tauri::async_runtime::spawn(async move {
                    match integration.lock() {
                        Ok(mut guard) => {
                            let mode = guard.recording_mode();
                            crate::debug!("Hotkey released (mode: {:?})", mode);
                            // Only handle release in PTT mode
                            if mode == RecordingMode::PushToTalk {
                                guard.handle_hotkey_release(&state);
                            }
                        }
                        Err(e) => {
                            crate::error!("Failed to acquire integration lock: {}", e);
                            let _ = app_handle.emit(
                                events::event_names::RECORDING_ERROR,
                                RecordingErrorPayload {
                                    message: "Internal error: please restart the application"
                                        .to_string(),
                                },
                            );
                        }
                    }
                });
            }),
        ) {
            crate::warn!("Failed to register hotkey: {:?}", e);
            return false;
        }
        return true;
    }

    crate::warn!("rdev backend not available");
    false
}
