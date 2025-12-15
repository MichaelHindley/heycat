// Unified VAD (Voice Activity Detection) configuration
// Shared between WakeWordDetector (listening) and SilenceDetector (recording)

use voice_activity_detector::VoiceActivityDetector;

/// Error type for VAD operations
#[derive(Debug, Clone, PartialEq)]
pub enum VadError {
    /// VAD initialization failed
    InitializationFailed(String),
}

impl std::fmt::Display for VadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VadError::InitializationFailed(msg) => {
                write!(f, "VAD initialization failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for VadError {}

/// VAD configuration shared across listening and recording components.
///
/// # Threshold Rationale
///
/// Different use cases benefit from different threshold settings:
///
/// - **Wake word detection (0.3)**: Lower threshold for better sensitivity.
///   The wake word detector needs to catch varied pronunciations and volumes,
///   including soft consonants. False positives here just trigger transcription
///   (relatively cheap) rather than user-visible actions.
///
/// - **Silence detection (0.5)**: Higher threshold for precision.
///   The silence detector must avoid cutting off speech prematurely.
///   A higher threshold ensures we only stop recording when true silence
///   is detected, not during brief pauses or soft speech.
///
/// - **Balanced (0.4)**: Good middle ground for general-purpose use.
///
/// The Silero VAD model outputs speech probability 0.0-1.0:
/// - Values below 0.3 are typically background noise
/// - Values 0.3-0.5 may be soft speech or ambiguous audio
/// - Values above 0.5 are confident speech detection
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Speech probability threshold (0.0-1.0)
    ///
    /// Audio frames with probability above this value are considered speech.
    /// See struct docs for threshold rationale.
    pub speech_threshold: f32,

    /// Audio sample rate in Hz
    ///
    /// Must match the audio input source. The Silero VAD model works best
    /// at 16kHz but supports 8kHz as well.
    pub sample_rate: u32,

    /// Chunk size for VAD processing
    ///
    /// Must be 512 for Silero VAD at 16kHz (32ms window).
    /// At 8kHz, use 256 (also 32ms window).
    pub chunk_size: usize,

    /// Minimum speech frames before considering speech detected
    ///
    /// Helps filter out brief noise spikes. Setting to 2 catches
    /// short utterances like "hello" while filtering random pops.
    pub min_speech_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            // Balanced threshold for general use
            // Override with wake_word_config() or silence_config() for specific uses
            speech_threshold: 0.4,
            sample_rate: 16000,
            chunk_size: 512, // Required by Silero VAD at 16kHz
            min_speech_frames: 2,
        }
    }
}

impl VadConfig {
    /// Configuration preset for wake word detection
    ///
    /// Uses a lower threshold (0.3) for better sensitivity to varied
    /// pronunciations and volumes. The cost of false positives is only
    /// an extra transcription attempt.
    pub fn wake_word() -> Self {
        Self {
            speech_threshold: 0.3,
            ..Default::default()
        }
    }

    /// Configuration preset for silence detection
    ///
    /// Uses a higher threshold (0.5) to avoid cutting off speech
    /// during pauses. Precision is more important than sensitivity
    /// when deciding to stop recording.
    pub fn silence() -> Self {
        Self {
            speech_threshold: 0.5,
            ..Default::default()
        }
    }

    /// Create config with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            speech_threshold: threshold,
            ..Default::default()
        }
    }
}

/// Factory function for creating VAD detector
///
/// Initializes a Silero VAD model with the given configuration.
/// The same factory is used by both WakeWordDetector and SilenceDetector
/// to ensure consistent initialization.
///
/// # Errors
///
/// Returns `VadError::InitializationFailed` if the VAD model fails to load.
pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(config.chunk_size)
        .build()
        .map_err(|e| VadError::InitializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VadConfig::default();
        assert_eq!(config.speech_threshold, 0.4);
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.min_speech_frames, 2);
    }

    #[test]
    fn test_wake_word_config() {
        let config = VadConfig::wake_word();
        assert_eq!(config.speech_threshold, 0.3);
        // Other fields should use defaults
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.chunk_size, 512);
    }

    #[test]
    fn test_silence_config() {
        let config = VadConfig::silence();
        assert_eq!(config.speech_threshold, 0.5);
        // Other fields should use defaults
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.chunk_size, 512);
    }

    #[test]
    fn test_with_threshold() {
        let config = VadConfig::with_threshold(0.6);
        assert_eq!(config.speech_threshold, 0.6);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_config_clone() {
        let config = VadConfig::wake_word();
        let cloned = config.clone();
        assert_eq!(config.speech_threshold, cloned.speech_threshold);
    }

    #[test]
    fn test_config_debug() {
        let config = VadConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("speech_threshold"));
        assert!(debug.contains("0.4"));
    }

    #[test]
    fn test_create_vad_success() {
        let config = VadConfig::default();
        let result = create_vad(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_vad_with_presets() {
        // Both presets should create valid VAD instances
        let wake_word_vad = create_vad(&VadConfig::wake_word());
        assert!(wake_word_vad.is_ok());

        let silence_vad = create_vad(&VadConfig::silence());
        assert!(silence_vad.is_ok());
    }

    #[test]
    fn test_vad_error_display() {
        let err = VadError::InitializationFailed("test error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("initialization failed"));
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_vad_error_eq() {
        let err1 = VadError::InitializationFailed("test".to_string());
        let err2 = VadError::InitializationFailed("test".to_string());
        assert_eq!(err1, err2);
    }
}
