// Tests for recording coordinator
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use crate::audio::{
    AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, FileWriter,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// =============================================================================
// MockAudioBackend
// =============================================================================

struct MockAudioBackend {
    state: CaptureState,
    should_fail_start: bool,
    should_fail_stop: bool,
    samples_to_add: Vec<f32>,
    buffer_ref: Option<AudioBuffer>,
}

impl MockAudioBackend {
    fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            should_fail_start: false,
            should_fail_stop: false,
            samples_to_add: Vec::new(),
            buffer_ref: None,
        }
    }

    fn with_start_failure(mut self) -> Self {
        self.should_fail_start = true;
        self
    }

    fn with_stop_failure(mut self) -> Self {
        self.should_fail_stop = true;
        self
    }

    fn with_samples(mut self, samples: Vec<f32>) -> Self {
        self.samples_to_add = samples;
        self
    }
}

impl AudioCaptureBackend for MockAudioBackend {
    fn start(&mut self, buffer: AudioBuffer) -> Result<(), AudioCaptureError> {
        if self.should_fail_start {
            return Err(AudioCaptureError::NoDeviceAvailable);
        }

        // Add samples to buffer (simulates capture)
        if !self.samples_to_add.is_empty() {
            let mut guard = buffer.lock().unwrap();
            guard.extend_from_slice(&self.samples_to_add);
        }

        self.buffer_ref = Some(buffer);
        self.state = CaptureState::Capturing;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        if self.should_fail_stop {
            return Err(AudioCaptureError::StreamError("Stop failed".to_string()));
        }
        self.state = CaptureState::Stopped;
        Ok(())
    }

    fn state(&self) -> CaptureState {
        self.state
    }
}

// =============================================================================
// MockFileWriter
// =============================================================================

struct MockFileWriter {
    output_dir: PathBuf,
    filename: String,
    should_fail: bool,
    created_dirs: Arc<Mutex<Vec<PathBuf>>>,
}

impl MockFileWriter {
    fn new() -> Self {
        Self {
            output_dir: std::env::temp_dir().join("heycat-coordinator-test"),
            filename: "test-recording.wav".to_string(),
            should_fail: false,
            created_dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }
}

impl FileWriter for MockFileWriter {
    fn output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }

    fn generate_filename(&self) -> String {
        self.filename.clone()
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        if self.should_fail {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission denied",
            ));
        }
        self.created_dirs.lock().unwrap().push(path.to_path_buf());
        std::fs::create_dir_all(path)
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

// =============================================================================
// Constructor and State Tests
// =============================================================================

#[test]
fn test_new_coordinator_starts_idle() {
    let coordinator = RecordingCoordinator::new(MockAudioBackend::new(), MockFileWriter::new());
    assert_eq!(coordinator.state(), RecordingState::Idle);
}

#[test]
fn test_coordinator_with_custom_sample_rate() {
    let coordinator = RecordingCoordinator::new(MockAudioBackend::new(), MockFileWriter::new())
        .with_sample_rate(48000);
    assert_eq!(coordinator.sample_rate(), 48000);
}

// =============================================================================
// start_recording Tests
// =============================================================================

#[test]
fn test_start_recording_transitions_to_recording() {
    let mut coordinator =
        RecordingCoordinator::new(MockAudioBackend::new(), MockFileWriter::new());

    let result = coordinator.start_recording();
    assert!(result.is_ok());
    assert_eq!(coordinator.state(), RecordingState::Recording);
}

#[test]
fn test_concurrent_start_rejected() {
    let mut coordinator =
        RecordingCoordinator::new(MockAudioBackend::new(), MockFileWriter::new());

    coordinator.start_recording().unwrap();
    let result = coordinator.start_recording();

    assert!(matches!(result, Err(CoordinatorError::AlreadyRecording)));
    assert_eq!(coordinator.state(), RecordingState::Recording);
}

#[test]
fn test_capture_error_rollback() {
    let backend = MockAudioBackend::new().with_start_failure();
    let mut coordinator = RecordingCoordinator::new(backend, MockFileWriter::new());

    let result = coordinator.start_recording();

    assert!(matches!(result, Err(CoordinatorError::CaptureError(_))));
    assert_eq!(coordinator.state(), RecordingState::Idle);
}

// =============================================================================
// stop_recording Tests
// =============================================================================

#[test]
fn test_stop_when_not_recording_returns_error() {
    let mut coordinator =
        RecordingCoordinator::new(MockAudioBackend::new(), MockFileWriter::new());

    let result = coordinator.stop_recording();

    assert!(matches!(result, Err(CoordinatorError::NotRecording)));
}

