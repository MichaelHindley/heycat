// Shared types for transcription services
// These types are used by all transcription backends (Parakeet, etc.)

use std::path::Path;

/// Transcription state machine states
/// State flow: Unloaded -> (load model) -> Idle -> Transcribing -> Completed/Error -> Idle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionState {
    /// No model loaded, cannot transcribe
    Unloaded,
    /// Model loaded, ready to transcribe
    Idle,
    /// Currently processing audio
    Transcribing,
    /// Transcription completed successfully
    Completed,
    /// Transcription failed with error
    Error,
}

/// Errors that can occur during transcription operations
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptionError {
    /// Model has not been loaded yet
    ModelNotLoaded,
    /// Failed to load the model
    ModelLoadFailed(String),
    /// Failed during transcription
    TranscriptionFailed(String),
    /// Audio data is invalid or empty
    InvalidAudio(String),
    /// Failed to acquire lock on context
    LockPoisoned,
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionError::ModelNotLoaded => write!(f, "Model not loaded"),
            TranscriptionError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            TranscriptionError::TranscriptionFailed(msg) => {
                write!(f, "Transcription failed: {}", msg)
            }
            TranscriptionError::InvalidAudio(msg) => write!(f, "Invalid audio: {}", msg),
            TranscriptionError::LockPoisoned => write!(f, "Internal lock error"),
        }
    }
}

impl std::error::Error for TranscriptionError {}

/// Result type for transcription operations
pub type TranscriptionResult<T> = Result<T, TranscriptionError>;

/// Trait for transcription services, enabling mockability in tests
#[allow(dead_code)]
pub trait TranscriptionService: Send + Sync {
    /// Load a model from the given path
    #[must_use = "this returns a Result that should be handled"]
    fn load_model(&self, path: &Path) -> TranscriptionResult<()>;

    /// Transcribe audio from a WAV file to text
    #[must_use = "this returns a Result that should be handled"]
    fn transcribe(&self, file_path: &str) -> TranscriptionResult<String>;

    /// Check if a model is loaded
    fn is_loaded(&self) -> bool;

    /// Get the current transcription state
    fn state(&self) -> TranscriptionState;

    /// Reset state from Completed/Error back to Idle
    /// This should be called after handling the transcription result
    #[must_use = "this returns a Result that should be handled"]
    fn reset_to_idle(&self) -> TranscriptionResult<()>;
}

// Tests removed per docs/TESTING.md:
// - Display/Debug trait tests: "if it compiles, it works"
// - Clone tests: Type system guarantee
// - Equality tests: Type system guarantee (#[derive(PartialEq)])
// - State existence tests: Type system guarantees enum variants exist
