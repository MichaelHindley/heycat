// Audio capture module for microphone recording

use std::sync::{Arc, Mutex};

mod cpal_backend;
pub use cpal_backend::CpalBackend;

pub mod thread;
pub use thread::{AudioThreadHandle, StopResult};

pub mod wav;
pub use wav::{encode_wav, parse_duration_from_file, SystemFileWriter};

#[cfg(test)]
mod mod_test;

#[cfg(test)]
mod wav_test;

/// Thread-safe buffer for storing audio samples
/// Uses Arc<Mutex<Vec<f32>>> to allow sharing between capture thread and consumers
#[derive(Debug)]
pub struct AudioBuffer(Arc<Mutex<Vec<f32>>>);

impl AudioBuffer {
    /// Create a new empty audio buffer
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    /// Lock the buffer for access
    pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, Vec<f32>>> {
        self.0.lock()
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AudioBuffer {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// Target sample rate for audio capture (16 kHz for Whisper compatibility)
pub const TARGET_SAMPLE_RATE: u32 = 16000;

/// Maximum buffer size in samples (~10 minutes at 16kHz = 9.6M samples)
/// This prevents unlimited memory growth during long recordings.
/// At 16kHz mono, this is approximately 38MB of f32 data.
pub const MAX_BUFFER_SAMPLES: usize = 16000 * 60 * 10;

/// State of the audio capture process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState {
    /// Not capturing audio
    Idle,
    /// Actively capturing audio
    Capturing,
    /// Capture stopped (audio data available)
    Stopped,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Errors that can occur during audio capture
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCaptureError {
    /// No audio input device is available
    NoDeviceAvailable,
    /// Error with the audio device
    DeviceError(String),
    /// Error with the audio stream
    StreamError(String),
}

/// Reason why audio capture was stopped automatically
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum StopReason {
    /// Buffer reached maximum capacity (~10 minutes)
    BufferFull,
    /// Lock poisoning error in audio callback
    LockError,
}

/// Trait for audio capture backends (allows mocking in tests)
pub trait AudioCaptureBackend {
    /// Start capturing audio into the provided buffer
    /// Returns the actual sample rate of the audio device
    ///
    /// The optional `stop_signal` sender can be used by callbacks to signal
    /// that recording should stop (e.g., buffer full, lock error).
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<std::sync::mpsc::Sender<StopReason>>,
    ) -> Result<u32, AudioCaptureError>;

    /// Stop capturing audio
    fn stop(&mut self) -> Result<(), AudioCaptureError>;
}
