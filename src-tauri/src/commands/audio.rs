//! Audio device commands for Tauri IPC.
//!
//! Contains commands for listing devices and monitoring audio levels.

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;

use crate::audio::AudioInputDevice;
use crate::events::event_names;

use super::common::get_settings_file;
use super::AudioMonitorState;

/// List all available audio input devices
///
/// Returns a list of audio input devices sorted with the default device first.
/// Returns an empty array (not an error) when no devices are available.
#[tauri::command]
pub fn list_audio_devices() -> Vec<AudioInputDevice> {
    crate::audio::list_input_devices()
}

/// Start audio level monitoring for device testing
///
/// Starts capturing audio from the specified device and emits "audio-level" events
/// with the current input level (0-100). Used for visual feedback in the device selector.
#[tauri::command]
pub fn start_audio_monitor(
    app_handle: AppHandle,
    monitor_state: State<'_, AudioMonitorState>,
    device_name: Option<String>,
) -> Result<(), String> {
    // Start monitoring and get the level receiver
    let level_rx = monitor_state.start(device_name)?;

    // Spawn a thread to forward levels to frontend
    std::thread::spawn(move || {
        while let Ok(level) = level_rx.recv() {
            let _ = app_handle.emit(event_names::AUDIO_LEVEL, level);
        }
    });

    Ok(())
}

/// Stop audio level monitoring
///
/// Stops capturing audio and releasing the device.
#[tauri::command]
pub fn stop_audio_monitor(monitor_state: State<'_, AudioMonitorState>) -> Result<(), String> {
    monitor_state.stop()
}

/// Initialize audio monitor at app startup
///
/// Pre-warms the AVAudioEngine so that audio settings UI is instant when opened.
/// Gracefully returns Ok if no audio devices are available.
#[tauri::command]
pub fn init_audio_monitor(
    app_handle: AppHandle,
    monitor_state: State<'_, AudioMonitorState>,
) -> Result<(), String> {
    // Check if any audio devices are available
    let devices = crate::audio::list_input_devices();
    if devices.is_empty() {
        crate::info!("No audio devices available, skipping audio monitor pre-initialization");
        return Ok(());
    }

    // Read saved device from settings store
    let settings_file = get_settings_file(&app_handle);
    let device_name = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("audio.selectedDevice"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Pre-initialize the audio engine
    monitor_state.init(device_name)
}
