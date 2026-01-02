//! Mock event emitters for testing.

use crate::events::{
    CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    RecordingCancelledPayload, RecordingErrorPayload, RecordingStartedPayload,
    RecordingStoppedPayload, TranscriptionCompletedPayload, TranscriptionErrorPayload,
    TranscriptionStartedPayload,
};
use std::sync::{Arc, Mutex};

/// Mock event emitter that records all events for test assertions.
#[derive(Default, Clone)]
pub struct MockEmitter {
    pub started: Arc<Mutex<Vec<RecordingStartedPayload>>>,
    pub stopped: Arc<Mutex<Vec<RecordingStoppedPayload>>>,
    pub cancelled: Arc<Mutex<Vec<RecordingCancelledPayload>>>,
    pub errors: Arc<Mutex<Vec<RecordingErrorPayload>>>,
    pub transcription_started: Arc<Mutex<Vec<TranscriptionStartedPayload>>>,
    pub transcription_completed: Arc<Mutex<Vec<TranscriptionCompletedPayload>>>,
    pub transcription_errors: Arc<Mutex<Vec<TranscriptionErrorPayload>>>,
    pub command_matched: Arc<Mutex<Vec<CommandMatchedPayload>>>,
    pub command_executed: Arc<Mutex<Vec<CommandExecutedPayload>>>,
    pub command_failed: Arc<Mutex<Vec<CommandFailedPayload>>>,
    pub command_ambiguous: Arc<Mutex<Vec<CommandAmbiguousPayload>>>,
    pub key_blocking_unavailable:
        Arc<Mutex<Vec<crate::events::hotkey_events::KeyBlockingUnavailablePayload>>>,
}

impl MockEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn started_count(&self) -> usize {
        self.started.lock().unwrap().len()
    }

    pub fn stopped_count(&self) -> usize {
        self.stopped.lock().unwrap().len()
    }

    pub fn cancelled_count(&self) -> usize {
        self.cancelled.lock().unwrap().len()
    }

    pub fn last_cancelled(&self) -> Option<RecordingCancelledPayload> {
        self.cancelled.lock().unwrap().last().cloned()
    }

    pub fn key_blocking_unavailable_count(&self) -> usize {
        self.key_blocking_unavailable.lock().unwrap().len()
    }
}

impl crate::events::RecordingEventEmitter for MockEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        self.started.lock().unwrap().push(payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        self.stopped.lock().unwrap().push(payload);
    }

    fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload) {
        self.cancelled.lock().unwrap().push(payload);
    }

    fn emit_recording_error(&self, payload: RecordingErrorPayload) {
        self.errors.lock().unwrap().push(payload);
    }
}

impl crate::events::TranscriptionEventEmitter for MockEmitter {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
        self.transcription_started.lock().unwrap().push(payload);
    }

    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
        self.transcription_completed.lock().unwrap().push(payload);
    }

    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
        self.transcription_errors.lock().unwrap().push(payload);
    }
}

impl crate::events::CommandEventEmitter for MockEmitter {
    fn emit_command_matched(&self, payload: CommandMatchedPayload) {
        self.command_matched.lock().unwrap().push(payload);
    }

    fn emit_command_executed(&self, payload: CommandExecutedPayload) {
        self.command_executed.lock().unwrap().push(payload);
    }

    fn emit_command_failed(&self, payload: CommandFailedPayload) {
        self.command_failed.lock().unwrap().push(payload);
    }

    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
        self.command_ambiguous.lock().unwrap().push(payload);
    }
}

impl crate::events::HotkeyEventEmitter for MockEmitter {
    fn emit_key_blocking_unavailable(
        &self,
        payload: crate::events::hotkey_events::KeyBlockingUnavailablePayload,
    ) {
        self.key_blocking_unavailable.lock().unwrap().push(payload);
    }
}
