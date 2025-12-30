// Turso database event emission helpers
//
// Turso/SQLite is synchronous. Events are emitted directly after
// successful CRUD operations.
//
// Architecture:
// ```text
// CRUD Operation (TursoClient method)
//        ↓
// Event Emission (this module)
//        ↓
// Tauri app_handle.emit()
//        ↓
// Frontend Event Bridge
//        ↓
// Query invalidation / Zustand update
// ```

use tauri::{AppHandle, Emitter};

use crate::events::{dictionary_events, window_context_events};

/// Event names for Turso database events
pub mod event_names {
    /// Emitted when voice_command table changes
    pub const VOICE_COMMANDS_UPDATED: &str = "voice_commands_updated";

    /// Emitted when recordings table changes
    pub const RECORDINGS_UPDATED: &str = "recordings_updated";

    /// Emitted when transcriptions table changes
    pub const TRANSCRIPTIONS_UPDATED: &str = "transcriptions_updated";
}

/// Payload for voice_commands_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCommandsUpdatedPayload {
    /// Type of change: "add", "update", "delete", or "sync"
    pub action: String,
    /// ID of the affected command
    pub command_id: String,
}

/// Payload for recordings_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingsUpdatedPayload {
    /// Type of change: "add", "update", or "delete"
    pub change_type: String,
    /// ID of the affected recording
    pub recording_id: Option<String>,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Payload for transcriptions_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionsUpdatedPayload {
    /// Type of change: "add", "update", or "delete"
    pub change_type: String,
    /// ID of the affected transcription
    pub transcription_id: Option<String>,
    /// ID of the associated recording
    pub recording_id: Option<String>,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Emit dictionary_updated event after a dictionary entry change.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for event emission
/// * `action` - Type of change: "add", "update", or "delete"
/// * `entry_id` - ID of the affected dictionary entry
pub fn emit_dictionary_updated(app_handle: &AppHandle, action: &str, entry_id: &str) {
    let payload = dictionary_events::DictionaryUpdatedPayload {
        action: action.to_string(),
        entry_id: entry_id.to_string(),
    };
    if let Err(e) = app_handle.emit(dictionary_events::DICTIONARY_UPDATED, payload) {
        crate::warn!("Failed to emit dictionary_updated event: {}", e);
    }
}

/// Emit window_contexts_updated event after a window context change.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for event emission
/// * `action` - Type of change: "add", "update", or "delete"
/// * `context_id` - ID of the affected window context
pub fn emit_window_contexts_updated(app_handle: &AppHandle, action: &str, context_id: &str) {
    let payload = window_context_events::WindowContextsUpdatedPayload {
        action: action.to_string(),
        context_id: context_id.to_string(),
    };
    if let Err(e) = app_handle.emit(window_context_events::WINDOW_CONTEXTS_UPDATED, payload) {
        crate::warn!("Failed to emit window_contexts_updated event: {}", e);
    }
}

/// Emit recordings_updated event after a recording change.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for event emission
/// * `change_type` - Type of change: "add", "update", or "delete"
/// * `recording_id` - ID of the affected recording
pub fn emit_recordings_updated(
    app_handle: &AppHandle,
    change_type: &str,
    recording_id: Option<&str>,
) {
    let payload = RecordingsUpdatedPayload {
        change_type: change_type.to_string(),
        recording_id: recording_id.map(|s| s.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(e) = app_handle.emit(event_names::RECORDINGS_UPDATED, payload) {
        crate::warn!("Failed to emit recordings_updated event: {}", e);
    }
}

/// Emit transcriptions_updated event after a transcription change.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for event emission
/// * `change_type` - Type of change: "add", "update", or "delete"
/// * `transcription_id` - ID of the affected transcription
/// * `recording_id` - ID of the associated recording
pub fn emit_transcriptions_updated(
    app_handle: &AppHandle,
    change_type: &str,
    transcription_id: Option<&str>,
    recording_id: Option<&str>,
) {
    let payload = TranscriptionsUpdatedPayload {
        change_type: change_type.to_string(),
        transcription_id: transcription_id.map(|s| s.to_string()),
        recording_id: recording_id.map(|s| s.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(e) = app_handle.emit(event_names::TRANSCRIPTIONS_UPDATED, payload) {
        crate::warn!("Failed to emit transcriptions_updated event: {}", e);
    }
}

/// Emit voice_commands_updated event after a voice command change.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for event emission
/// * `action` - Type of change: "add", "update", or "delete"
/// * `command_id` - ID of the affected command
pub fn emit_voice_commands_updated(app_handle: &AppHandle, action: &str, command_id: &str) {
    let payload = VoiceCommandsUpdatedPayload {
        action: action.to_string(),
        command_id: command_id.to_string(),
    };
    if let Err(e) = app_handle.emit(event_names::VOICE_COMMANDS_UPDATED, payload) {
        crate::warn!("Failed to emit voice_commands_updated event: {}", e);
    }
}

#[cfg(test)]
#[path = "events_test.rs"]
mod tests;
