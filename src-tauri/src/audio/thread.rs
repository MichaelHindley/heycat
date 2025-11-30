// Dedicated audio thread for capturing audio
//
// This module provides a thread-safe interface to audio capture.
// CpalBackend contains cpal::Stream which is NOT Send+Sync, so we isolate
// it on a dedicated thread and communicate via channels.

use super::{AudioBuffer, AudioCaptureBackend, CpalBackend};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Commands sent to the audio thread
#[derive(Debug)]
pub enum AudioCommand {
    /// Start capturing audio into the provided buffer
    Start(AudioBuffer),
    /// Stop capturing audio
    Stop,
    /// Shutdown the audio thread
    Shutdown,
}

/// Handle to the audio capture thread
///
/// This handle is Send + Sync and can be safely shared across threads.
/// Commands are sent via channel to the dedicated audio thread.
pub struct AudioThreadHandle {
    sender: Sender<AudioCommand>,
    _thread: JoinHandle<()>,
}

impl AudioThreadHandle {
    /// Spawn a new audio capture thread
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::channel();

        let thread = thread::spawn(move || {
            audio_thread_main(receiver);
        });

        Self {
            sender,
            _thread: thread,
        }
    }

    /// Start audio capture into the provided buffer
    ///
    /// Returns Ok(()) if the command was sent successfully.
    /// The actual capture may still fail on the audio thread.
    pub fn start(&self, buffer: AudioBuffer) -> Result<(), AudioThreadError> {
        self.sender
            .send(AudioCommand::Start(buffer))
            .map_err(|_| AudioThreadError::ThreadDisconnected)
    }

    /// Stop audio capture
    pub fn stop(&self) -> Result<(), AudioThreadError> {
        self.sender
            .send(AudioCommand::Stop)
            .map_err(|_| AudioThreadError::ThreadDisconnected)
    }

    /// Shutdown the audio thread gracefully
    pub fn shutdown(&self) -> Result<(), AudioThreadError> {
        self.sender
            .send(AudioCommand::Shutdown)
            .map_err(|_| AudioThreadError::ThreadDisconnected)
    }
}

/// Errors from audio thread operations
#[derive(Debug, Clone, PartialEq)]
pub enum AudioThreadError {
    /// The audio thread has disconnected
    ThreadDisconnected,
}

/// Main loop for the audio thread
///
/// Creates CpalBackend and processes commands.
/// This runs on a dedicated thread where CpalBackend can safely live.
#[cfg_attr(coverage_nightly, coverage(off))]
fn audio_thread_main(receiver: Receiver<AudioCommand>) {
    eprintln!("[audio-thread] Audio thread started, creating CpalBackend...");
    let mut backend = CpalBackend::new();
    eprintln!("[audio-thread] CpalBackend created, waiting for commands...");

    while let Ok(command) = receiver.recv() {
        match command {
            AudioCommand::Start(buffer) => {
                eprintln!("[audio-thread] Received START command");
                match backend.start(buffer) {
                    Ok(()) => eprintln!("[audio-thread] Audio capture started successfully"),
                    Err(e) => eprintln!("[audio-thread] Audio capture failed to start: {:?}", e),
                }
            }
            AudioCommand::Stop => {
                eprintln!("[audio-thread] Received STOP command");
                match backend.stop() {
                    Ok(()) => eprintln!("[audio-thread] Audio capture stopped successfully"),
                    Err(e) => eprintln!("[audio-thread] Audio capture failed to stop: {:?}", e),
                }
            }
            AudioCommand::Shutdown => {
                eprintln!("[audio-thread] Received SHUTDOWN command");
                let _ = backend.stop();
                break;
            }
        }
    }
    eprintln!("[audio-thread] Audio thread exiting");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_thread_handle_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AudioThreadHandle>();
    }

    #[test]
    fn test_spawn_and_shutdown() {
        let handle = AudioThreadHandle::spawn();
        assert!(handle.shutdown().is_ok());
    }

    #[test]
    fn test_start_stop_commands() {
        let handle = AudioThreadHandle::spawn();
        let buffer = AudioBuffer::new();

        // Start should succeed (command sent)
        assert!(handle.start(buffer).is_ok());

        // Stop should succeed
        assert!(handle.stop().is_ok());

        // Shutdown
        assert!(handle.shutdown().is_ok());
    }
}
