// cpal-based audio capture backend
// This code interacts with hardware and is excluded from coverage measurement
//
// Note: All impl blocks here are excluded from coverage because they
// interact with hardware and cannot be unit tested.
#![cfg_attr(coverage_nightly, coverage(off))]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream};
use rubato::{FftFixedIn, Resampler};

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, MAX_RESAMPLE_BUFFER_SAMPLES, TARGET_SAMPLE_RATE};
use crate::audio_constants::RESAMPLE_CHUNK_SIZE;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

/// Audio capture backend using cpal for platform-specific audio capture
pub struct CpalBackend {
    state: CaptureState,
    stream: Option<Stream>,
    /// Stores callback state reference for diagnostic logging on stop
    callback_state: Option<Arc<CallbackState>>,
}

impl CpalBackend {
    /// Create a new cpal backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            stream: None,
            callback_state: None,
        }
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Find an audio input device by name
///
/// Searches through all input devices and returns the one matching the given name.
/// Returns None if no device with that name is found.
fn find_device_by_name(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    host.input_devices()
        .ok()?
        .find(|d| d.name().map(|n| n == name).unwrap_or(false))
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

/// Shared state for audio processing callback
/// Captures all the Arc-wrapped resources needed by the callback
struct CallbackState {
    buffer: AudioBuffer,
    stop_signal: Option<Sender<StopReason>>,
    signaled: Arc<AtomicBool>,
    resampler: Option<Arc<Mutex<FftFixedIn<f32>>>>,
    resample_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_size: usize,
    /// Tracks total input samples received from device (for diagnostic logging)
    input_sample_count: Arc<AtomicUsize>,
    /// Tracks total output samples after resampling (for diagnostic logging)
    output_sample_count: Arc<AtomicUsize>,
    /// Device sample rate (for ratio calculation in diagnostics)
    device_sample_rate: u32,
}

impl CallbackState {
    /// Process f32 audio samples - handles resampling and buffer management
    ///
    /// This is the core audio processing logic, extracted to avoid duplication
    /// across F32, I16, and U16 sample format callbacks.
    fn process_samples(&self, f32_samples: &[f32]) {
        // Track input samples for diagnostic logging
        self.input_sample_count.fetch_add(f32_samples.len(), Ordering::Relaxed);

        let samples_to_add = if let Some(ref resampler) = self.resampler {
            // Accumulate samples and resample when we have enough
            let mut resample_buf = match self.resample_buffer.lock() {
                Ok(buf) => buf,
                Err(_) => return,
            };

            // Signal stop if resample buffer overflows - data loss is unacceptable
            if resample_buf.len() + f32_samples.len() > MAX_RESAMPLE_BUFFER_SAMPLES {
                crate::error!("Resample buffer overflow: resampling can't keep up with audio input");
                if !self.signaled.swap(true, Ordering::SeqCst) {
                    if let Some(ref sender) = self.stop_signal {
                        let _ = sender.send(StopReason::ResampleOverflow);
                    }
                }
                return;
            }
            resample_buf.extend_from_slice(f32_samples);

            // Process full chunks using pre-allocated buffer
            let mut resampled = Vec::new();
            while resample_buf.len() >= self.chunk_size {
                // Use pre-allocated chunk buffer to avoid allocation
                if let Ok(mut chunk_buf) = self.chunk_buffer.lock() {
                    chunk_buf.copy_from_slice(&resample_buf[..self.chunk_size]);
                    resample_buf.drain(..self.chunk_size);
                    if let Ok(mut r) = resampler.lock() {
                        if let Ok(output) = r.process(&[chunk_buf.as_slice()], None) {
                            if !output.is_empty() {
                                resampled.extend_from_slice(&output[0]);
                            }
                        }
                    }
                } else {
                    // Fallback to allocation if chunk buffer lock fails
                    let chunk: Vec<f32> = resample_buf.drain(..self.chunk_size).collect();
                    if let Ok(mut r) = resampler.lock() {
                        if let Ok(output) = r.process(&[chunk], None) {
                            if !output.is_empty() {
                                resampled.extend_from_slice(&output[0]);
                            }
                        }
                    }
                }
            }
            resampled
        } else {
            // No resampling needed
            f32_samples.to_vec()
        };

        // Use lock-free ring buffer for reduced contention
        // Check if buffer is full before pushing
        if self.buffer.is_full() {
            if !self.signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = self.stop_signal {
                    let _ = sender.send(StopReason::BufferFull);
                }
            }
            return;
        }

        // Track output samples for diagnostic logging
        self.output_sample_count.fetch_add(samples_to_add.len(), Ordering::Relaxed);

        // Push samples to ring buffer (lock-free)
        let pushed = self.buffer.push_samples(&samples_to_add);
        if pushed < samples_to_add.len() {
            // Buffer became full during push
            if !self.signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = self.stop_signal {
                    let _ = sender.send(StopReason::BufferFull);
                }
            }
        }
    }

    /// Flush any remaining samples in the resample buffer
    ///
    /// Called from stop() after the stream is dropped but before CallbackState is dropped.
    /// This ensures residual samples that didn't fill a complete chunk are processed.
    fn flush_residuals(&self) {
        // Only need to flush if we have a resampler
        let Some(ref resampler) = self.resampler else {
            return;
        };

        let mut resample_buf = match self.resample_buffer.lock() {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let residual_count = resample_buf.len();
        if residual_count == 0 || residual_count >= self.chunk_size {
            // No residuals, or already a full chunk (would have been processed)
            return;
        }

        crate::debug!("Flushing {} residual samples from resample buffer", residual_count);

        // Zero-pad to chunk size
        resample_buf.resize(self.chunk_size, 0.0);

        // Process the padded chunk
        if let Ok(mut r) = resampler.lock() {
            if let Ok(output) = r.process(&[resample_buf.as_slice()], None) {
                if !output.is_empty() {
                    // Calculate expected output samples based on residual count and resample ratio
                    let expected_ratio = TARGET_SAMPLE_RATE as f64 / self.device_sample_rate as f64;
                    let expected_output = (residual_count as f64 * expected_ratio).ceil() as usize;
                    let actual_output = output[0].len().min(expected_output);

                    // Track these output samples
                    self.output_sample_count.fetch_add(actual_output, Ordering::Relaxed);

                    // Push only the meaningful samples (not the zero-padded tail)
                    self.buffer.push_samples(&output[0][..actual_output]);

                    crate::debug!(
                        "Flushed {} residual samples -> {} output samples",
                        residual_count, actual_output
                    );
                }
            }
        }

        // Clear the buffer
        resample_buf.clear();
    }

    /// Log sample count diagnostics when recording stops
    fn log_sample_diagnostics(&self) {
        let input = self.input_sample_count.load(Ordering::Relaxed);
        let output = self.output_sample_count.load(Ordering::Relaxed);

        if input == 0 {
            crate::debug!("Sample diagnostics: No samples recorded");
            return;
        }

        let actual_ratio = output as f64 / input as f64;
        let expected_ratio = TARGET_SAMPLE_RATE as f64 / self.device_sample_rate as f64;
        let ratio_error = ((actual_ratio - expected_ratio) / expected_ratio * 100.0).abs();

        crate::info!(
            "Sample diagnostics: input={}, output={}, actual_ratio={:.6}, expected_ratio={:.6}, error={:.2}%",
            input, output, actual_ratio, expected_ratio, ratio_error
        );
    }
}

