// Recording and Transcription CRUD operations using Turso/libsql
//
// Provides database operations for recordings and transcriptions using SQL queries.

use libsql::params;

use super::client::TursoClient;
use crate::audio::StopReason;

/// Recording metadata stored in Turso
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used in tests
pub struct RecordingRecord {
    pub id: String,
    pub file_path: String,
    pub duration_secs: f64,
    pub sample_count: u64,
    pub stop_reason: Option<StopReason>,
    pub created_at: String,
    pub active_window_app_name: Option<String>,
    pub active_window_bundle_id: Option<String>,
    pub active_window_title: Option<String>,
}

/// Error type for recording operations
#[derive(Debug, Clone)]
pub enum RecordingStoreError {
    NotFound(String),
    PersistenceError(String),
    LoadError(String),
}

impl std::fmt::Display for RecordingStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingStoreError::NotFound(id) => write!(f, "Recording not found: {}", id),
            RecordingStoreError::PersistenceError(msg) => {
                write!(f, "Recording persistence error: {}", msg)
            }
            RecordingStoreError::LoadError(msg) => write!(f, "Recording load error: {}", msg),
        }
    }
}

impl std::error::Error for RecordingStoreError {}

/// Transcription record stored in Turso
#[derive(Debug, Clone)]
pub struct TranscriptionRecord {
    pub id: String,
    pub recording_id: String,
    pub text: String,
    pub language: Option<String>,
    pub model_version: String,
    pub duration_ms: u64,
    pub created_at: String,
}

/// Error type for transcription operations
#[derive(Debug, Clone)]
pub enum TranscriptionStoreError {
    PersistenceError(String),
    LoadError(String),
}

impl std::fmt::Display for TranscriptionStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionStoreError::PersistenceError(msg) => {
                write!(f, "Transcription persistence error: {}", msg)
            }
            TranscriptionStoreError::LoadError(msg) => {
                write!(f, "Transcription load error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TranscriptionStoreError {}

impl TursoClient {
    // ============================================================
    // Recording Operations
    // ============================================================

    /// Add a new recording.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the recording
    /// * `file_path` - Path to the audio file
    /// * `duration_secs` - Duration in seconds
    /// * `sample_count` - Number of audio samples
    /// * `stop_reason` - Why recording stopped
    /// * `active_window_app_name` - App name when recording started
    /// * `active_window_bundle_id` - Bundle ID when recording started
    /// * `active_window_title` - Window title when recording started
    pub async fn add_recording(
        &self,
        id: String,
        file_path: String,
        duration_secs: f64,
        sample_count: u64,
        stop_reason: Option<StopReason>,
        active_window_app_name: Option<String>,
        active_window_bundle_id: Option<String>,
        active_window_title: Option<String>,
    ) -> Result<RecordingRecord, RecordingStoreError> {
        let created_at = chrono::Utc::now().to_rfc3339();
        let stop_reason_str = stop_reason.as_ref().map(|r| format!("{:?}", r));

        self.execute(
            r#"INSERT INTO recording
               (id, file_path, duration_secs, sample_count, stop_reason, created_at,
                active_window_app_name, active_window_bundle_id, active_window_title)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                id.clone(),
                file_path.clone(),
                duration_secs,
                sample_count as i64,
                stop_reason_str,
                created_at.clone(),
                active_window_app_name.clone(),
                active_window_bundle_id.clone(),
                active_window_title.clone()
            ],
        )
        .await
        .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(RecordingRecord {
            id,
            file_path,
            duration_secs,
            sample_count,
            stop_reason,
            created_at,
            active_window_app_name,
            active_window_bundle_id,
            active_window_title,
        })
    }

    /// List all recordings ordered by created_at DESC.
    pub async fn list_recordings(&self) -> Result<Vec<RecordingRecord>, RecordingStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, file_path, duration_secs, sample_count, stop_reason, created_at,
                          active_window_app_name, active_window_bundle_id, active_window_title
                   FROM recording
                   ORDER BY created_at DESC"#,
                (),
            )
            .await
            .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;

        let mut recordings = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?
        {
            let recording = parse_recording_row(&row)?;
            recordings.push(recording);
        }

        Ok(recordings)
    }

    /// Get a recording by file path.
    pub async fn get_recording_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<RecordingRecord>, RecordingStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, file_path, duration_secs, sample_count, stop_reason, created_at,
                          active_window_app_name, active_window_bundle_id, active_window_title
                   FROM recording
                   WHERE file_path = ?1"#,
                params![file_path.to_string()],
            )
            .await
            .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;

        match rows
            .next()
            .await
            .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?
        {
            Some(row) => {
                let recording = parse_recording_row(&row)?;
                Ok(Some(recording))
            }
            None => Ok(None),
        }
    }

    /// Delete a recording by file path.
    /// Cascading delete will remove related transcriptions.
    pub async fn delete_recording_by_path(
        &self,
        file_path: &str,
    ) -> Result<(), RecordingStoreError> {
        // Check if recording exists
        let exists = self.recording_exists_by_path(file_path).await?;
        if !exists {
            return Err(RecordingStoreError::NotFound(file_path.to_string()));
        }

        // Delete recording (CASCADE will handle transcriptions)
        self.execute(
            "DELETE FROM recording WHERE file_path = ?1",
            params![file_path.to_string()],
        )
        .await
        .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Check if a recording exists by file path.
    async fn recording_exists_by_path(&self, file_path: &str) -> Result<bool, RecordingStoreError> {
        let mut rows = self
            .query(
                "SELECT 1 FROM recording WHERE file_path = ?1",
                params![file_path.to_string()],
            )
            .await
            .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(rows
            .next()
            .await
            .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?
            .is_some())
    }

    // ============================================================
    // Transcription Operations
    // ============================================================

    /// Add a new transcription.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the transcription
    /// * `recording_id` - ID of the associated recording
    /// * `text` - Transcribed text
    /// * `language` - Detected language
    /// * `model_version` - Version of the transcription model used
    /// * `duration_ms` - Time taken for transcription in milliseconds
    pub async fn add_transcription(
        &self,
        id: String,
        recording_id: String,
        text: String,
        language: Option<String>,
        model_version: String,
        duration_ms: u64,
    ) -> Result<TranscriptionRecord, TranscriptionStoreError> {
        let created_at = chrono::Utc::now().to_rfc3339();

        self.execute(
            r#"INSERT INTO transcription
               (id, recording_id, text, language, model_version, duration_ms, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                id.clone(),
                recording_id.clone(),
                text.clone(),
                language.clone(),
                model_version.clone(),
                duration_ms as i64,
                created_at.clone()
            ],
        )
        .await
        .map_err(|e| TranscriptionStoreError::PersistenceError(e.to_string()))?;

        Ok(TranscriptionRecord {
            id,
            recording_id,
            text,
            language,
            model_version,
            duration_ms,
            created_at,
        })
    }

    /// List all transcriptions.
    pub async fn list_transcriptions(
        &self,
    ) -> Result<Vec<TranscriptionRecord>, TranscriptionStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, recording_id, text, language, model_version, duration_ms, created_at
                   FROM transcription
                   ORDER BY created_at DESC"#,
                (),
            )
            .await
            .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;

        let mut transcriptions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?
        {
            let transcription = parse_transcription_row(&row)?;
            transcriptions.push(transcription);
        }

        Ok(transcriptions)
    }

    /// Get transcriptions by recording ID.
    /// Uses the idx_transcription_recording_id index for efficient lookup.
    pub async fn get_transcriptions_by_recording(
        &self,
        recording_id: &str,
    ) -> Result<Vec<TranscriptionRecord>, TranscriptionStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, recording_id, text, language, model_version, duration_ms, created_at
                   FROM transcription
                   WHERE recording_id = ?1
                   ORDER BY created_at DESC"#,
                params![recording_id.to_string()],
            )
            .await
            .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;

        let mut transcriptions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?
        {
            let transcription = parse_transcription_row(&row)?;
            transcriptions.push(transcription);
        }

        Ok(transcriptions)
    }
}

