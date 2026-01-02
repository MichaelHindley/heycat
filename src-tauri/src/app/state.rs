//! Managed state type aliases for Tauri application.
//!
//! Centralizes all state type aliases used across the application
//! for app.manage() and State<'_, T> access.

use std::sync::{Arc, Mutex};

use crate::audio::{AudioMonitorHandle, AudioThreadHandle};
use crate::commands::TauriEventEmitter;
use crate::hotkey::HotkeyIntegration;
use crate::keyboard_capture::KeyboardCapture;
use crate::recording::RecordingManager;
use crate::transcription::RecordingTranscriptionService;
use crate::turso::TursoClient;

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

/// Type alias for audio thread state
pub type AudioThreadState = Arc<AudioThreadHandle>;

/// Type alias for production recording state (RecordingManager is Send+Sync)
pub type ProductionState = Arc<Mutex<RecordingManager>>;

/// Type alias for hotkey integration state
pub type HotkeyIntegrationState =
    Arc<Mutex<HotkeyIntegration<TauriEventEmitter, TauriEventEmitter, TauriEventEmitter>>>;

/// Type alias for transcription service state
pub type TranscriptionServiceState =
    Arc<RecordingTranscriptionService<TauriEventEmitter, TauriEventEmitter>>;

/// Type alias for audio monitor state (the thread handle)
pub type AudioMonitorState = Arc<AudioMonitorHandle>;

/// Type alias for hotkey service state (uses dynamic backend)
pub type HotkeyServiceState = crate::hotkey::HotkeyServiceDyn;

/// Type alias for keyboard capture state
pub type KeyboardCaptureState = Arc<Mutex<KeyboardCapture>>;

/// Concrete type for HotkeyService with dynamic backend (OS-selected)
pub type HotkeyServiceHandle = crate::hotkey::HotkeyServiceDyn;
