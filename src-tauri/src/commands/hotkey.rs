//! Hotkey management commands for Tauri IPC.
//!
//! Contains commands for registering, updating, and managing global hotkeys.

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;

use crate::events::{event_names, RecordingErrorPayload};
use crate::hotkey::RecordingMode;

use super::common::get_settings_file;
use super::{HotkeyIntegrationState, HotkeyServiceState, KeyboardCaptureState, ProductionState};

/// Suspend the global recording shortcut
///
/// Temporarily unregisters the recording shortcut to allow the webview to capture
/// keyboard events (e.g., when recording a new shortcut in settings).
#[tauri::command]
pub fn suspend_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
) -> Result<(), String> {
    let settings_file = get_settings_file(&app_handle);
    let shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    if let Some(shortcut) = shortcut {
        crate::info!("Suspending recording shortcut: {}", shortcut);
        service.backend.unregister(&shortcut).map_err(|e| e.to_string())
    } else {
        crate::info!("No recording shortcut to suspend");
        Ok(())
    }
}

/// Resume the global recording shortcut
///
/// Re-registers the recording shortcut after it was suspended.
#[tauri::command]
pub fn resume_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
    integration: State<'_, HotkeyIntegrationState>,
    recording_state: State<'_, ProductionState>,
) -> Result<(), String> {
    let settings_file = get_settings_file(&app_handle);
    let shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| "No recording shortcut configured".to_string())?;

    crate::info!("Resuming recording shortcut: {}", shortcut);

    let integration_clone = integration.inner().clone();
    let state_clone = recording_state.inner().clone();
    let app_handle_clone = app_handle.clone();

    service
        .backend
        .register(
            &shortcut,
            Box::new(move || {
                crate::debug!("Hotkey pressed!");
                match integration_clone.lock() {
                    Ok(mut guard) => {
                        guard.handle_toggle(&state_clone);
                    }
                    Err(e) => {
                        crate::error!("Failed to acquire integration lock: {}", e);
                        let _ = app_handle_clone.emit(
                            event_names::RECORDING_ERROR,
                            RecordingErrorPayload {
                                message: "Internal error: please restart the application".to_string(),
                            },
                        );
                    }
                }
            }),
        )
        .map_err(|e| e.to_string())
}

/// Update the global recording shortcut
///
/// Unregisters the current shortcut and registers a new one.
#[tauri::command]
pub fn update_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
    integration: State<'_, HotkeyIntegrationState>,
    recording_state: State<'_, ProductionState>,
    new_shortcut: String,
) -> Result<(), String> {
    crate::info!("Updating recording shortcut to: {}", new_shortcut);

    let settings_file = get_settings_file(&app_handle);
    let current_shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Unregister current shortcut if one exists
    if let Some(ref current) = current_shortcut {
        if let Err(e) = service.backend.unregister(current) {
            crate::warn!("Failed to unregister old shortcut '{}': {}", current, e);
        }
    }

    let integration_clone = integration.inner().clone();
    let state_clone = recording_state.inner().clone();
    let app_handle_clone = app_handle.clone();

    // Register new shortcut
    service
        .backend
        .register(
            &new_shortcut,
            Box::new(move || {
                crate::debug!("Hotkey pressed!");
                match integration_clone.lock() {
                    Ok(mut guard) => {
                        guard.handle_toggle(&state_clone);
                    }
                    Err(e) => {
                        crate::error!("Failed to acquire integration lock: {}", e);
                        let _ = app_handle_clone.emit(
                            event_names::RECORDING_ERROR,
                            RecordingErrorPayload {
                                message: "Internal error: please restart the application".to_string(),
                            },
                        );
                    }
                }
            }),
        )
        .map_err(|e| format!("Failed to register new shortcut: {}", e))?;

    // Save to settings
    if let Ok(store) = app_handle.store(&settings_file) {
        store.set("hotkey.recordingShortcut", serde_json::json!(new_shortcut));
        if let Err(e) = store.save() {
            crate::warn!("Failed to persist settings: {}", e);
        }
    }

    crate::info!("Recording shortcut updated successfully to: {}", new_shortcut);
    Ok(())
}

/// Get the current recording shortcut from settings
#[tauri::command]
pub fn get_recording_shortcut(app_handle: AppHandle) -> String {
    let settings_file = get_settings_file(&app_handle);
    app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// Get the current recording mode from settings
#[tauri::command]
pub fn get_recording_mode(app_handle: AppHandle) -> RecordingMode {
    let settings_file = get_settings_file(&app_handle);
    app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("shortcuts.recordingMode"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Set the recording mode in settings
///
/// Rejects if recording is currently active.
#[tauri::command]
pub fn set_recording_mode(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    integration: State<'_, HotkeyIntegrationState>,
    mode: RecordingMode,
) -> Result<(), String> {
    // Check if recording is active
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;

    let current_state = manager.get_state();
    if current_state != crate::recording::RecordingState::Idle {
        return Err("Cannot change recording mode while recording is active.".to_string());
    }
    drop(manager);

    // Update HotkeyIntegration in memory
    let mut integration_guard = integration.lock().map_err(|_| {
        "Unable to access hotkey integration. Please try again or restart the application."
    })?;
    integration_guard.set_recording_mode(mode);
    drop(integration_guard);

    // Persist to settings
    let settings_file = get_settings_file(&app_handle);
    if let Ok(store) = app_handle.store(&settings_file) {
        store.set(
            "shortcuts.recordingMode",
            serde_json::to_value(&mode).unwrap_or_default(),
        );
        if let Err(e) = store.save() {
            crate::warn!("Failed to persist settings: {}", e);
            return Err(format!("Failed to save settings: {}", e));
        }
    } else {
        return Err("Failed to access settings store.".to_string());
    }

    crate::info!("Recording mode updated to: {:?}", mode);
    Ok(())
}

/// Start capturing keyboard events for shortcut recording
///
/// Uses CGEventTap to capture all keyboard events including fn key and media keys.
#[tauri::command]
pub fn start_shortcut_recording(
    app_handle: AppHandle,
    capture_state: State<'_, KeyboardCaptureState>,
) -> Result<(), String> {
    crate::info!("Starting shortcut recording...");

    let mut capture = capture_state.lock().map_err(|e| e.to_string())?;

    if capture.is_running() {
        return Err("Shortcut recording is already running".to_string());
    }

    let app_handle_clone = app_handle.clone();
    capture.start(move |event| {
        if let Err(e) = app_handle_clone.emit(event_names::SHORTCUT_KEY_CAPTURED, &event) {
            crate::warn!("Failed to emit shortcut_key_captured event: {}", e);
        }
    })?;

    crate::info!("Shortcut recording started");
    Ok(())
}

/// Stop capturing keyboard events
#[tauri::command]
pub fn stop_shortcut_recording(capture_state: State<'_, KeyboardCaptureState>) -> Result<(), String> {
    crate::info!("Stopping shortcut recording...");

    let mut capture = capture_state.lock().map_err(|e| e.to_string())?;
    capture.stop()?;

    crate::info!("Shortcut recording stopped");
    Ok(())
}

/// Open System Preferences to the Accessibility pane
#[tauri::command]
pub fn open_accessibility_preferences() -> Result<(), String> {
    crate::info!("Opening Accessibility preferences...");

    crate::keyboard_capture::permissions::open_accessibility_settings().map_err(|e| {
        crate::error!("Failed to open preferences: {}", e);
        e
    })?;

    crate::info!("Opened Accessibility preferences");
    Ok(())
}
