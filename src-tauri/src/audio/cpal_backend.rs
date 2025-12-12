// cpal-based audio capture backend
// This code interacts with hardware and is excluded from coverage measurement
//
// Note: All impl blocks here are excluded from coverage because they
// interact with hardware and cannot be unit tested.
#![cfg_attr(coverage_nightly, coverage(off))]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream};
use rubato::{FftFixedIn, Resampler};

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, MAX_BUFFER_SAMPLES, TARGET_SAMPLE_RATE};
use crate::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

/// Audio capture backend using cpal for platform-specific audio capture
pub struct CpalBackend {
    state: CaptureState,
    stream: Option<Stream>,
}

impl CpalBackend {
    /// Create a new cpal backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            stream: None,
        }
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Try to find a supported config with the target sample rate
fn find_config_with_sample_rate(
    device: &cpal::Device,
    target_rate: u32,
) -> Option<cpal::SupportedStreamConfig> {
    if let Ok(configs) = device.supported_input_configs() {
        for config_range in configs {
            let min_rate = config_range.min_sample_rate().0;
            let max_rate = config_range.max_sample_rate().0;
            if min_rate <= target_rate && target_rate <= max_rate {
                return Some(config_range.with_sample_rate(SampleRate(target_rate)));
            }
        }
    }
    None
}

/// Create a resampler for converting from source rate to target rate
fn create_resampler(
    source_rate: u32,
    target_rate: u32,
    chunk_size: usize,
) -> Result<FftFixedIn<f32>, AudioCaptureError> {
    FftFixedIn::new(
        source_rate as usize,
        target_rate as usize,
        chunk_size,
        1, // sub_chunks - use 1 for simplicity
        1, // channels - mono
    )
    .map_err(|e| AudioCaptureError::DeviceError(format!("Failed to create resampler: {}", e)))
}

