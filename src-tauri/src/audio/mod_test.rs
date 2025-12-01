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

