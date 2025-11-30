// Recording coordinator - orchestrates audio capture, state, and WAV encoding

use crate::audio::{
    encode_wav, AudioBuffer, AudioCaptureBackend, AudioCaptureError, AudioCaptureService,
    FileWriter, WavEncodingError, DEFAULT_SAMPLE_RATE,
};
use crate::recording::{RecordingManager, RecordingState, RecordingStateError};

/// Errors that can occur during recording coordination
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinatorError {
    /// Already recording - concurrent start rejected
    AlreadyRecording,
    /// Not currently recording
    NotRecording,
    /// Audio capture failed
    CaptureError(AudioCaptureError),
    /// State transition failed
    StateError(RecordingStateError),
    /// WAV encoding failed
    EncodingError(WavEncodingError),
}

impl std::fmt::Display for CoordinatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinatorError::AlreadyRecording => write!(f, "Already recording"),
            CoordinatorError::NotRecording => write!(f, "Not currently recording"),
            CoordinatorError::CaptureError(e) => write!(f, "Capture error: {:?}", e),
            CoordinatorError::StateError(e) => write!(f, "State error: {}", e),
            CoordinatorError::EncodingError(e) => write!(f, "Encoding error: {:?}", e),
        }
    }
}

impl std::error::Error for CoordinatorError {}

/// Metadata returned after a successful recording
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RecordingMetadata {
    /// Duration of the recording in seconds
    pub duration_secs: f64,
    /// Path to the saved WAV file
    pub file_path: String,
    /// Number of audio samples recorded
    pub sample_count: usize,
}

/// Orchestrates audio capture, state management, and WAV encoding
pub struct RecordingCoordinator<B: AudioCaptureBackend, W: FileWriter> {
    capture_service: AudioCaptureService<B>,
    state_manager: RecordingManager,
    file_writer: W,
    sample_rate: u32,
}

impl<B: AudioCaptureBackend, W: FileWriter> RecordingCoordinator<B, W> {
    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl<B: AudioCaptureBackend, W: FileWriter> RecordingCoordinator<B, W> {
    /// Create a new recording coordinator
    pub fn new(backend: B, file_writer: W) -> Self {
        Self {
            capture_service: AudioCaptureService::new(backend),
            state_manager: RecordingManager::new(),
            file_writer,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }

    /// Create a coordinator with custom sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Get the current recording state
    pub fn state(&self) -> RecordingState {
        self.state_manager.get_state()
    }

    /// Start recording audio
    ///
    /// Returns error if already recording
    pub fn start_recording(&mut self) -> Result<(), CoordinatorError> {
        // Reject if not idle
        if self.state_manager.get_state() != RecordingState::Idle {
            return Err(CoordinatorError::AlreadyRecording);
        }

        // Transition to Recording (creates buffer)
        self.state_manager
            .transition_to(RecordingState::Recording)
            .map_err(CoordinatorError::StateError)?;

        // Get buffer and start capture
        let buffer = self
            .state_manager
            .get_audio_buffer()
            .map_err(CoordinatorError::StateError)?;

        // Start capture - rollback on failure
        if let Err(e) = self.capture_service.start_capture(buffer) {
            self.state_manager.reset_to_idle();
            return Err(CoordinatorError::CaptureError(e));
        }

        Ok(())
    }

    /// Stop recording and save the WAV file
    ///
    /// Returns recording metadata on success
    pub fn stop_recording(&mut self) -> Result<RecordingMetadata, CoordinatorError> {
        // Check if currently recording
        if self.state_manager.get_state() != RecordingState::Recording {
            return Err(CoordinatorError::NotRecording);
        }

        // Stop capture
        self.capture_service
            .stop_capture()
            .map_err(CoordinatorError::CaptureError)?;

        // Transition to Processing
        self.state_manager
            .transition_to(RecordingState::Processing)
            .map_err(CoordinatorError::StateError)?;

        // Get buffer and extract samples
        let buffer = self
            .state_manager
            .get_audio_buffer()
            .map_err(CoordinatorError::StateError)?;

        let samples = buffer.lock().unwrap().clone();
        let sample_count = samples.len();

        // Encode WAV - stay in Processing on failure (preserves buffer for retry)
        let file_path = encode_wav(&samples, self.sample_rate, &self.file_writer)
            .map_err(CoordinatorError::EncodingError)?;

        // Calculate duration
        let duration_secs = sample_count as f64 / self.sample_rate as f64;

        // Transition to Idle (clears buffer)
        self.state_manager
            .transition_to(RecordingState::Idle)
            .map_err(CoordinatorError::StateError)?;

        Ok(RecordingMetadata {
            duration_secs,
            file_path,
            sample_count,
        })
    }
}
