---
status: in-progress
created: 2025-12-15
completed: null
dependencies: []
---

# Spec: Replace unsafe callbacks with async event channel

## Description

Replace the direct callback invocation in ListeningPipeline's analysis thread with an async event channel. Currently, the wake word callback runs on the analysis thread while potentially holding locks, creating deadlock risk if the callback tries to acquire additional locks.

## Acceptance Criteria

- [ ] Create `WakeWordEvent` enum for event types (Detected, Unavailable, Error)
- [ ] Add `tokio::sync::mpsc` channel to `ListeningPipeline`
- [ ] Analysis thread sends events via channel instead of calling callback directly
- [ ] `HotkeyIntegration` subscribes to event channel
- [ ] No callbacks execute on the analysis thread
- [ ] Wake word detection still triggers recording correctly
- [ ] No deadlock possible from event handling

## Test Cases

- [ ] Unit test: Events sent through channel are received
- [ ] Unit test: Multiple events can be queued without blocking
- [ ] Unit test: Channel handles backpressure gracefully
- [ ] Integration test: Wake word detection triggers recording
- [ ] Stress test: Rapid wake word events don't cause deadlock

## Dependencies

None - can be done in parallel with shared-transcription-model

## Preconditions

- Existing callback mechanism works (tests pass)
- Understanding of current callback flow

## Implementation Notes

```rust
// src-tauri/src/listening/events.rs
pub enum WakeWordEvent {
    Detected {
        text: String,
        confidence: f32,
        audio_buffer: Vec<f32>,
    },
    Unavailable { reason: String },
    Error { message: String },
}

// In ListeningPipeline
pub struct ListeningPipeline {
    // ... existing fields
    event_tx: tokio::sync::mpsc::Sender<WakeWordEvent>,
}

// In analysis thread (pipeline.rs:474-477)
// OLD: callback();
// NEW: event_tx.try_send(WakeWordEvent::Detected { ... });

// In HotkeyIntegration
pub async fn start_event_loop(&self, mut event_rx: mpsc::Receiver<WakeWordEvent>) {
    while let Some(event) = event_rx.recv().await {
        match event {
            WakeWordEvent::Detected { .. } => self.handle_wake_word_detected(),
            // ...
        }
    }
}
```

Key files:
- `listening/pipeline.rs:474-477` - Replace callback with channel send
- `listening/events.rs` - New file for event types
- `hotkey/integration.rs` - Subscribe to channel

## Related Specs

- `shared-transcription-model.spec.md` - Can be done in parallel
- `transcription-timeout.spec.md` - Depends on this (uses same async pattern)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (event subscription)
- Connects to: `ListeningPipeline`, `HotkeyIntegration`, `RecordingManager`

## Integration Test

- Test location: `src-tauri/src/listening/pipeline_test.rs`
- Verification: [ ] Integration test passes
