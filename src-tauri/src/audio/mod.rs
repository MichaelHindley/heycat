// Audio capture module for microphone recording

use std::sync::{Arc, Mutex};

mod cpal_backend;
pub use cpal_backend::CpalBackend;

pub mod thread;
pub use thread::{AudioThreadError, AudioThreadHandle};

pub mod wav;
pub use wav::{encode_wav, FileWriter, SystemFileWriter, WavEncodingError};

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

/// Default sample rate for audio capture (44.1 kHz)
pub const DEFAULT_SAMPLE_RATE: u32 = 44100;

/// Configuration for audio capture
pub struct AudioConfig {
    /// Sample rate in Hz (default: 44100)
    pub sample_rate: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

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

/// Trait for audio capture backends (allows mocking in tests)
pub trait AudioCaptureBackend {
    /// Start capturing audio into the provided buffer
    fn start(&mut self, buffer: AudioBuffer) -> Result<(), AudioCaptureError>;

    /// Stop capturing audio
    fn stop(&mut self) -> Result<(), AudioCaptureError>;

    /// Get the current capture state
    fn state(&self) -> CaptureState;
}

/// Service for managing audio capture
pub struct AudioCaptureService<B: AudioCaptureBackend> {
    backend: B,
    config: AudioConfig,
}

impl<B: AudioCaptureBackend> AudioCaptureService<B> {
    /// Create a new audio capture service with default configuration
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            config: AudioConfig::default(),
        }
    }

    /// Create a new audio capture service with custom configuration
    pub fn with_config(backend: B, config: AudioConfig) -> Self {
        Self { backend, config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Get the current capture state
    pub fn state(&self) -> CaptureState {
        self.backend.state()
    }

    /// Start capturing audio into the provided buffer
    pub fn start_capture(&mut self, buffer: AudioBuffer) -> Result<(), AudioCaptureError> {
        self.backend.start(buffer)
    }

    /// Stop capturing audio
    pub fn stop_capture(&mut self) -> Result<(), AudioCaptureError> {
        self.backend.stop()
    }
}
