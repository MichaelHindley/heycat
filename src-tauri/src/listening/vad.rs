// Unified VAD (Voice Activity Detection) configuration
// Shared between WakeWordDetector (listening) and SilenceDetector (recording)

use crate::audio_constants::{
    chunk_size_for_sample_rate, DEFAULT_SAMPLE_RATE, VAD_THRESHOLD_BALANCED, VAD_THRESHOLD_SILENCE,
    VAD_THRESHOLD_WAKE_WORD,
};
use voice_activity_detector::VoiceActivityDetector;

/// Error type for VAD operations
#[derive(Debug, Clone, PartialEq)]
pub enum VadError {
    /// VAD initialization failed
    InitializationFailed(String),
    /// Invalid configuration (e.g., unsupported sample rate)
    ConfigurationInvalid(String),
}

impl std::fmt::Display for VadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VadError::InitializationFailed(msg) => {
                write!(f, "VAD initialization failed: {}", msg)
            }
            VadError::ConfigurationInvalid(msg) => {
                write!(f, "VAD configuration invalid: {}", msg)
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
///
/// Note: `speech_threshold` and `min_speech_frames` are not used by `create_vad()`
/// (Silero VAD doesn't accept thresholds at init time). They exist for documentation
/// and future use if consumers want to extract threshold config from a unified source.
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Speech probability threshold (0.0-1.0)
    ///
    /// Audio frames with probability above this value are considered speech.
    /// See struct docs for threshold rationale.
    #[allow(dead_code)]
    pub speech_threshold: f32,

    /// Audio sample rate in Hz
    ///
    /// Must match the audio input source. The Silero VAD model only supports
    /// 8000 or 16000 Hz. The chunk size is automatically derived from this
    /// value (32ms window = 256 samples at 8kHz, 512 at 16kHz).
    pub sample_rate: u32,

    /// Minimum speech frames before considering speech detected
    ///
    /// Helps filter out brief noise spikes. Setting to 2 catches
    /// short utterances like "hello" while filtering random pops.
    #[allow(dead_code)]
    pub min_speech_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            // Balanced threshold for general use
            // Override with wake_word_config() or silence_config() for specific uses
            speech_threshold: VAD_THRESHOLD_BALANCED,
            sample_rate: DEFAULT_SAMPLE_RATE,
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
    #[allow(dead_code)]
    pub fn wake_word() -> Self {
        Self {
            speech_threshold: VAD_THRESHOLD_WAKE_WORD,
            ..Default::default()
        }
    }

    /// Configuration preset for silence detection
    ///
    /// Uses a higher threshold (0.5) to avoid cutting off speech
    /// during pauses. Precision is more important than sensitivity
    /// when deciding to stop recording.
    #[allow(dead_code)]
    pub fn silence() -> Self {
        Self {
            speech_threshold: VAD_THRESHOLD_SILENCE,
            ..Default::default()
        }
    }

    /// Create config with custom threshold
    #[allow(dead_code)]
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
/// Returns `VadError::ConfigurationInvalid` if the sample rate is not 8000 or 16000 Hz.
/// Returns `VadError::InitializationFailed` if the VAD model fails to load.
pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    // Validate sample rate - Silero VAD only supports 8kHz and 16kHz
    match config.sample_rate {
        8000 | 16000 => {}
        other => {
            return Err(VadError::ConfigurationInvalid(format!(
                "Unsupported sample rate: {} Hz. Must be 8000 or 16000 Hz.",
                other
            )))
        }
    }

    // Calculate chunk size from sample rate (32ms window)
    let chunk_size = chunk_size_for_sample_rate(config.sample_rate);

    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(chunk_size)
        .build()
        .map_err(|e| VadError::InitializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_constants::{VAD_CHUNK_SIZE_16KHZ, VAD_CHUNK_SIZE_8KHZ};

    #[test]
    fn test_default_config() {
        let config = VadConfig::default();
        assert_eq!(config.speech_threshold, VAD_THRESHOLD_BALANCED);
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
        assert_eq!(config.min_speech_frames, 2);
    }

    #[test]
    fn test_wake_word_config() {
        let config = VadConfig::wake_word();
        assert_eq!(config.speech_threshold, VAD_THRESHOLD_WAKE_WORD);
        // Other fields should use defaults
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn test_silence_config() {
        let config = VadConfig::silence();
        assert_eq!(config.speech_threshold, VAD_THRESHOLD_SILENCE);
        // Other fields should use defaults
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn test_with_threshold() {
        let config = VadConfig::with_threshold(0.6);
        assert_eq!(config.speech_threshold, 0.6);
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
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

    // Sample rate validation tests

    #[test]
    fn test_create_vad_with_8khz_succeeds() {
        let config = VadConfig {
            sample_rate: 8000,
            ..Default::default()
        };
        let result = create_vad(&config);
        assert!(result.is_ok());
        // Verify chunk size is 256 for 8kHz (32ms window)
        assert_eq!(chunk_size_for_sample_rate(8000), VAD_CHUNK_SIZE_8KHZ);
    }

    #[test]
    fn test_create_vad_with_16khz_succeeds() {
        let config = VadConfig {
            sample_rate: 16000,
            ..Default::default()
        };
        let result = create_vad(&config);
        assert!(result.is_ok());
        // Verify chunk size is 512 for 16kHz (32ms window)
        assert_eq!(chunk_size_for_sample_rate(16000), VAD_CHUNK_SIZE_16KHZ);
    }

    #[test]
    fn test_create_vad_with_44100hz_returns_error() {
        let config = VadConfig {
            sample_rate: 44100,
            ..Default::default()
        };
        let result = create_vad(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, VadError::ConfigurationInvalid(_)));
    }

    #[test]
    fn test_create_vad_with_0hz_returns_error() {
        let config = VadConfig {
            sample_rate: 0,
            ..Default::default()
        };
        let result = create_vad(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, VadError::ConfigurationInvalid(_)));
    }

    #[test]
    fn test_sample_rate_error_message_mentions_supported_rates() {
        let config = VadConfig {
            sample_rate: 22050,
            ..Default::default()
        };
        let result = create_vad(&config);
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("8000"), "Error should mention 8000 Hz");
        assert!(msg.contains("16000"), "Error should mention 16000 Hz");
    }

    #[test]
    fn test_configuration_invalid_error_display() {
        let err = VadError::ConfigurationInvalid("test config error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("configuration invalid"));
        assert!(msg.contains("test config error"));
    }

    #[test]
    fn test_configuration_invalid_error_eq() {
        let err1 = VadError::ConfigurationInvalid("test".to_string());
        let err2 = VadError::ConfigurationInvalid("test".to_string());
        assert_eq!(err1, err2);
    }
}
