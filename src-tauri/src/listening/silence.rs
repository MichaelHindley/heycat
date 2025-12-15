// Silence detection for automatic recording stop
// Uses VAD (Voice Activity Detection) to identify end of speech

use super::vad::{create_vad, VadConfig};
use crate::{debug, info, trace};
use std::time::Instant;
use voice_activity_detector::VoiceActivityDetector;

/// Reason why recording was automatically stopped due to silence detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SilenceStopReason {
    /// Recording stopped because user finished speaking (silence after speech)
    SilenceAfterSpeech,
    /// Recording stopped because no speech was detected after wake word (false activation)
    NoSpeechTimeout,
}

/// Configuration for silence detection
#[derive(Debug, Clone)]
pub struct SilenceConfig {
    /// VAD speech probability threshold (0.0 - 1.0, default: 0.5)
    pub vad_speech_threshold: f32,
    /// Duration of silence before stopping recording in milliseconds (default: 2000)
    pub silence_duration_ms: u32,
    /// Duration before canceling if no speech detected after wake word in milliseconds (default: 5000)
    pub no_speech_timeout_ms: u32,
    /// Duration of pause that doesn't trigger stop in milliseconds (default: 1000)
    #[allow(dead_code)] // Reserved for future pause detection refinement
    pub pause_tolerance_ms: u32,
    /// Sample rate for VAD processing (default: 16000)
    pub sample_rate: u32,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            vad_speech_threshold: 0.5,
            silence_duration_ms: 2000,
            no_speech_timeout_ms: 5000,
            pause_tolerance_ms: 1000,
            sample_rate: 16000,
        }
    }
}

/// Result of processing audio samples
#[derive(Debug, Clone, PartialEq)]
pub enum SilenceDetectionResult {
    /// Continue recording, no action needed
    Continue,
    /// Stop recording due to silence
    Stop(SilenceStopReason),
}

/// Silence detector for automatic recording stop
///
/// Processes audio samples and determines when to stop recording based on:
/// - Silence after speech (user finished talking)
/// - No speech timeout (false activation after wake word)
pub struct SilenceDetector {
    config: SilenceConfig,
    /// Whether we've detected any speech since recording started
    has_detected_speech: bool,
    /// When the current silence period started (if currently silent)
    silence_start: Option<Instant>,
    /// When recording started (for no-speech timeout)
    recording_start: Instant,
    /// Voice activity detector for speech detection
    vad: Option<VoiceActivityDetector>,
}

impl SilenceDetector {
    /// Create a new silence detector with default configuration
    pub fn new() -> Self {
        Self::with_config(SilenceConfig::default())
    }

    /// Create a new silence detector with custom configuration
    ///
    /// Uses the unified VadConfig with silence preset for optimal precision.
    pub fn with_config(config: SilenceConfig) -> Self {
        // Initialize VAD using unified factory
        let vad_config = VadConfig {
            speech_threshold: config.vad_speech_threshold,
            sample_rate: config.sample_rate,
            chunk_size: 512, // Required by Silero VAD at 16kHz
            min_speech_frames: 2,
        };

        let vad = create_vad(&vad_config).ok();

        if vad.is_some() {
            debug!("[silence] VAD initialized (threshold={})", config.vad_speech_threshold);
        } else {
            debug!("[silence] VAD initialization failed, speech detection will be disabled");
        }

        Self {
            config,
            has_detected_speech: false,
            silence_start: None,
            recording_start: Instant::now(),
            vad,
        }
    }

    /// Reset the detector state for a new recording session
    pub fn reset(&mut self) {
        debug!("[silence] Detector reset for new recording session");
        self.has_detected_speech = false;
        self.silence_start = None;
        self.recording_start = Instant::now();

        // Reinitialize VAD for fresh state using unified factory
        let vad_config = VadConfig {
            speech_threshold: self.config.vad_speech_threshold,
            sample_rate: self.config.sample_rate,
            chunk_size: 512,
            min_speech_frames: 2,
        };
        self.vad = create_vad(&vad_config).ok();
    }

    /// Get the configuration
    #[allow(dead_code)] // Utility method for introspection
    pub fn config(&self) -> &SilenceConfig {
        &self.config
    }

    /// Check if speech has been detected
    #[allow(dead_code)] // Utility method for status checks
    pub fn has_detected_speech(&self) -> bool {
        self.has_detected_speech
    }

