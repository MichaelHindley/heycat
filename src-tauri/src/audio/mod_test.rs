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

// STREAMING_CHUNK_SIZE tests
#[test]
fn test_streaming_chunk_size_value() {
    // STREAMING_CHUNK_SIZE should be 2560 (160ms at 16kHz)
    assert_eq!(STREAMING_CHUNK_SIZE, 2560);
}

#[test]
fn test_streaming_chunk_size_calculation() {
    // Verify the calculation: 160ms * 16000Hz = 2560 samples
    let duration_ms = 160;
    let sample_rate = 16000;
    let expected_samples = (duration_ms * sample_rate) / 1000;
    assert_eq!(expected_samples, STREAMING_CHUNK_SIZE);
}

// Streaming accumulation tests
#[test]
fn test_streaming_accumulator_partial_chunk_not_sent() {
    // Simulate streaming accumulator behavior: partial chunks should not be sent
    // This tests the logic pattern used in CallbackState::process_samples
    let chunk_size = STREAMING_CHUNK_SIZE;
    let mut accumulator: Vec<f32> = Vec::new();
    let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);

    // Add samples less than chunk size
    let partial_samples: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
    accumulator.extend_from_slice(&partial_samples);

    // Simulate the streaming logic from process_samples
    while accumulator.len() >= chunk_size {
        let chunk: Vec<f32> = accumulator.drain(..chunk_size).collect();
        let _ = sender.try_send(chunk);
    }

    // Should not have sent anything (< 2560 samples)
    assert!(receiver.try_recv().is_err());
    // Accumulator should still have the partial samples
    assert_eq!(accumulator.len(), 1000);
}

#[test]
fn test_streaming_accumulator_sends_when_chunk_size_reached() {
    // Samples accumulate until chunk size reached before sending
    let chunk_size = STREAMING_CHUNK_SIZE;
    let mut accumulator: Vec<f32> = Vec::new();
    let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);

    // Add exactly chunk_size samples
    let samples: Vec<f32> = (0..chunk_size).map(|i| i as f32 / chunk_size as f32).collect();
    accumulator.extend_from_slice(&samples);

    // Simulate the streaming logic from process_samples
    while accumulator.len() >= chunk_size {
        let chunk: Vec<f32> = accumulator.drain(..chunk_size).collect();
        let _ = sender.try_send(chunk);
    }

    // Should have sent exactly one chunk
    let received_chunk = receiver.try_recv().expect("Should have received one chunk");
    assert_eq!(received_chunk.len(), STREAMING_CHUNK_SIZE);

    // Accumulator should be empty now
    assert!(accumulator.is_empty());

    // No more chunks to receive
    assert!(receiver.try_recv().is_err());
}

#[test]
fn test_streaming_accumulator_multiple_chunks() {
    // Test that multiple chunks are sent when enough samples accumulate
    let chunk_size = STREAMING_CHUNK_SIZE;
    let mut accumulator: Vec<f32> = Vec::new();
    let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);

    // Add 2.5 chunks worth of samples
    let total_samples = chunk_size * 2 + chunk_size / 2; // 2.5 * 2560 = 6400 samples
    let samples: Vec<f32> = (0..total_samples).map(|i| i as f32).collect();
    accumulator.extend_from_slice(&samples);

    // Simulate the streaming logic from process_samples
    while accumulator.len() >= chunk_size {
        let chunk: Vec<f32> = accumulator.drain(..chunk_size).collect();
        let _ = sender.try_send(chunk);
    }

    // Should have sent 2 chunks
    let chunk1 = receiver.try_recv().expect("Should have received first chunk");
    let chunk2 = receiver.try_recv().expect("Should have received second chunk");
    assert_eq!(chunk1.len(), STREAMING_CHUNK_SIZE);
    assert_eq!(chunk2.len(), STREAMING_CHUNK_SIZE);

    // No third chunk
    assert!(receiver.try_recv().is_err());

    // Accumulator should have the remainder (1280 samples)
    assert_eq!(accumulator.len(), chunk_size / 2);
}

