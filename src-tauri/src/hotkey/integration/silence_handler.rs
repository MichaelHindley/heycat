//! Silence detection handler for HotkeyIntegration.
//!
//! Manages starting and stopping silence detection during hotkey recordings.

use crate::events::{
    CommandEventEmitter, RecordingEventEmitter, TranscriptionCompletedPayload,
    TranscriptionEventEmitter,
};
use crate::recording::RecordingManager;
use std::sync::{Arc, Mutex};

use super::clipboard_helper::copy_and_paste;
use super::config::TranscriptionResult;
use super::transcription_runner::execute_transcription_task;
use super::HotkeyIntegration;

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: RecordingEventEmitter,
    T: TranscriptionEventEmitter + 'static,
    C: CommandEventEmitter + 'static,
{
    /// Start silence detection for hotkey recording
    ///
    /// When silence detection is enabled and all required components are configured,
    /// this starts monitoring the recording audio for silence. The recording will
    /// automatically stop when silence is detected after speech ends.
    ///
    /// This method is called after recording starts successfully. The detection runs
    /// in a separate thread and will handle saving/transcription when silence triggers.
    pub(crate) fn start_silence_detection(&self, recording_state: &Mutex<RecordingManager>) {
        // Check if silence detection is enabled (from silence config)
        if !self.silence.enabled {
            crate::debug!("Silence detection disabled for hotkey recordings");
            return;
        }

        // Check for required components
        let detectors = match &self.recording_detectors {
            Some(d) => d.clone(),
            None => {
                crate::debug!("Recording detectors not configured, skipping silence detection");
                return;
            }
        };

        let audio_thread = match &self.audio_thread {
            Some(at) => at.clone(),
            None => {
                crate::debug!("No audio thread configured, cannot start silence detection");
                return;
            }
        };

        // Verify transcription emitter is configured (needed for the callback)
        let transcription_config = match &self.transcription {
            Some(c) => c,
            None => {
                crate::debug!("No transcription config, cannot start silence detection");
                return;
            }
        };
        if transcription_config.emitter.is_none() {
            crate::debug!("No transcription emitter configured, cannot start silence detection");
            return;
        }

        // Get the audio buffer from recording state
        let buffer = {
            let manager = match recording_state.lock() {
                Ok(m) => m,
                Err(_) => {
                    crate::warn!("Failed to lock recording state for silence detection");
                    return;
                }
            };

            match manager.get_audio_buffer() {
                Ok(buf) => buf.clone(),
                Err(_) => {
                    crate::warn!("No audio buffer available for silence detection");
                    return;
                }
            }
        };

        // Create transcription callback that calls spawn_transcription
        // This is the same pattern used in wake word flow
        // Extract components from transcription config
        let shared_model = transcription_config.shared_model.clone();
        let transcription_emitter_for_callback = transcription_config.emitter.clone();
        let app_handle_for_callback = self.app_handle.clone();
        let recording_state_for_callback = self.recording_state.clone();
        let transcription_semaphore_for_callback = transcription_config.semaphore.clone();
        let transcription_timeout_for_callback = transcription_config.timeout;

        // Build transcription callback
        let transcription_callback: Option<Box<dyn Fn(String) + Send + 'static>> =
            if shared_model.is_some() && transcription_emitter_for_callback.is_some() {
                Some(Box::new(move |file_path: String| {
                    // Extract required components from Option wrappers
                    let shared_model = match &shared_model {
                        Some(m) => m.clone(),
                        None => return,
                    };
                    let transcription_emitter = match &transcription_emitter_for_callback {
                        Some(te) => te.clone(),
                        None => return,
                    };

                    if !shared_model.is_loaded() {
                        crate::info!("Transcription skipped: model not loaded");
                        return;
                    }

                    let semaphore = transcription_semaphore_for_callback.clone();
                    let timeout_duration = transcription_timeout_for_callback;
                    let app_handle = app_handle_for_callback.clone();
                    let recording_state = recording_state_for_callback.clone();

                    crate::info!(
                        "[silence_detection] Spawning transcription task for: {}",
                        file_path
                    );

                    tauri::async_runtime::spawn(async move {
                        // Execute transcription using shared helper
                        let result = execute_transcription_task(
                            file_path,
                            shared_model.clone(),
                            semaphore,
                            transcription_emitter.clone(),
                            timeout_duration,
                            recording_state.clone(),
                        )
                        .await;

                        // Handle transcription result
                        let TranscriptionResult { text, duration_ms } = match result {
                            Ok(r) => r,
                            Err(()) => return, // Error already emitted and buffer cleared by helper
                        };

                        // Silence detection auto-stop always goes to clipboard
                        // Voice command matching is only supported for manual hotkey recordings
                        // (via spawn_transcription). This is by design - auto-stop recordings
                        // are intended for quick dictation, not command execution.
                        copy_and_paste(&app_handle, &text);

                        // Emit completed
                        transcription_emitter
                            .emit_transcription_completed(TranscriptionCompletedPayload {
                                text,
                                duration_ms,
                            });

                        // Reset model and clear buffer
                        let _ = shared_model.reset_to_idle();
                        if let Some(ref state) = recording_state {
                            if let Ok(mut manager) = state.lock() {
                                manager.clear_last_recording();
                                crate::debug!("Cleared recording buffer");
                            }
                        }
                    });
                }))
            } else {
                None
            };

        // Lock detectors and start monitoring
        let mut det = match detectors.lock() {
            Ok(d) => d,
            Err(_) => {
                crate::warn!("Failed to lock recording detectors");
                return;
            }
        };

        // Use the recording state Arc that was configured via builder
        let recording_state_arc = match &self.recording_state {
            Some(rs) => rs.clone(),
            None => {
                crate::warn!("No recording state configured, cannot start silence detection");
                return;
            }
        };

        // Create a recording emitter for the detection coordinator
        // We need to use R which implements RecordingEventEmitter
        // But we can't clone self.recording_emitter since it's moved into self
        // Instead, create a new emitter from the app handle
        let recording_emitter_for_detectors = match &self.app_handle {
            Some(handle) => Arc::new(crate::commands::TauriEventEmitter::new(handle.clone())),
            None => {
                crate::warn!(
                    "No app handle configured, cannot create emitter for silence detection"
                );
                return;
            }
        };

        crate::info!("[silence_detection] Starting monitoring for hotkey recording");
        if let Err(e) = det.start_monitoring(
            buffer,
            recording_state_arc,
            audio_thread,
            recording_emitter_for_detectors,
            transcription_callback,
        ) {
            crate::warn!("Failed to start silence detection: {}", e);
        } else {
            crate::info!("[silence_detection] Monitoring started successfully");
        }
    }

    /// Stop silence detection for hotkey recording
    ///
    /// Called when the user manually stops recording via hotkey. This ensures
    /// the silence detection thread is stopped before processing the recording,
    /// allowing manual stop to take precedence over auto-stop.
    pub(crate) fn stop_silence_detection(&self) {
        let detectors = match &self.recording_detectors {
            Some(d) => d,
            None => return,
        };

        if let Ok(mut det) = detectors.lock() {
            if det.is_running() {
                crate::info!("[silence_detection] Stopping monitoring (manual stop)");
                det.stop_monitoring();
            }
        }
    }
}
