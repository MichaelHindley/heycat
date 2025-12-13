---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - eou-streaming-transcription.spec.md
  - streaming-audio-integration.spec.md
  - wire-up-transcription.spec.md
---

# Spec: Wire up streaming transcription to hotkey integration

## Description

Connect the StreamingTranscriber to the hotkey integration so that streaming mode actually works. Currently, the mode toggle UI exists but switching to Streaming mode has no effect - transcription always uses batch (TDT) mode. This spec wires up the EOU streaming transcriber to process audio chunks during recording and emit `transcription_partial` events in real-time.

## Acceptance Criteria

- [ ] `HotkeyIntegration` has a `streaming_transcriber` field
- [ ] Recording start creates streaming channel when mode is `Streaming`
- [ ] Streaming sender is passed to `audio_thread.start()` when mode is `Streaming`
- [ ] Consumer task spawns on recording start in streaming mode
- [ ] Consumer task reads chunks and calls `streaming_transcriber.process_samples()`
- [ ] `transcription_partial` events emitted during recording in streaming mode
- [ ] Recording stop in streaming mode calls `finalize()` instead of `spawn_transcription()`
- [ ] `transcription_completed` event emitted on streaming finalization
- [ ] Batch mode continues to work unchanged
- [ ] Mode is checked at recording start (not toggle time) for deterministic behavior

## Test Cases

- [ ] Unit test: HotkeyIntegration accepts streaming_transcriber via builder
- [ ] Unit test: Recording start in Batch mode passes `None` to audio_thread.start()
- [ ] Unit test: Recording start in Streaming mode passes `Some(sender)` to audio_thread.start()
- [ ] Unit test: Recording stop in Batch mode calls spawn_transcription()
- [ ] Unit test: Recording stop in Streaming mode calls finalize()
- [ ] Integration test: Full streaming flow emits partial events then completed event

## Dependencies

- `eou-streaming-transcription.spec.md` - StreamingTranscriber must exist
- `streaming-audio-integration.spec.md` - Audio channel support must exist
- `wire-up-transcription.spec.md` - TranscriptionManager must be wired up

## Preconditions

- `StreamingTranscriber` struct exists with `process_samples()` and `finalize()` methods
- Audio backend supports optional `StreamingAudioSender` parameter
- `TranscriptionManager` has `current_mode()` method
- EOU model can be loaded via `StreamingTranscriber::load_model()`

## Implementation Notes

### Architecture

```
STREAMING MODE FLOW:
┌─────────────┐     ┌──────────────────┐     ┌────────────────────┐
│ Audio Capture│────▶│ Channel (sender) │────▶│ Consumer Task      │
│ 160ms chunks │     │  SyncChannel     │     │ process_samples()  │
└─────────────┘     └──────────────────┘     └────────┬───────────┘
                                                       │
                                             ┌─────────▼─────────┐
                                             │StreamingTranscriber│
                                             │  emit partial     │
                                             └─────────┬─────────┘
                                                       │
                                             ┌─────────▼─────────┐
                                             │ Recording Stops   │
                                             │ → finalize()      │
                                             │ → emit completed  │
                                             │ → clipboard/cmd   │
                                             └───────────────────┘
```

### HotkeyIntegration Changes

Add to struct:
```rust
use crate::parakeet::{StreamingTranscriber, TranscriptionMode};
use crate::audio::{StreamingAudioSender, StreamingAudioReceiver};

pub struct HotkeyIntegration<R, T, C> {
    // ... existing fields ...
    streaming_transcriber: Option<Arc<Mutex<StreamingTranscriber<T>>>>,
    streaming_receiver: Option<Arc<Mutex<Option<StreamingAudioReceiver>>>>,
}
```

Add builder method:
```rust
pub fn with_streaming_transcriber(mut self, transcriber: Arc<Mutex<StreamingTranscriber<T>>>) -> Self {
    self.streaming_transcriber = Some(transcriber);
    self
}
```

### Recording Start Changes

In `handle_toggle()` when starting recording, check mode:
```rust
let mode = self.transcription_manager.as_ref()
    .map(|tm| tm.current_mode())
    .unwrap_or(TranscriptionMode::Batch);

let streaming_sender = match mode {
    TranscriptionMode::Streaming => {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);
        // Store receiver for consumer task
        if let Some(ref rx_holder) = self.streaming_receiver {
            *rx_holder.lock().unwrap() = Some(receiver);
        }
        // Spawn consumer task
        self.spawn_streaming_consumer();
        Some(sender)
    }
    TranscriptionMode::Batch => None,
};

// Pass to start_recording_impl
start_recording_impl(state, self.audio_thread.as_deref(), model_available, streaming_sender)
```

### Consumer Task

```rust
fn spawn_streaming_consumer(&self) {
    let receiver = match &self.streaming_receiver {
        Some(rx) => rx.clone(),
        None => return,
    };
    let transcriber = match &self.streaming_transcriber {
        Some(t) => t.clone(),
        None => return,
    };

    std::thread::spawn(move || {
        let rx_guard = receiver.lock().unwrap();
        if let Some(ref rx) = *rx_guard {
            while let Ok(chunk) = rx.recv() {
                if let Ok(mut t) = transcriber.lock() {
                    if let Err(e) = t.process_samples(&chunk) {
                        warn!("Streaming transcription error: {}", e);
                    }
                }
            }
        }
    });
}
```

### Recording Stop Changes

In `handle_toggle()` when stopping recording:
```rust
let mode = self.transcription_manager.as_ref()
    .map(|tm| tm.current_mode())
    .unwrap_or(TranscriptionMode::Batch);

match mode {
    TranscriptionMode::Batch => {
        self.spawn_transcription();  // existing flow
    }
    TranscriptionMode::Streaming => {
        self.finalize_streaming();
    }
}
```

### Finalize Streaming

```rust
fn finalize_streaming(&self) {
    let transcriber = match &self.streaming_transcriber {
        Some(t) => t.clone(),
        None => return,
    };

    // Drop receiver to signal consumer task to exit
    if let Some(ref rx_holder) = self.streaming_receiver {
        *rx_holder.lock().unwrap() = None;
    }

    // Finalize transcription
    if let Ok(mut t) = transcriber.lock() {
        match t.finalize() {
            Ok(text) => {
                // Handle command matching / clipboard same as batch
                self.handle_transcription_result(&text);
            }
            Err(e) => {
                error!("Streaming finalization failed: {}", e);
            }
        }
        t.reset();
    }
}
```

### start_recording_impl Signature Change

Update `commands/logic.rs`:
```rust
pub fn start_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
    model_available: bool,
    streaming_sender: Option<StreamingAudioSender>,  // NEW
) -> Result<(), String>
```

And pass to audio_thread.start():
```rust
audio_thread.start(buffer, streaming_sender)
```

## Related Specs

- `eou-streaming-transcription.spec.md` - Defines StreamingTranscriber
- `streaming-audio-integration.spec.md` - Defines audio channel support
- `wire-up-transcription.spec.md` - Batch mode wire-up (reference for pattern)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs:198` (handle_toggle)
- Production call site: `src-tauri/src/commands/logic.rs:52` (start_recording_impl)
- Connects to: `parakeet/streaming.rs`, `audio/thread.rs`, `audio/cpal_backend.rs`

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs` (extend existing)
- Verification: [ ] Integration test passes