#[test]
fn test_full_cycle_produces_metadata() {
    // Clean up temp dir
    let temp_dir = std::env::temp_dir().join("heycat-coordinator-test");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let samples: Vec<f32> = vec![0.5; 44100]; // 1 second of audio
    let backend = MockAudioBackend::new().with_samples(samples);
    let writer = MockFileWriter::new().with_filename("test-full-cycle.wav");

    let mut coordinator = RecordingCoordinator::new(backend, writer);

    coordinator.start_recording().unwrap();
    let metadata = coordinator.stop_recording().unwrap();

    assert_eq!(metadata.sample_count, 44100);
    assert!((metadata.duration_secs - 1.0).abs() < 0.001);
    assert!(metadata.file_path.contains("test-full-cycle.wav"));
    assert_eq!(coordinator.state(), RecordingState::Idle);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_metadata_duration_with_custom_sample_rate() {
    let temp_dir = std::env::temp_dir().join("heycat-coordinator-test-48k");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let samples: Vec<f32> = vec![0.5; 48000]; // 1 second at 48kHz
    let backend = MockAudioBackend::new().with_samples(samples);
    let mut writer = MockFileWriter::new();
    writer.output_dir = temp_dir.clone();
    writer.filename = "test-48k.wav".to_string();

    let mut coordinator =
        RecordingCoordinator::new(backend, writer).with_sample_rate(48000);

    coordinator.start_recording().unwrap();
    let metadata = coordinator.stop_recording().unwrap();

    assert_eq!(metadata.sample_count, 48000);
    assert!((metadata.duration_secs - 1.0).abs() < 0.001);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_encoding_error_preserves_buffer() {
    let samples: Vec<f32> = vec![0.5; 100];
    let backend = MockAudioBackend::new().with_samples(samples);
    let writer = MockFileWriter::new().with_failure();

    let mut coordinator = RecordingCoordinator::new(backend, writer);

    coordinator.start_recording().unwrap();
    let result = coordinator.stop_recording();

    assert!(matches!(result, Err(CoordinatorError::EncodingError(_))));
    // Should stay in Processing state (buffer preserved for retry)
    assert_eq!(coordinator.state(), RecordingState::Processing);
}

#[test]
fn test_buffer_cleared_after_successful_save() {
    let temp_dir = std::env::temp_dir().join("heycat-coordinator-test-clear");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let samples: Vec<f32> = vec![0.5; 100];
    let backend = MockAudioBackend::new().with_samples(samples);
    let mut writer = MockFileWriter::new();
    writer.output_dir = temp_dir.clone();

    let mut coordinator = RecordingCoordinator::new(backend, writer);

    coordinator.start_recording().unwrap();
    coordinator.stop_recording().unwrap();

    // State should be Idle (buffer cleared internally)
    assert_eq!(coordinator.state(), RecordingState::Idle);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// Error Type Tests
// =============================================================================

#[test]
fn test_coordinator_error_display() {
    let err = CoordinatorError::AlreadyRecording;
    assert_eq!(format!("{}", err), "Already recording");

    let err = CoordinatorError::NotRecording;
    assert_eq!(format!("{}", err), "Not currently recording");

    let err = CoordinatorError::CaptureError(AudioCaptureError::NoDeviceAvailable);
    let display = format!("{}", err);
    assert!(display.contains("Capture error"));

    let err = CoordinatorError::StateError(RecordingStateError::NoAudioBuffer);
    let display = format!("{}", err);
    assert!(display.contains("State error"));

    let err = CoordinatorError::EncodingError(crate::audio::WavEncodingError::InvalidInput(
        "test".to_string(),
    ));
    let display = format!("{}", err);
    assert!(display.contains("Encoding error"));
}

#[test]
fn test_coordinator_error_is_std_error() {
    let error: Box<dyn std::error::Error> = Box::new(CoordinatorError::AlreadyRecording);
    assert!(error.to_string().contains("Already recording"));
}

// =============================================================================
// Recording Metadata Tests
// =============================================================================

#[test]
fn test_recording_metadata_clone() {
    let metadata = RecordingMetadata {
        duration_secs: 1.5,
        file_path: "/tmp/test.wav".to_string(),
        sample_count: 66150,
    };
    let cloned = metadata.clone();
    assert_eq!(metadata, cloned);
}

#[test]
fn test_recording_metadata_debug() {
    let metadata = RecordingMetadata {
        duration_secs: 1.0,
        file_path: "/tmp/test.wav".to_string(),
        sample_count: 44100,
    };
    let debug = format!("{:?}", metadata);
    assert!(debug.contains("duration_secs"));
    assert!(debug.contains("file_path"));
    assert!(debug.contains("sample_count"));
}
