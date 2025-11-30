// Recording state management for Tauri application

use crate::audio::{AudioBuffer, DEFAULT_SAMPLE_RATE};
use serde::Serialize;

/// Recording state enum representing the current state of the recording process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RecordingState {
    /// Not recording, ready to start
    Idle,
    /// Actively recording audio
    Recording,
    /// Recording stopped, processing audio (encoding, saving)
    Processing,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Errors that can occur during state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingStateError {
    /// Invalid state transition attempted
    InvalidTransition {
        from: RecordingState,
        to: RecordingState,
    },
    /// Audio buffer not available
    NoAudioBuffer,
}

impl std::fmt::Display for RecordingStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingStateError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {:?} to {:?}", from, to)
            }
            RecordingStateError::NoAudioBuffer => {
                write!(f, "Audio buffer not available")
            }
        }
    }
}

impl std::error::Error for RecordingStateError {}

/// Audio data returned for transcription pipeline integration
#[derive(Debug, Clone, Serialize)]
pub struct AudioData {
    /// Raw audio samples as f32 values normalized to [-1.0, 1.0]
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g., 44100)
    pub sample_rate: u32,
    /// Duration of the audio in seconds
    pub duration_secs: f64,
}

/// Retained recording data from the last completed recording
#[derive(Debug, Clone)]
struct LastRecording {
    samples: Vec<f32>,
    sample_rate: u32,
}

/// Manager for recording state with thread-safe access
/// Designed to be wrapped in Mutex and managed by Tauri state
pub struct RecordingManager {
    state: RecordingState,
    audio_buffer: Option<AudioBuffer>,
    /// Retained audio data from the last recording for transcription
    last_recording: Option<LastRecording>,
}

impl RecordingManager {
    /// Create a new RecordingManager in Idle state
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
            audio_buffer: None,
            last_recording: None,
        }
    }

    /// Get the current recording state
    pub fn get_state(&self) -> RecordingState {
        self.state
    }

    /// Transition to a new state with validation
    ///
    /// Valid transitions:
    /// - Idle -> Recording (starts recording, creates new buffer)
    /// - Recording -> Processing (stops recording, keeps buffer)
    /// - Processing -> Idle (clears buffer)
    ///
    /// Returns error for invalid transitions
    pub fn transition_to(&mut self, new_state: RecordingState) -> Result<(), RecordingStateError> {
        let valid = matches!(
            (self.state, new_state),
            (RecordingState::Idle, RecordingState::Recording)
                | (RecordingState::Recording, RecordingState::Processing)
                | (RecordingState::Processing, RecordingState::Idle)
        );

        if !valid {
            return Err(RecordingStateError::InvalidTransition {
                from: self.state,
                to: new_state,
            });
        }

        // Handle buffer lifecycle during transitions
        match (self.state, new_state) {
            (RecordingState::Idle, RecordingState::Recording) => {
                self.audio_buffer = Some(AudioBuffer::new());
            }
            (RecordingState::Processing, RecordingState::Idle) => {
                // Retain the buffer for transcription before clearing
                if let Some(ref buffer) = self.audio_buffer {
                    let samples = buffer.lock().unwrap();
                    self.last_recording = Some(LastRecording {
                        samples: samples.clone(),
                        sample_rate: DEFAULT_SAMPLE_RATE,
                    });
                }
                self.audio_buffer = None;
            }
            _ => {}
        }

        self.state = new_state;
        Ok(())
    }

    /// Get a reference to the audio buffer (available during Recording and Processing states)
    pub fn get_audio_buffer(&self) -> Result<AudioBuffer, RecordingStateError> {
        self.audio_buffer
            .clone()
            .ok_or(RecordingStateError::NoAudioBuffer)
    }

    /// Get the last recording's audio data for transcription
    ///
    /// Returns the audio data from the most recent completed recording.
    /// The buffer is retained in memory after the recording is saved.
    pub fn get_last_recording_buffer(&self) -> Result<AudioData, RecordingStateError> {
        match &self.last_recording {
            Some(recording) => {
                let sample_count = recording.samples.len();
                let duration_secs = sample_count as f64 / recording.sample_rate as f64;
                Ok(AudioData {
                    samples: recording.samples.clone(),
                    sample_rate: recording.sample_rate,
                    duration_secs,
                })
            }
            None => Err(RecordingStateError::NoAudioBuffer),
        }
    }

    /// Clear the retained last recording buffer
    ///
    /// Call this to free memory when the transcription is complete
    pub fn clear_last_recording(&mut self) {
        self.last_recording = None;
    }

    /// Force reset to Idle state, clearing any audio buffer
    ///
    /// Use for error recovery when normal state transitions aren't possible
    /// (e.g., capture failure during start_recording)
    pub fn reset_to_idle(&mut self) {
        self.state = RecordingState::Idle;
        self.audio_buffer = None;
    }
}

impl Default for RecordingManager {
    fn default() -> Self {
        Self::new()
    }
}