impl AudioCaptureBackend for CpalBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
    ) -> Result<u32, AudioCaptureError> {
        crate::info!("Starting audio capture (target: {}Hz)...", TARGET_SAMPLE_RATE);

        // Get the default audio host
        let host = cpal::default_host();
        crate::debug!("Host: {:?}", host.id());

        // Find the requested device or fall back to default
        let device = if let Some(ref name) = device_name {
            match find_device_by_name(name) {
                Some(d) => {
                    crate::info!("Using requested device: {}", name);
                    d
                }
                None => {
                    crate::warn!(
                        "Requested device '{}' not found, falling back to default",
                        name
                    );
                    host.default_input_device().ok_or_else(|| {
                        crate::error!("No input device available!");
                        AudioCaptureError::NoDeviceAvailable
                    })?
                }
            }
        } else {
            host.default_input_device().ok_or_else(|| {
                crate::error!("No input device available!");
                AudioCaptureError::NoDeviceAvailable
            })?
        };
        crate::debug!(
            "Input device: {:?}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Try to get a config with 16kHz sample rate, fall back to default
        let (config, needs_resampling) = if let Some(config_16k) = find_config_with_sample_rate(&device, TARGET_SAMPLE_RATE) {
            crate::info!("Device supports {}Hz natively", TARGET_SAMPLE_RATE);
            (config_16k, false)
        } else {
            let default_config = device.default_input_config().map_err(|e| {
                crate::error!("Failed to get input config: {}", e);
                AudioCaptureError::DeviceError(e.to_string())
            })?;
            crate::warn!(
                "Device doesn't support {}Hz, will resample from {}Hz",
                TARGET_SAMPLE_RATE,
                default_config.sample_rate().0
            );
            (default_config, true)
        };

        let device_sample_rate = config.sample_rate().0;
        crate::debug!(
            "Config: {} Hz, {:?}, {} channels",
            device_sample_rate,
            config.sample_format(),
            config.channels()
        );

        // Create resampler if needed
        let resampler: Option<Arc<Mutex<FftFixedIn<f32>>>> = if needs_resampling {
            let r = create_resampler(device_sample_rate, TARGET_SAMPLE_RATE, RESAMPLE_CHUNK_SIZE)?;
            Some(Arc::new(Mutex::new(r)))
        } else {
            None
        };

        // Shared flag to ensure we only signal once
        let signaled = std::sync::Arc::new(AtomicBool::new(false));

        // Create error handler that signals stop on stream errors
        let err_signal = stop_signal.clone();
        let err_signaled = signaled.clone();
        let err_fn = move |err: cpal::StreamError| {
            crate::error!("Audio stream error: {}", err);
            // Signal stop so recording doesn't continue with garbage data
            if !err_signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = err_signal {
                    let _ = sender.send(StopReason::StreamError);
                }
            }
        };

        // Buffer for accumulating samples before resampling
        let resample_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

        // Pre-allocated chunk buffer to avoid allocations in hot path
        let chunk_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0f32; RESAMPLE_CHUNK_SIZE]));

        // Create shared callback state - all callbacks use the same processing logic
        let callback_state = Arc::new(CallbackState {
            buffer,
            stop_signal,
            signaled,
            resampler,
            resample_buffer,
            chunk_buffer,
            chunk_size: RESAMPLE_CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(0)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate,
        });

        // Build the input stream based on sample format
        // Each callback converts to f32 and delegates to CallbackState::process_samples
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let state = callback_state.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // F32 samples are already in the correct format
                        state.process_samples(data);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let state = callback_state.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert i16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        state.process_samples(&f32_samples);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let state = callback_state.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert u16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();
                        state.process_samples(&f32_samples);
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
            crate::error!("Failed to build input stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        // Start the stream
        stream.play().map_err(|e| {
            crate::error!("Failed to start stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        crate::info!("Audio stream started successfully at {}Hz (output: {}Hz)", device_sample_rate, TARGET_SAMPLE_RATE);
        self.stream = Some(stream);
        self.callback_state = Some(callback_state);
        self.state = CaptureState::Capturing;
        // Always return TARGET_SAMPLE_RATE since we resample if needed
        Ok(TARGET_SAMPLE_RATE)
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        crate::debug!("Stopping audio capture...");

        // First, stop the stream so audio callback stops running
        if let Some(stream) = self.stream.take() {
            // Stream will be dropped here, stopping capture
            drop(stream);
            crate::debug!("Audio stream stopped");
        } else {
            crate::debug!("No active stream to stop");
        }

        // Now flush any residual samples and log diagnostics
        // This must happen after stream is stopped but before callback_state is dropped
        if let Some(ref callback_state) = self.callback_state {
            callback_state.flush_residuals();
            callback_state.log_sample_diagnostics();
        }

        // Clear callback state
        self.callback_state = None;
        self.state = CaptureState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod resampler_tests {
    use super::*;
    use rubato::Resampler;

    const SOURCE_RATE: usize = 48000;
    const TARGET_RATE: usize = 16000;
    const CHUNK_SIZE: usize = RESAMPLE_CHUNK_SIZE;

    /// Test that the FFT resampler eventually produces output after enough input.
    /// The FFT resampler has internal latency and may not produce output on every call.
    #[test]
    fn test_resampler_produces_output_after_warmup() {
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        let mut total_output = 0usize;

        // Process multiple chunks to warm up the resampler
        for _ in 0..10 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let output = resampler.process(&[&chunk], None).unwrap();
            total_output += output[0].len();
        }

        // After enough input, we should have some output
        assert!(
            total_output > 0,
            "Resampler should produce output after processing multiple chunks"
        );
    }

    /// Test that sample ratio converges to expected value over many chunks.
    /// The FFT resampler has internal latency that can cause ratio drift on small samples.
    #[test]
    fn test_sample_ratio_converges() {
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();
        let expected_ratio = TARGET_RATE as f64 / SOURCE_RATE as f64;

        let mut total_input = 0usize;
        let mut total_output = 0usize;

        // Process many chunks to get past initial latency effects
        for _ in 0..100 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let output = resampler.process(&[&chunk], None).unwrap();

            total_input += CHUNK_SIZE;
            total_output += output[0].len();
        }

        // With enough samples, ratio should be close to expected
        // Allow 5% tolerance due to FFT windowing effects
        let actual_ratio = total_output as f64 / total_input as f64;
        let ratio_error = ((actual_ratio - expected_ratio) / expected_ratio * 100.0).abs();
        assert!(
            ratio_error < 5.0,
            "Ratio error {:.2}% exceeds 5%",
            ratio_error
        );
    }

    /// Test edge case: flush when buffer is empty (should be a no-op)
    #[test]
    fn test_flush_with_empty_buffer() {
        let resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // Create CallbackState with empty resample buffer
        let callback_state = CallbackState {
            buffer: AudioBuffer::new(),
            stop_signal: None,
            signaled: Arc::new(AtomicBool::new(false)),
            resampler: Some(Arc::new(Mutex::new(resampler))),
            resample_buffer: Arc::new(Mutex::new(Vec::new())),
            chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
            chunk_size: CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(0)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate: SOURCE_RATE as u32,
        };

        // Flush should be a no-op when buffer is empty
        callback_state.flush_residuals();

        // Verify no samples were output
        assert_eq!(
            callback_state.output_sample_count.load(Ordering::Relaxed),
            0
        );
    }

    /// Test that resample buffer is cleared after flush
    #[test]
    fn test_buffer_cleared_after_flush() {
        // First warm up the resampler so it has internal state
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // Process several chunks to warm up
        for _ in 0..5 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let _ = resampler.process(&[&chunk], None).unwrap();
        }

        // Create CallbackState with residual samples in buffer
        let residual_samples: Vec<f32> = vec![0.5f32; 500];
        let callback_state = CallbackState {
            buffer: AudioBuffer::new(),
            stop_signal: None,
            signaled: Arc::new(AtomicBool::new(false)),
            resampler: Some(Arc::new(Mutex::new(resampler))),
            resample_buffer: Arc::new(Mutex::new(residual_samples)),
            chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
            chunk_size: CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(5 * CHUNK_SIZE + 500)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate: SOURCE_RATE as u32,
        };

        // Flush the residual samples
        callback_state.flush_residuals();

        // Verify buffer is now empty
        let resample_buf = callback_state.resample_buffer.lock().unwrap();
        assert!(
            resample_buf.is_empty(),
            "Resample buffer should be empty after flush"
        );

        // The flush should process the residual samples through the resampler.
        // Due to FFT latency, we may or may not get output, so just verify the buffer is cleared.
    }

    /// Test that flush processes residuals without panicking
    #[test]
    fn test_flush_residuals_does_not_panic() {
        let resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // Test with various residual sizes
        for residual_size in [1, 100, 500, 1023] {
            let residual_samples: Vec<f32> = vec![0.5f32; residual_size];
            let callback_state = CallbackState {
                buffer: AudioBuffer::new(),
                stop_signal: None,
                signaled: Arc::new(AtomicBool::new(false)),
                resampler: Some(Arc::new(Mutex::new(
                    FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap(),
                ))),
                resample_buffer: Arc::new(Mutex::new(residual_samples)),
                chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
                chunk_size: CHUNK_SIZE,
                input_sample_count: Arc::new(AtomicUsize::new(residual_size)),
                output_sample_count: Arc::new(AtomicUsize::new(0)),
                device_sample_rate: SOURCE_RATE as u32,
            };

            // Should not panic
            callback_state.flush_residuals();

            // Buffer should be cleared
            let resample_buf = callback_state.resample_buffer.lock().unwrap();
            assert!(
                resample_buf.is_empty(),
                "Buffer should be empty after flush with {} residuals",
                residual_size
            );
        }
    }
}
