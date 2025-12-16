---
status: pending
created: 2025-12-16
completed: null
dependencies: []
priority: P0
---

# Spec: Make event subscription mandatory before pipeline start

## Description

Currently if `subscribe_events()` isn't called before `start()`, a channel is created but events are silently dropped (lines 280-284). This is a common source of missed wake word detections. The code only logs a warning but continues.

Make event subscription mandatory by returning an error from `start()` if no subscriber is configured.

## Acceptance Criteria

- [ ] `start()` returns error if `subscribe_events()` was not called
- [ ] Error message clearly states "must call subscribe_events() before start()"
- [ ] Remove the silent channel creation in `start()`
- [ ] Existing callers updated to call `subscribe_events()` before `start()`
- [ ] Documentation updated to reflect the requirement

## Test Cases

- [ ] Test `start()` without `subscribe_events()` returns clear error
- [ ] Test `start()` after `subscribe_events()` succeeds
- [ ] Test events are actually delivered to subscriber
- [ ] Test calling `subscribe_events()` multiple times is safe

## Dependencies

None

## Preconditions

- Events-based wake word detection already implemented

## Implementation Notes

**File:** `src-tauri/src/listening/pipeline.rs`

**Current behavior (lines 280-284):**
```rust
let event_tx = self.event_tx.clone().unwrap_or_else(|| {
    crate::warn!("[pipeline] No event subscriber configured, events will be dropped");
    let (tx, _rx) = tokio_mpsc::channel(EVENT_CHANNEL_BUFFER_SIZE);
    tx  // Receiver is dropped immediately!
});
```

**Proposed change:**
```rust
pub fn start<E: 'static>(
    &mut self,
    audio_handle: &AudioThreadHandle,
    emitter: Arc<E>,
) -> Result<(), WakeWordError> {
    let event_tx = self.event_tx.clone()
        .ok_or(WakeWordError::NoEventSubscriber)?;
    // ... rest of start logic
}
```

**Add error variant:**
```rust
pub enum WakeWordError {
    // ... existing variants
    NoEventSubscriber,  // NEW
}
```

**Update Display:**
```rust
WakeWordError::NoEventSubscriber => write!(
    f, "Must call subscribe_events() before start()"
),
```

**Callers to update:**
- `src-tauri/src/listening/manager.rs` - ListeningManager::enable_listening()

## Related Specs

- safe-callback-channel.spec.md (completed - introduced events)
- thread-coordination-fix.spec.md (complementary)

## Integration Points

- Production call site: `src-tauri/src/listening/manager.rs`
- Connects to: ListeningManager

## Integration Test

- Test location: `src-tauri/src/listening/pipeline.rs` (test module)
- Verification: [ ] Integration test passes