    /// Check if speech is present using VAD
    ///
    /// Processes audio in 512-sample chunks (required by Silero VAD at 16kHz).
    /// Returns true if any chunk has speech probability above threshold.
    fn check_vad(&mut self, samples: &[f32]) -> bool {
        let vad = match &mut self.vad {
            Some(v) => v,
            None => {
                trace!("[silence] VAD not available, assuming no speech");
                return false;
            }
        };

        // Process in 512-sample chunks (required by Silero VAD at 16kHz)
        let chunk_size = 512;
        let mut max_probability: f32 = 0.0;

        for chunk in samples.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let probability = vad.predict(chunk.to_vec());
                max_probability = max_probability.max(probability);
                if probability >= self.config.vad_speech_threshold {
                    return true; // Speech detected
                }
            }
        }

        trace!("[silence] VAD max_probability={:.3}, threshold={}", max_probability, self.config.vad_speech_threshold);
        false
    }

    /// Process a frame of audio samples and return detection result
    ///
    /// Call this periodically with frames of audio (e.g., 100ms chunks).
    /// Returns whether to continue recording or stop (with reason).
    pub fn process_samples(&mut self, samples: &[f32]) -> SilenceDetectionResult {
        let now = Instant::now();

        // Use VAD to detect speech
        let has_speech = self.check_vad(samples);
        let is_silent = !has_speech;

        if is_silent {
            // Audio is silent (no speech detected by VAD)
            if self.silence_start.is_none() {
                // Start tracking silence period
                debug!("[silence] Silence period started (VAD)");
                self.silence_start = Some(now);
            }

            let silence_duration = self.silence_start.unwrap().elapsed();

            if !self.has_detected_speech {
                // No speech yet - check for no-speech timeout
                let total_elapsed = self.recording_start.elapsed();
                trace!(
                    "[silence] No speech yet, elapsed={:?}, timeout={}ms",
                    total_elapsed,
                    self.config.no_speech_timeout_ms
                );
                if total_elapsed.as_millis() >= self.config.no_speech_timeout_ms as u128 {
                    info!(
                        "[silence] NO_SPEECH_TIMEOUT triggered after {:?}",
                        total_elapsed
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
                }
            } else {
                // Had speech - check for silence after speech (ignoring brief pauses)
                trace!(
                    "[silence] Silence after speech, duration={:?}, threshold={}ms",
                    silence_duration,
                    self.config.silence_duration_ms
                );
                if silence_duration.as_millis() >= self.config.silence_duration_ms as u128 {
                    info!(
                        "[silence] SILENCE_AFTER_SPEECH triggered after {:?} of silence",
                        silence_duration
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech);
                }
            }
        } else {
            // Speech detected by VAD
            if !self.has_detected_speech {
                debug!("[silence] First speech detected via VAD!");
            }
            if self.silence_start.is_some() {
                debug!("[silence] Speech resumed after silence");
            }
            self.has_detected_speech = true;
            self.silence_start = None;
        }

        SilenceDetectionResult::Continue
    }
}

impl Default for SilenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_silence_config_default() {
        let config = SilenceConfig::default();
        assert_eq!(config.vad_speech_threshold, 0.5);
        assert_eq!(config.silence_duration_ms, 2000);
        assert_eq!(config.no_speech_timeout_ms, 5000);
        assert_eq!(config.pause_tolerance_ms, 1000);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_silence_detector_new() {
        let detector = SilenceDetector::new();
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_silence_detector_default() {
        let detector = SilenceDetector::default();
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = SilenceDetector::new();
        // Manually set state to simulate speech detection
        detector.has_detected_speech = true;
        detector.silence_start = Some(Instant::now());

        detector.reset();
        assert!(!detector.has_detected_speech());
        assert!(detector.silence_start.is_none());
    }

    #[test]
    fn test_no_speech_timeout() {
        let config = SilenceConfig {
            no_speech_timeout_ms: 50, // Very short for testing
            ..Default::default()
        };
        let mut detector = SilenceDetector::with_config(config);
        // Silent samples - VAD won't detect speech
        let silent_samples = vec![0.0; 512];

        // Process silence until timeout
        thread::sleep(Duration::from_millis(60));
        let result = detector.process_samples(&silent_samples);

        assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout));
    }

    #[test]
    fn test_silence_after_speech_state_machine() {
        let config = SilenceConfig {
            silence_duration_ms: 50, // Very short for testing
            ..Default::default()
        };
        let mut detector = SilenceDetector::with_config(config);
        let silent_samples = vec![0.0; 512];

        // Manually simulate that speech was detected
        detector.has_detected_speech = true;

        // Start silence tracking
        let _ = detector.process_samples(&silent_samples);

        // Wait and check again
        thread::sleep(Duration::from_millis(60));
        let result = detector.process_samples(&silent_samples);

        assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech));
    }

    #[test]
    fn test_config_accessor() {
        let config = SilenceConfig {
            vad_speech_threshold: 0.7,
            ..Default::default()
        };
        let detector = SilenceDetector::with_config(config);
        assert_eq!(detector.config().vad_speech_threshold, 0.7);
    }

    #[test]
    fn test_silence_stop_reason_debug() {
        let reason = SilenceStopReason::SilenceAfterSpeech;
        let debug = format!("{:?}", reason);
        assert!(debug.contains("SilenceAfterSpeech"));
    }

    #[test]
    fn test_silence_detection_result_eq() {
        let r1 = SilenceDetectionResult::Continue;
        let r2 = SilenceDetectionResult::Continue;
        assert_eq!(r1, r2);

        let r3 = SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
        let r4 = SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
        assert_eq!(r3, r4);

        assert_ne!(r1, r3);
    }

    #[test]
    fn test_silence_samples_no_speech() {
        let mut detector = SilenceDetector::new();
        // Pure silence - VAD should not detect speech
        let silent_samples = vec![0.0; 512];

        let _ = detector.process_samples(&silent_samples);
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_continues_while_waiting() {
        let mut detector = SilenceDetector::new();
        let silent_samples = vec![0.0; 512];

        // Should continue (not timeout yet)
        let result = detector.process_samples(&silent_samples);
        assert_eq!(result, SilenceDetectionResult::Continue);
    }

    #[test]
    fn test_vad_initialized() {
        let detector = SilenceDetector::new();
        // VAD should be initialized
        assert!(detector.vad.is_some());
    }
}
