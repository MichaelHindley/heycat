// TranscriptionManager for Parakeet-based transcription
// Provides TDT (batch) transcription using NVIDIA Parakeet models

use super::shared::SharedTranscriptionModel;
use super::types::{TranscriptionResult, TranscriptionService, TranscriptionState};
use std::path::Path;

/// Thread-safe transcription manager for Parakeet models
/// Manages TDT (batch) transcription
///
/// This manager now wraps a SharedTranscriptionModel internally, allowing the same
/// model to be shared with other components like WakeWordDetector. This saves ~3GB
/// of memory by avoiding duplicate model loading.
pub struct TranscriptionManager {
    /// Shared transcription model (wraps ParakeetTDT)
    shared_model: SharedTranscriptionModel,
}

impl Default for TranscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptionManager {
    /// Create a new TranscriptionManager without a loaded model
    ///
    /// This creates its own internal SharedTranscriptionModel. For memory efficiency,
    /// prefer using `with_shared_model()` to share a model across components.
    pub fn new() -> Self {
        Self {
            shared_model: SharedTranscriptionModel::new(),
        }
    }

    /// Create a TranscriptionManager that uses an existing shared model
    ///
    /// This is the preferred constructor for production use, as it allows
    /// sharing a single ~3GB model between TranscriptionManager and WakeWordDetector.
    pub fn with_shared_model(shared_model: SharedTranscriptionModel) -> Self {
        Self { shared_model }
    }

    /// Load the TDT model from the given directory path
    pub fn load_tdt_model(&self, model_dir: &Path) -> TranscriptionResult<()> {
        self.shared_model.load(model_dir)
    }

    /// Check if TDT model is loaded
    pub fn is_tdt_loaded(&self) -> bool {
        self.shared_model.is_loaded()
    }

    /// Get the underlying shared model (for sharing with other components)
    #[allow(dead_code)] // Accessor for future use
    pub fn shared_model(&self) -> SharedTranscriptionModel {
        self.shared_model.clone()
    }
}

impl TranscriptionService for TranscriptionManager {
    #[allow(dead_code)]
    fn load_model(&self, path: &Path) -> TranscriptionResult<()> {
        self.shared_model.load(path)
    }

    fn transcribe(&self, file_path: &str) -> TranscriptionResult<String> {
        self.shared_model.transcribe_file(file_path)
    }

    fn is_loaded(&self) -> bool {
        self.shared_model.is_loaded()
    }

    #[allow(dead_code)]
    fn state(&self) -> TranscriptionState {
        self.shared_model.state()
    }

    fn reset_to_idle(&self) -> TranscriptionResult<()> {
        self.shared_model.reset_to_idle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::TranscriptionError;

    #[test]
    fn test_transcription_manager_new_is_unloaded() {
        let manager = TranscriptionManager::new();
        assert!(!manager.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_transcription_manager_default_is_unloaded() {
        let manager = TranscriptionManager::default();
        assert!(!manager.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_transcribe_returns_error_when_model_not_loaded() {
        let manager = TranscriptionManager::new();
        let result = manager.transcribe("/nonexistent/audio.wav");
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_returns_error_for_empty_path() {
        let manager = TranscriptionManager::new();
        let result = manager.transcribe("");
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
    }

    #[test]
    fn test_load_model_fails_with_invalid_path() {
        let manager = TranscriptionManager::new();
        let result = manager.load_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

    #[test]
    fn test_reset_to_idle_noop_from_unloaded() {
        let manager = TranscriptionManager::new();
        assert_eq!(manager.state(), TranscriptionState::Unloaded);

        manager.reset_to_idle().unwrap();
        // Should remain Unloaded, not reset
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_is_tdt_loaded_false_initially() {
        let manager = TranscriptionManager::new();
        assert!(!manager.is_tdt_loaded());
    }

    #[test]
    fn test_load_tdt_model_fails_with_invalid_path() {
        let manager = TranscriptionManager::new();
        let result = manager.load_tdt_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

    #[test]
    fn test_with_shared_model() {
        let shared = SharedTranscriptionModel::new();
        let manager = TranscriptionManager::with_shared_model(shared.clone());

        // Both should report unloaded
        assert!(!manager.is_loaded());
        assert!(!shared.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
        assert_eq!(shared.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_shared_model_accessor() {
        let manager = TranscriptionManager::new();
        let shared = manager.shared_model();

        // Shared model should reflect the same state
        assert!(!manager.is_loaded());
        assert!(!shared.is_loaded());
    }
}
