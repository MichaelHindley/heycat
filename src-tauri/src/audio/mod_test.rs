// Tests for audio capture module
// Test code is excluded from coverage since we measure production code coverage
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

// Test: AudioBuffer type alias should be Arc<Mutex<Vec<f32>>>
#[test]
fn test_audio_buffer_new() {
    // Create an AudioBuffer and verify it can hold f32 samples
    let buffer: AudioBuffer = AudioBuffer::new();

    // Push some samples
    {
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
        guard.push(-0.5);
    }

    // Verify samples are stored correctly
    let guard = buffer.lock().unwrap();
    assert_eq!(guard.len(), 2);
    assert_eq!(guard[0], 0.5);
    assert_eq!(guard[1], -0.5);
}

#[test]
fn test_audio_buffer_default() {
    // Default should create empty buffer
    let buffer: AudioBuffer = AudioBuffer::default();
    let guard = buffer.lock().unwrap();
    assert!(guard.is_empty());
}

#[test]
fn test_audio_buffer_clone() {
    // Clone should share the same underlying data (Arc semantics)
    let buffer1 = AudioBuffer::new();
    let buffer2 = buffer1.clone();

    // Modify through buffer1
    {
        let mut guard = buffer1.lock().unwrap();
        guard.push(1.0);
    }

    // Verify buffer2 sees the change (shared Arc)
    let guard = buffer2.lock().unwrap();
    assert_eq!(guard.len(), 1);
    assert_eq!(guard[0], 1.0);
}

// AudioConfig tests
#[test]
fn test_audio_config_default() {
    let config = AudioConfig::default();
    assert_eq!(config.sample_rate, 44100);
}

#[test]
fn test_audio_config_custom_sample_rate() {
    let config = AudioConfig {
        sample_rate: 48000,
    };
    assert_eq!(config.sample_rate, 48000);
}

// CaptureState tests
#[test]
fn test_capture_state_default_is_idle() {
    let state = CaptureState::default();
    assert_eq!(state, CaptureState::Idle);
}

#[test]
fn test_capture_state_variants() {
    // Verify all state variants exist and are distinguishable
    let idle = CaptureState::Idle;
    let capturing = CaptureState::Capturing;
    let stopped = CaptureState::Stopped;

    assert_ne!(idle, capturing);
    assert_ne!(capturing, stopped);
    assert_ne!(stopped, idle);
}

// AudioCaptureError tests
#[test]
fn test_error_no_device_available() {
    let err = AudioCaptureError::NoDeviceAvailable;
    assert!(matches!(err, AudioCaptureError::NoDeviceAvailable));
}

#[test]
fn test_error_device_error() {
    let err = AudioCaptureError::DeviceError("Device disconnected".to_string());
    if let AudioCaptureError::DeviceError(msg) = err {
        assert_eq!(msg, "Device disconnected");
    } else {
        panic!("Expected DeviceError");
    }
}

#[test]
fn test_error_stream_error() {
    let err = AudioCaptureError::StreamError("Buffer overflow".to_string());
    if let AudioCaptureError::StreamError(msg) = err {
        assert_eq!(msg, "Buffer overflow");
    } else {
        panic!("Expected StreamError");
    }
}

// MockBackend for testing
struct MockBackend {
    state: CaptureState,
    should_fail: bool,
    error_type: Option<AudioCaptureError>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            should_fail: false,
            error_type: None,
        }
    }

    fn with_failure(error: AudioCaptureError) -> Self {
        Self {
            state: CaptureState::Idle,
            should_fail: true,
            error_type: Some(error),
        }
    }
}

impl AudioCaptureBackend for MockBackend {
    fn start(&mut self, _buffer: AudioBuffer) -> Result<(), AudioCaptureError> {
        if self.should_fail {
            return Err(self.error_type.clone().unwrap_or(AudioCaptureError::NoDeviceAvailable));
        }
        self.state = CaptureState::Capturing;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        if self.should_fail {
            return Err(self.error_type.clone().unwrap_or(AudioCaptureError::StreamError("Stop failed".into())));
        }
        self.state = CaptureState::Stopped;
        Ok(())
    }

    fn state(&self) -> CaptureState {
        self.state
    }
}

// AudioCaptureService tests
#[test]
fn test_service_new_with_default_config() {
    let service = AudioCaptureService::new(MockBackend::new());
    assert_eq!(service.config().sample_rate, 44100);
    assert_eq!(service.state(), CaptureState::Idle);
}

#[test]
fn test_service_new_with_custom_config() {
    let config = AudioConfig { sample_rate: 48000 };
    let service = AudioCaptureService::with_config(MockBackend::new(), config);
    assert_eq!(service.config().sample_rate, 48000);
}

#[test]
fn test_service_start_recording() {
    let mut service = AudioCaptureService::new(MockBackend::new());
    let buffer = AudioBuffer::new();

    let result = service.start_capture(buffer);
    assert!(result.is_ok());
    assert_eq!(service.state(), CaptureState::Capturing);
}

#[test]
fn test_service_stop_recording() {
    let mut service = AudioCaptureService::new(MockBackend::new());
    let buffer = AudioBuffer::new();

    service.start_capture(buffer).unwrap();
    let result = service.stop_capture();
    assert!(result.is_ok());
    assert_eq!(service.state(), CaptureState::Stopped);
}

#[test]
fn test_service_start_failure() {
    let backend = MockBackend::with_failure(AudioCaptureError::NoDeviceAvailable);
    let mut service = AudioCaptureService::new(backend);
    let buffer = AudioBuffer::new();

    let result = service.start_capture(buffer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AudioCaptureError::NoDeviceAvailable);
}