impl AudioCaptureBackend for CpalBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
    ) -> Result<u32, AudioCaptureError> {
        info!("Starting audio capture (target: {}Hz)...", TARGET_SAMPLE_RATE);

        // Get the default audio host
        let host = cpal::default_host();
        debug!("Host: {:?}", host.id());

        // Get the default input device
        let device = host.default_input_device().ok_or_else(|| {
            error!("No input device available!");
            AudioCaptureError::NoDeviceAvailable
        })?;
        debug!(
            "Input device: {:?}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Try to get a config with 16kHz sample rate, fall back to default
        let (config, needs_resampling) = if let Some(config_16k) = find_config_with_sample_rate(&device, TARGET_SAMPLE_RATE) {
            info!("Device supports {}Hz natively", TARGET_SAMPLE_RATE);
            (config_16k, false)
        } else {
            let default_config = device.default_input_config().map_err(|e| {
                error!("Failed to get input config: {}", e);
                AudioCaptureError::DeviceError(e.to_string())
            })?;
            warn!(
                "Device doesn't support {}Hz, will resample from {}Hz",
                TARGET_SAMPLE_RATE,
                default_config.sample_rate().0
            );
            (default_config, true)
        };

        let device_sample_rate = config.sample_rate().0;
        debug!(
            "Config: {} Hz, {:?}, {} channels",
            device_sample_rate,
            config.sample_format(),
            config.channels()
        );

        // Create resampler if needed
        let resampler: Option<Arc<Mutex<FftFixedIn<f32>>>> = if needs_resampling {
            // Use a chunk size suitable for real-time processing
            let chunk_size = 1024;
            let r = create_resampler(device_sample_rate, TARGET_SAMPLE_RATE, chunk_size)?;
            Some(Arc::new(Mutex::new(r)))
        } else {
            None
        };

        // Create an error handler closure
        let err_fn = |err: cpal::StreamError| {
            error!("Audio stream error: {}", err);
        };

        // Shared flag to ensure we only signal once
        let signaled = std::sync::Arc::new(AtomicBool::new(false));

        // Buffer for accumulating samples before resampling
        let resample_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let chunk_size = 1024usize;

        // Build the input stream based on sample format
        // Each callback checks buffer size to prevent unbounded memory growth
        // and signals if buffer is full or lock fails
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal.clone();
                let signaled_clone = signaled.clone();
                let resampler_clone = resampler.clone();
                let resample_buf_clone = resample_buffer.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let samples_to_add = if let Some(ref resampler) = resampler_clone {
                            // Accumulate samples and resample when we have enough
                            let mut resample_buf = match resample_buf_clone.lock() {
                                Ok(buf) => buf,
                                Err(_) => return,
                            };
                            resample_buf.extend_from_slice(data);

                            // Process full chunks
                            let mut resampled = Vec::new();
                            while resample_buf.len() >= chunk_size {
                                let chunk: Vec<f32> = resample_buf.drain(..chunk_size).collect();
                                if let Ok(mut r) = resampler.lock() {
                                    if let Ok(output) = r.process(&[chunk], None) {
                                        if !output.is_empty() {
                                            resampled.extend_from_slice(&output[0]);
                                        }
                                    }
                                }
                            }
                            resampled
                        } else {
                            // No resampling needed
                            data.to_vec()
                        };

                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    let to_add = samples_to_add.len().min(remaining);
                                    guard.extend_from_slice(&samples_to_add[..to_add]);
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    // Buffer full - signal once
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                // Lock poisoned - signal once
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal.clone();
                let signaled_clone = signaled.clone();
                let resampler_clone = resampler.clone();
                let resample_buf_clone = resample_buffer.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert i16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

                        let samples_to_add = if let Some(ref resampler) = resampler_clone {
                            let mut resample_buf = match resample_buf_clone.lock() {
                                Ok(buf) => buf,
                                Err(_) => return,
                            };
                            resample_buf.extend_from_slice(&f32_samples);

                            let mut resampled = Vec::new();
                            while resample_buf.len() >= chunk_size {
                                let chunk: Vec<f32> = resample_buf.drain(..chunk_size).collect();
                                if let Ok(mut r) = resampler.lock() {
                                    if let Ok(output) = r.process(&[chunk], None) {
                                        if !output.is_empty() {
                                            resampled.extend_from_slice(&output[0]);
                                        }
                                    }
                                }
                            }
                            resampled
                        } else {
                            f32_samples
                        };

                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    let to_add = samples_to_add.len().min(remaining);
                                    guard.extend_from_slice(&samples_to_add[..to_add]);
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal;
                let signaled_clone = signaled;
                let resampler_clone = resampler;
                let resample_buf_clone = resample_buffer;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert u16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();

                        let samples_to_add = if let Some(ref resampler) = resampler_clone {
                            let mut resample_buf = match resample_buf_clone.lock() {
                                Ok(buf) => buf,
                                Err(_) => return,
                            };
                            resample_buf.extend_from_slice(&f32_samples);

                            let mut resampled = Vec::new();
                            while resample_buf.len() >= chunk_size {
                                let chunk: Vec<f32> = resample_buf.drain(..chunk_size).collect();
                                if let Ok(mut r) = resampler.lock() {
                                    if let Ok(output) = r.process(&[chunk], None) {
                                        if !output.is_empty() {
                                            resampled.extend_from_slice(&output[0]);
                                        }
                                    }
                                }
                            }
                            resampled
                        } else {
                            f32_samples
                        };

                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    let to_add = samples_to_add.len().min(remaining);
                                    guard.extend_from_slice(&samples_to_add[..to_add]);
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                return Err(AudioCaptureError::DeviceError(
                    "Unsupported sample format".to_string(),
                ))
            }
        }
        .map_err(|e| {
            error!("Failed to build input stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        // Start the stream
        stream.play().map_err(|e| {
            error!("Failed to start stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        info!("Audio stream started successfully at {}Hz (output: {}Hz)", device_sample_rate, TARGET_SAMPLE_RATE);
        self.stream = Some(stream);
        self.state = CaptureState::Capturing;
        // Always return TARGET_SAMPLE_RATE since we resample if needed
        Ok(TARGET_SAMPLE_RATE)
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        debug!("Stopping audio capture...");
        if let Some(stream) = self.stream.take() {
            // Stream will be dropped here, stopping capture
            drop(stream);
            debug!("Audio stream stopped");
        } else {
            debug!("No active stream to stop");
        }
        self.state = CaptureState::Stopped;
        Ok(())
    }
}