/// Parse a database row into a RecordingRecord
fn parse_recording_row(row: &libsql::Row) -> Result<RecordingRecord, RecordingStoreError> {
    let id: String = row
        .get(0)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let file_path: String = row
        .get(1)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let duration_secs: f64 = row
        .get(2)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let sample_count: i64 = row
        .get(3)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let stop_reason_str: Option<String> = row
        .get(4)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let created_at: String = row
        .get(5)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let active_window_app_name: Option<String> = row
        .get(6)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let active_window_bundle_id: Option<String> = row
        .get(7)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;
    let active_window_title: Option<String> = row
        .get(8)
        .map_err(|e| RecordingStoreError::LoadError(e.to_string()))?;

    let stop_reason = stop_reason_str.and_then(|s| parse_stop_reason(&s));

    Ok(RecordingRecord {
        id,
        file_path,
        duration_secs,
        sample_count: sample_count as u64,
        stop_reason,
        created_at,
        active_window_app_name,
        active_window_bundle_id,
        active_window_title,
    })
}

/// Parse a database row into a TranscriptionRecord
fn parse_transcription_row(row: &libsql::Row) -> Result<TranscriptionRecord, TranscriptionStoreError> {
    let id: String = row
        .get(0)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let recording_id: String = row
        .get(1)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let text: String = row
        .get(2)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let language: Option<String> = row
        .get(3)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let model_version: String = row
        .get(4)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let duration_ms: i64 = row
        .get(5)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;
    let created_at: String = row
        .get(6)
        .map_err(|e| TranscriptionStoreError::LoadError(e.to_string()))?;

    Ok(TranscriptionRecord {
        id,
        recording_id,
        text,
        language,
        model_version,
        duration_ms: duration_ms as u64,
        created_at,
    })
}

/// Parse StopReason from string
fn parse_stop_reason(s: &str) -> Option<StopReason> {
    match s {
        "BufferFull" => Some(StopReason::BufferFull),
        "LockError" => Some(StopReason::LockError),
        "StreamError" => Some(StopReason::StreamError),
        "ResampleOverflow" => Some(StopReason::ResampleOverflow),
        "SilenceAfterSpeech" => Some(StopReason::SilenceAfterSpeech),
        "NoSpeechTimeout" => Some(StopReason::NoSpeechTimeout),
        _ => None,
    }
}

#[cfg(test)]
#[path = "recording_test.rs"]
mod tests;
