//! Audio pipeline diagnostics and quality metrics
//!
//! This module provides quality warnings for the audio processing pipeline.
//!
//! Note: The diagnostics functionality is currently not actively used but
//! the types are part of the public API (StopResult, RecordingResult).

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Threshold for "too quiet" warning (-30dBFS RMS ≈ 0.0316 linear)
const QUIET_THRESHOLD_RMS: f32 = 0.0316;

/// Threshold for clipping detection (samples at or near ±1.0)
const CLIPPING_THRESHOLD: f32 = 0.99;

/// Minimum sample count before issuing warnings (avoid false positives on short bursts)
const MIN_SAMPLES_FOR_WARNING: usize = 8000; // ~0.5 seconds at 16kHz

/// Check if debug audio capture is enabled via environment variable
pub fn debug_audio_enabled() -> bool {
    std::env::var("HEYCAT_DEBUG_AUDIO").is_ok()
}

/// Quality warning types emitted to the frontend
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityWarningType {
    /// Input signal is too quiet for reliable transcription
    TooQuiet,
    /// Input signal is clipping (distortion)
    Clipping,
}

/// Severity level for quality warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    /// Informational - may affect quality but not critical
    Info,
    /// Warning - likely to affect transcription quality
    Warning,
}

/// Quality warning event payload for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityWarning {
    pub warning_type: QualityWarningType,
    pub severity: WarningSeverity,
    pub message: String,
}

/// Audio level metrics
#[derive(Debug, Clone, Default)]
pub struct LevelMetrics {
    /// Peak level (maximum absolute sample value)
    pub peak: f32,
    /// RMS level (root mean square)
    pub rms: f32,
    /// Number of samples analyzed
    pub sample_count: usize,
}

impl LevelMetrics {
    /// Convert RMS to dBFS
    pub fn rms_dbfs(&self) -> f32 {
        if self.rms <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.rms.log10()
        }
    }
}

/// Recording diagnostics collector
///
/// Collects metrics throughout a recording session and can emit warnings
/// when appropriate.
pub struct RecordingDiagnostics {
    /// Total input samples received
    input_sample_count: AtomicUsize,
    /// Running peak level (input)
    input_peak: std::sync::Mutex<f32>,
    /// Running sum of squared samples for RMS (input)
    input_sum_sq: std::sync::Mutex<f64>,
    /// Count of clipping samples detected
    clipping_count: AtomicUsize,
    /// Whether warnings have been emitted (to avoid spam)
    quiet_warning_emitted: AtomicBool,
    clipping_warning_emitted: AtomicBool,
}

impl RecordingDiagnostics {
    /// Create a new diagnostics collector
    pub fn new() -> Self {
        Self {
            input_sample_count: AtomicUsize::new(0),
            input_peak: std::sync::Mutex::new(0.0),
            input_sum_sq: std::sync::Mutex::new(0.0),
            clipping_count: AtomicUsize::new(0),
            quiet_warning_emitted: AtomicBool::new(false),
            clipping_warning_emitted: AtomicBool::new(false),
        }
    }

    /// Record input samples (call before processing)
    pub fn record_input(&self, samples: &[f32]) {
        let count = samples.len();
        self.input_sample_count.fetch_add(count, Ordering::Relaxed);

        // Update peak and sum of squares
        if let (Ok(mut peak), Ok(mut sum_sq)) =
            (self.input_peak.lock(), self.input_sum_sq.lock())
        {
            for &sample in samples {
                let abs_sample = sample.abs();
                if abs_sample > *peak {
                    *peak = abs_sample;
                }
                *sum_sq += (sample * sample) as f64;

                // Detect clipping
                if abs_sample >= CLIPPING_THRESHOLD {
                    self.clipping_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Get input level metrics
    pub fn input_metrics(&self) -> LevelMetrics {
        let sample_count = self.input_sample_count.load(Ordering::Relaxed);
        let peak = self.input_peak.lock().map(|p| *p).unwrap_or(0.0);
        let sum_sq = self.input_sum_sq.lock().map(|s| *s).unwrap_or(0.0);

        let rms = if sample_count > 0 {
            ((sum_sq / sample_count as f64) as f32).sqrt()
        } else {
            0.0
        };

        LevelMetrics {
            peak,
            rms,
            sample_count,
        }
    }

    /// Get clipping count
    pub fn clipping_count(&self) -> usize {
        self.clipping_count.load(Ordering::Relaxed)
    }

    /// Check for quality warnings and return them
    ///
    /// Call this periodically or at the end of recording.
    /// Each warning type is only returned once per recording session.
    pub fn check_warnings(&self) -> Vec<QualityWarning> {
        let mut warnings = Vec::new();

        let input = self.input_metrics();

        // Check for quiet input (only after enough samples)
        if input.sample_count >= MIN_SAMPLES_FOR_WARNING {
            if input.rms < QUIET_THRESHOLD_RMS
                && !self.quiet_warning_emitted.swap(true, Ordering::Relaxed)
            {
                warnings.push(QualityWarning {
                    warning_type: QualityWarningType::TooQuiet,
                    severity: WarningSeverity::Warning,
                    message: format!(
                        "Input signal is very quiet ({:.1}dBFS RMS). Move closer to microphone or speak louder.",
                        input.rms_dbfs()
                    ),
                });
            }
        }

        // Check for clipping
        let clip_count = self.clipping_count();
        if clip_count > 0
            && !self.clipping_warning_emitted.swap(true, Ordering::Relaxed)
        {
            let severity = if clip_count > 100 {
                WarningSeverity::Warning
            } else {
                WarningSeverity::Info
            };

            warnings.push(QualityWarning {
                warning_type: QualityWarningType::Clipping,
                severity,
                message: format!(
                    "Audio clipping detected ({} samples). Reduce microphone gain or move further away.",
                    clip_count
                ),
            });
        }

        warnings
    }
}

impl Default for RecordingDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// Allow sharing across threads
unsafe impl Send for RecordingDiagnostics {}
unsafe impl Sync for RecordingDiagnostics {}

#[cfg(test)]
#[path = "diagnostics_test.rs"]
mod tests;
