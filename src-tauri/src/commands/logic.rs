// Command implementation logic - testable functions separate from Tauri wrappers

use crate::audio::{
    encode_wav, parse_duration_from_file, AudioThreadHandle, SystemFileWriter, DEFAULT_SAMPLE_RATE,
};
use crate::error;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata, RecordingState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Information about a single recording for frontend consumption
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordingInfo {
    /// Filename of the recording (e.g., "recording-2025-12-01-143025.wav")
    pub filename: String,
    /// Full path to the recording file
    pub file_path: String,
    /// Duration of the recording in seconds
    pub duration_secs: f64,
    /// Creation timestamp in ISO 8601 format
    pub created_at: String,
    /// File size in bytes
    pub file_size_bytes: u64,
}

/// Information about the current recording state for frontend consumption
#[derive(Debug, Clone, Serialize)]
pub struct RecordingStateInfo {
    /// Current state (Idle, Recording, Processing)
    pub state: RecordingState,
}

/// Implementation of start_recording
///
/// # Arguments
/// * `state` - The recording manager state
/// * `audio_thread` - Optional audio thread handle for starting capture
///
/// # Errors
/// Returns an error string if:
/// - Already recording
/// - State transition fails
/// - Audio capture fails to start
/// - State lock is poisoned
pub fn start_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;

    // Check current state
    if manager.get_state() != RecordingState::Idle {
        return Err(
            "A recording is already in progress. Stop the current recording first.".to_string(),
        );
    }

    // Start recording with default sample rate
    let buffer = manager
        .start_recording(DEFAULT_SAMPLE_RATE)
        .map_err(|_| "Failed to initialize recording.")?;

    // Start audio capture if audio thread is available
    if let Some(audio_thread) = audio_thread {
        match audio_thread.start(buffer) {
            Ok(sample_rate) => {
                // Update with actual sample rate from device
                manager.set_sample_rate(sample_rate);
            }
            Err(_) => {
                // Audio capture failed - rollback state and return error
                manager.reset_to_idle();
                return Err(
                    "Could not access the microphone. Please check that your microphone is connected and permissions are granted.".to_string(),
                );
            }
        }
    }

    Ok(())
}

/// Implementation of stop_recording
///
/// # Arguments
/// * `state` - The recording manager state
/// * `audio_thread` - Optional audio thread handle for stopping capture
///
/// # Returns
/// Recording metadata including duration, file path, and sample count
///
/// # Errors
/// Returns an error string if:
/// - Not currently recording
/// - State transition fails
/// - WAV encoding fails
/// - State lock is poisoned
pub fn stop_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
) -> Result<RecordingMetadata, String> {
    let mut manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;

    // Check current state
    if manager.get_state() != RecordingState::Recording {
        return Err("No recording in progress. Start a recording first.".to_string());
    }

    // Stop audio capture if audio thread is available
    let stop_result = if let Some(audio_thread) = audio_thread {
        audio_thread.stop().ok()
    } else {
        None
    };

    // Get the actual sample rate before transitioning
    let sample_rate = manager.get_sample_rate().unwrap_or(DEFAULT_SAMPLE_RATE);

    // Transition to Processing
    manager
        .transition_to(RecordingState::Processing)
        .map_err(|_| "Failed to process recording.")?;

    // Get the audio buffer and encode
    let buffer = manager
        .get_audio_buffer()
        .map_err(|_| "No recorded audio available.")?;
    let samples = buffer
        .lock()
        .map_err(|_| "Unable to access recorded audio.")?
        .clone();
    let sample_count = samples.len();

    // Encode WAV if we have samples
    let file_path = if !samples.is_empty() {
        let writer = SystemFileWriter;
        encode_wav(&samples, sample_rate, &writer)
            .map_err(|_| "Failed to save the recording. Please check disk space and try again.")?
    } else {
        // No samples recorded - return placeholder
        String::new()
    };

    // Calculate duration using actual sample rate
    let duration_secs = sample_count as f64 / sample_rate as f64;

    // Transition to Idle
    manager
        .transition_to(RecordingState::Idle)
        .map_err(|_| "Failed to complete recording.")?;

    // Extract stop reason from result
    let stop_reason = stop_result.and_then(|r| r.reason);

    Ok(RecordingMetadata {
        duration_secs,
        file_path,
        sample_count,
        stop_reason,
    })
}

/// Implementation of get_recording_state
///
/// # Returns
/// Current state information for the frontend
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn get_recording_state_impl(
    state: &Mutex<RecordingManager>,
) -> Result<RecordingStateInfo, String> {
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
    Ok(RecordingStateInfo {
        state: manager.get_state(),
    })
}

/// Implementation of get_last_recording_buffer
///
/// # Returns
/// Audio data from the most recent completed recording
///
/// # Errors
/// Returns an error string if:
/// - No previous recording exists
/// - State lock is poisoned
pub fn get_last_recording_buffer_impl(
    state: &Mutex<RecordingManager>,
) -> Result<AudioData, String> {
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
    manager.get_last_recording_buffer().map_err(|_| {
        "No recording available. Please make a recording first.".to_string()
    })
}

/// Implementation of clear_last_recording_buffer
///
/// Clears the retained recording buffer to free memory
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn clear_last_recording_buffer_impl(state: &Mutex<RecordingManager>) -> Result<(), String> {
    let mut manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
    manager.clear_last_recording();
    Ok(())
}

/// Get the recordings directory path
///
/// Uses the same path as SystemFileWriter for consistency
fn get_recordings_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("heycat")
        .join("recordings")
}

/// Implementation of list_recordings
///
/// Lists all recordings from the app data directory with their metadata.
///
/// # Returns
/// A list of RecordingInfo sorted by creation time (newest first).
/// Returns an empty list if the recordings directory doesn't exist or is empty.
///
/// # Errors
/// Only returns an error if there's a critical system failure.
/// Individual file errors are logged and the file is skipped.
pub fn list_recordings_impl() -> Result<Vec<RecordingInfo>, String> {
    let recordings_dir = get_recordings_dir();

    // Return empty list if directory doesn't exist (not an error)
    if !recordings_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&recordings_dir).map_err(|e| {
        error!("Failed to read recordings directory: {}", e);
        format!("Unable to access recordings directory: {}", e)
    })?;

    let mut recordings: Vec<RecordingInfo> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only process .wav files
        if path.extension().and_then(|s| s.to_str()) != Some("wav") {
            continue;
        }

        // Get file metadata
        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to read metadata for {}: {}", path.display(), e);
                continue;
            }
        };

        // Get filename
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => {
                error!("Invalid filename for {}", path.display());
                continue;
            }
        };

        // Get file size
        let file_size_bytes = metadata.len();

        // Get creation time (or modification time as fallback)
        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| {
                let datetime: DateTime<Utc> = t.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_else(|e| {
                error!("Failed to get creation time for {}: {}", path.display(), e);
                String::new()
            });

        // Parse duration from WAV header
        let duration_secs = match parse_duration_from_file(&path) {
            Ok(d) => d,
            Err(e) => {
                error!(
                    "Failed to parse duration for {}: {:?}",
                    path.display(),
                    e
                );
                continue;
            }
        };

        recordings.push(RecordingInfo {
            filename,
            file_path: path.to_string_lossy().to_string(),
            duration_secs,
            created_at,
            file_size_bytes,
        });
    }

    // Sort by created_at descending (newest first)
    recordings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(recordings)
}
