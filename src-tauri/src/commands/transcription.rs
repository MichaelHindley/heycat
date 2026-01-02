//! Transcription commands for Tauri IPC.
//!
//! Contains commands for transcribing audio files and managing transcription records.

use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri::Emitter;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::emit_or_warn;
use crate::events::{
    event_names, TranscriptionCompletedPayload, TranscriptionErrorPayload,
    TranscriptionStartedPayload,
};
use crate::parakeet::SharedTranscriptionModel;
use crate::turso::events as turso_events;

use super::logic::transcribe_file_impl;
use super::TursoClientState;

/// Transcription record for frontend consumption
#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionInfo {
    pub id: String,
    pub recording_id: String,
    pub text: String,
    pub language: Option<String>,
    pub model_version: String,
    pub duration_ms: u64,
    pub created_at: String,
}

/// Transcribe an audio file and copy result to clipboard
///
/// Also stores the transcription in Turso.
#[tauri::command]
pub async fn transcribe_file(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    shared_model: State<'_, Arc<SharedTranscriptionModel>>,
    file_path: String,
) -> Result<String, String> {
    // Emit transcription started event
    let start_time = std::time::Instant::now();
    emit_or_warn!(
        app_handle,
        event_names::TRANSCRIPTION_STARTED,
        TranscriptionStartedPayload {
            timestamp: crate::events::current_timestamp(),
        }
    );

    // Clone what we need for the blocking task
    let model = shared_model.inner().clone();
    let path = file_path.clone();

    // Run transcription on blocking thread pool
    let result = tokio::task::spawn_blocking(move || transcribe_file_impl(&model, &path))
        .await
        .map_err(|e| format!("Transcription task failed: {}", e))?;

    match result {
        Ok(text) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Copy to clipboard
            if let Err(e) = app_handle.clipboard().write_text(&text) {
                crate::warn!("Failed to copy transcription to clipboard: {}", e);
            }

            // Store transcription in Turso
            if let Ok(Some(recording)) = turso_client.get_recording_by_path(&file_path).await {
                let recording_id = recording.id.clone();
                let transcription_id = uuid::Uuid::new_v4().to_string();
                if let Err(e) = turso_client
                    .add_transcription(
                        transcription_id.clone(),
                        recording_id.clone(),
                        text.clone(),
                        None,
                        "parakeet-tdt".to_string(),
                        duration_ms,
                    )
                    .await
                {
                    crate::warn!("Failed to store transcription in Turso: {}", e);
                } else {
                    crate::debug!("Transcription stored in Turso");
                    turso_events::emit_transcriptions_updated(
                        &app_handle,
                        "add",
                        Some(&transcription_id),
                        Some(&recording_id),
                    );
                }
            } else {
                crate::debug!("No Turso recording found for path: {}", file_path);
            }

            // Emit transcription completed event
            emit_or_warn!(
                app_handle,
                event_names::TRANSCRIPTION_COMPLETED,
                TranscriptionCompletedPayload {
                    text: text.clone(),
                    duration_ms,
                }
            );

            Ok(text)
        }
        Err(e) => {
            emit_or_warn!(
                app_handle,
                event_names::TRANSCRIPTION_ERROR,
                TranscriptionErrorPayload { error: e.clone() }
            );

            Err(e)
        }
    }
}

/// List all transcriptions from Turso
#[tauri::command]
pub async fn list_transcriptions(
    turso_client: State<'_, TursoClientState>,
) -> Result<Vec<TranscriptionInfo>, String> {
    turso_client
        .list_transcriptions()
        .await
        .map(|transcriptions| {
            transcriptions
                .into_iter()
                .map(|t| TranscriptionInfo {
                    id: t.id,
                    recording_id: t.recording_id,
                    text: t.text,
                    language: t.language,
                    model_version: t.model_version,
                    duration_ms: t.duration_ms,
                    created_at: t.created_at,
                })
                .collect()
        })
        .map_err(|e| format!("Failed to list transcriptions: {}", e))
}

/// Get transcriptions for a specific recording
#[tauri::command]
pub async fn get_transcriptions_by_recording(
    turso_client: State<'_, TursoClientState>,
    recording_id: String,
) -> Result<Vec<TranscriptionInfo>, String> {
    turso_client
        .get_transcriptions_by_recording(&recording_id)
        .await
        .map(|transcriptions| {
            transcriptions
                .into_iter()
                .map(|t| TranscriptionInfo {
                    id: t.id,
                    recording_id: t.recording_id,
                    text: t.text,
                    language: t.language,
                    model_version: t.model_version,
                    duration_ms: t.duration_ms,
                    created_at: t.created_at,
                })
                .collect()
        })
        .map_err(|e| format!("Failed to get transcriptions: {}", e))
}
