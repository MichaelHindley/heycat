//! Transcription execution logic for HotkeyIntegration.
//!
//! Contains the core transcription task execution and voice command matching.

use crate::events::{
    current_timestamp, CommandAmbiguousPayload, CommandCandidate, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::parakeet::{SharedTranscriptionModel, TranscriptionService};
use crate::recording::RecordingManager;
use crate::voice_commands::matcher::MatchResult;
use crate::voice_commands::registry::CommandDefinition;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use super::clipboard_helper::copy_and_paste;
use super::config::TranscriptionResult;
use super::HotkeyIntegration;

/// Execute transcription with semaphore-limited concurrency, timeout, and error handling.
///
/// This is the core transcription logic shared between:
/// - `spawn_transcription` (hotkey recordings with voice command matching)
/// - `start_silence_detection` transcription callback (silence-triggered auto-stop)
///
/// Returns `Ok(TranscriptionResult)` on success, `Err(())` on failure (errors already emitted).
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn execute_transcription_task<T: TranscriptionEventEmitter>(
    file_path: String,
    shared_model: Arc<SharedTranscriptionModel>,
    semaphore: Arc<Semaphore>,
    transcription_emitter: Arc<T>,
    timeout_duration: Duration,
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
) -> Result<TranscriptionResult, ()> {
    // Helper to clear recording buffer - call this in all exit paths to prevent memory leaks
    let clear_recording_buffer = || {
        if let Some(ref state) = recording_state {
            if let Ok(mut manager) = state.lock() {
                manager.clear_last_recording();
                crate::debug!("Cleared recording buffer");
            }
        }
    };

    // Acquire semaphore permit to limit concurrent transcriptions
    let _permit = match semaphore.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            crate::warn!("Too many concurrent transcriptions, skipping this one");
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: "Too many transcriptions in progress. Please wait and try again."
                    .to_string(),
            });
            clear_recording_buffer();
            return Err(());
        }
    };

    // Emit transcription_started event
    let start_time = Instant::now();
    transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
        timestamp: current_timestamp(),
    });

    crate::debug!("Transcribing file: {}", file_path);

    // Perform transcription on blocking thread pool (CPU-intensive) with timeout
    let transcriber = shared_model.clone();
    let transcription_future =
        tokio::task::spawn_blocking(move || transcriber.transcribe(&file_path));

    let transcription_result = tokio::time::timeout(timeout_duration, transcription_future).await;

    let text = match transcription_result {
        Ok(Ok(Ok(text))) => text,
        Ok(Ok(Err(e))) => {
            crate::error!("Transcription failed: {}", e);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: e.to_string(),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
        Ok(Err(e)) => {
            crate::error!("Transcription task panicked: {}", e);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: "Internal transcription error.".to_string(),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
        Err(_) => {
            crate::error!("Transcription timed out after {:?}", timeout_duration);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: format!(
                    "Transcription timed out after {} seconds. The audio may be too long or the model may be stuck.",
                    timeout_duration.as_secs()
                ),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;
    crate::info!(
        "Transcription completed in {}ms: {} chars",
        duration_ms,
        text.len()
    );

    Ok(TranscriptionResult { text, duration_ms })
}

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: crate::events::RecordingEventEmitter,
    T: TranscriptionEventEmitter + 'static,
    C: CommandEventEmitter + 'static,
{
    /// Spawn transcription as an async task
    ///
    /// Transcribes the WAV file, tries command matching, then fallback to clipboard.
    /// Uses Tauri's async runtime for bounded async execution.
    /// No-op if transcription manager or transcription emitter is not configured.
    ///
    /// This method is public so it can be called from the wake word recording flow
    /// (via the coordinator) in addition to the hotkey recording flow.
    ///
    /// When a transcription_callback is configured (via with_transcription_callback),
    /// this method delegates to that callback instead of performing transcription inline.
    /// This enables integration with RecordingTranscriptionService.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn spawn_transcription(&self, file_path: String) {
        // If a transcription callback is configured, delegate to it
        // This enables HotkeyIntegration to use TranscriptionService without duplication
        if let Some(ref config) = self.transcription {
            if let Some(ref callback) = config.callback {
                crate::info!(
                    "Delegating transcription to external service for: {}",
                    file_path
                );
                callback(file_path);
                return;
            }
        }

        // Fallback: inline transcription (for backward compatibility and tests)
        // Check all required components are present from transcription config
        let transcription_config = match &self.transcription {
            Some(c) => c,
            None => {
                crate::debug!("Transcription skipped: no transcription config");
                return;
            }
        };

        let shared_model = match &transcription_config.shared_model {
            Some(m) => m.clone(),
            None => {
                crate::debug!("Transcription skipped: no shared transcription model configured");
                return;
            }
        };

        let transcription_emitter = match &transcription_config.emitter {
            Some(te) => te.clone(),
            None => {
                crate::debug!("Transcription skipped: no transcription emitter configured");
                return;
            }
        };

        // Check if model is loaded
        if !shared_model.is_loaded() {
            crate::info!("Transcription skipped: transcription model not loaded");
            return;
        }

        // Optional voice command components from voice_commands config
        let (turso_client, command_matcher, action_dispatcher, command_emitter) =
            if let Some(ref vc) = self.voice_commands {
                (
                    Some(vc.turso_client.clone()),
                    Some(vc.matcher.clone()),
                    Some(vc.dispatcher.clone()),
                    vc.emitter.clone(),
                )
            } else {
                (None, None, None, None)
            };

        // Clone app_handle for clipboard access
        let app_handle = self.app_handle.clone();

        // Clone recording_state for buffer cleanup after transcription
        let recording_state = self.recording_state.clone();

        // Clone semaphore and timeout from transcription config
        let semaphore = transcription_config.semaphore.clone();
        let timeout_duration = transcription_config.timeout;

        crate::info!("Spawning transcription task...");

        // Spawn async task using Tauri's async runtime
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

            crate::info!("=== spawn_transcription received text ===");
            crate::info!("text content: {:?}", text);
            crate::info!("=== end spawn_transcription text ===");

            // Try voice command matching if configured
            enum MatchOutcome {
                Matched {
                    cmd: CommandDefinition,
                    trigger: String,
                    confidence: f64,
                },
                Ambiguous {
                    candidates: Vec<CommandCandidate>,
                },
                NoMatch,
            }

            // Helper to clear recording buffer
            let clear_recording_buffer = || {
                if let Some(ref state) = recording_state {
                    if let Ok(mut manager) = state.lock() {
                        manager.clear_last_recording();
                        crate::debug!("Cleared recording buffer");
                    }
                }
            };

            let command_handled = if let (
                Some(client),
                Some(matcher),
                Some(dispatcher),
                Some(emitter),
            ) =
                (&turso_client, &command_matcher, &action_dispatcher, &command_emitter)
            {
                // Fetch all commands from Turso
                let all_commands = match client.list_voice_commands().await {
                    Ok(commands) => commands,
                    Err(e) => {
                        crate::error!("Failed to fetch voice commands from Turso: {}", e);
                        transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                            error: "Failed to load voice commands. Please try again.".to_string(),
                        });
                        clear_recording_buffer();
                        return;
                    }
                };

                // Build a lookup map for finding commands by ID
                let commands_by_id: std::collections::HashMap<uuid::Uuid, &CommandDefinition> =
                    all_commands.iter().map(|cmd| (cmd.id, cmd)).collect();

                let match_result = matcher.match_commands(&text, &all_commands);

                let outcome = match match_result {
                    MatchResult::Exact {
                        command: matched_cmd,
                        ..
                    } => match commands_by_id.get(&matched_cmd.id) {
                        Some(cmd) => MatchOutcome::Matched {
                            cmd: (*cmd).clone(),
                            trigger: matched_cmd.trigger.clone(),
                            confidence: 1.0,
                        },
                        None => MatchOutcome::NoMatch,
                    },
                    MatchResult::Fuzzy {
                        command: matched_cmd,
                        score,
                        ..
                    } => match commands_by_id.get(&matched_cmd.id) {
                        Some(cmd) => MatchOutcome::Matched {
                            cmd: (*cmd).clone(),
                            trigger: matched_cmd.trigger.clone(),
                            confidence: score,
                        },
                        None => MatchOutcome::NoMatch,
                    },
                    MatchResult::Ambiguous { candidates } => {
                        let candidate_data: Vec<_> = candidates
                            .iter()
                            .map(|c| CommandCandidate {
                                id: c.command.id.to_string(),
                                trigger: c.command.trigger.clone(),
                                confidence: c.score,
                            })
                            .collect();
                        MatchOutcome::Ambiguous {
                            candidates: candidate_data,
                        }
                    }
                    MatchResult::NoMatch => MatchOutcome::NoMatch,
                };

                match outcome {
                    MatchOutcome::Matched {
                        cmd,
                        trigger,
                        confidence,
                    } => {
                        crate::info!(
                            "Command matched: {} (confidence: {:.2})",
                            trigger,
                            confidence
                        );

                        // Emit command_matched event
                        emitter.emit_command_matched(CommandMatchedPayload {
                            transcription: text.clone(),
                            command_id: cmd.id.to_string(),
                            trigger: trigger.clone(),
                            confidence,
                        });

                        // Execute command directly using await (no new runtime needed!)
                        match dispatcher.execute(&cmd).await {
                            Ok(action_result) => {
                                crate::info!("Command executed: {}", action_result.message);
                                emitter.emit_command_executed(CommandExecutedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    message: action_result.message,
                                });
                            }
                            Err(action_error) => {
                                crate::error!("Command execution failed: {}", action_error);
                                emitter.emit_command_failed(CommandFailedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    error_code: action_error.code.to_string(),
                                    error_message: action_error.message,
                                });
                            }
                        }
                        true // Command was handled
                    }
                    MatchOutcome::Ambiguous { candidates } => {
                        crate::info!("Ambiguous match: {} candidates", candidates.len());

                        // Emit command_ambiguous event for disambiguation UI
                        emitter.emit_command_ambiguous(CommandAmbiguousPayload {
                            transcription: text.clone(),
                            candidates,
                        });
                        true // Command matching was handled (ambiguous)
                    }
                    MatchOutcome::NoMatch => {
                        crate::debug!("No command match for: {}", text);
                        false // Fall through to clipboard
                    }
                }
            } else {
                crate::debug!("Voice commands not configured, skipping command matching");
                false
            };

            // Fallback to clipboard if no command was handled
            if !command_handled {
                copy_and_paste(&app_handle, &text);
            }

            // Always emit transcription_completed (whether command handled or not)
            // This ensures the frontend clears the "Transcribing..." state
            crate::info!("=== Emitting transcription_completed ===");
            crate::info!("text to emit: {:?}", text);
            crate::info!("=== end emit ===");
            transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                text,
                duration_ms,
            });

            // Reset transcription state to idle
            if let Err(e) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", e);
            }

            // Clear recording buffer to free memory
            clear_recording_buffer();
        });
    }
}
