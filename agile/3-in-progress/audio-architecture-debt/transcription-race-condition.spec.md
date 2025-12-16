---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
priority: P0
---

# Spec: Add semaphore to prevent concurrent batch+streaming transcription

## Description

The `SharedTranscriptionModel` has a critical race condition: `transcribe_file()` (batch mode) uses the RAII state guard, but `transcribe_samples()` (streaming mode for wake word) bypasses the state machine entirely. Both methods can run simultaneously on the same model, causing latency spikes and unpredictable behavior.

Add a semaphore or exclusive access mechanism to ensure batch and streaming transcription cannot execute concurrently.

## Acceptance Criteria

- [ ] Add `Semaphore` or `RwLock` to `SharedTranscriptionModel` to prevent concurrent transcription
- [ ] `transcribe_file()` acquires exclusive access before transcribing
- [ ] `transcribe_samples()` acquires exclusive access before transcribing
- [ ] If one mode is active, the other blocks or returns an error
- [ ] Document the mutual exclusion in code comments
- [ ] No deadlock introduced by new locking

## Test Cases

- [ ] Test concurrent calls to `transcribe_file()` + `transcribe_samples()` are serialized
- [ ] Test that two `transcribe_file()` calls don't interleave
- [ ] Test that semaphore is released on error paths
- [ ] Stress test with rapid alternating batch/streaming calls

## Dependencies

None

## Preconditions

- Existing `SharedTranscriptionModel` with separate batch and streaming paths

## Implementation Notes

**File:** `src-tauri/src/parakeet/shared.rs`

**Current state:**
- Lines 207-248: `transcribe_file()` uses `TranscribingGuard` (state machine)
- Lines 258-290: `transcribe_samples()` bypasses state (lines 270-272 comment explains why)

**Options:**
1. Use `tokio::sync::Semaphore` with 1 permit
2. Use `parking_lot::RwLock` for reader (streaming) / writer (batch) semantics
3. Use `std::sync::Mutex` wrapper around transcription operations

**Recommended approach:**
```rust
pub struct SharedTranscriptionModel {
    model: Arc<Mutex<Option<ParakeetModelWrapper>>>,
    state: Arc<Mutex<TranscriptionState>>,
    transcription_lock: Arc<Semaphore>,  // NEW: 1 permit
}

pub async fn transcribe_samples(&self, ...) -> Result<...> {
    let _permit = self.transcription_lock.acquire().await?;
    // ... existing logic
}

pub async fn transcribe_file(&self, ...) -> Result<...> {
    let _permit = self.transcription_lock.acquire().await?;
    // ... existing logic with guard
}
```

## Related Specs

- shared-transcription-model.spec.md (completed - original SharedTranscriptionModel)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs:385` (streaming)
- Production call site: `src-tauri/src/hotkey/integration.rs` (batch)
- Connects to: ListeningPipeline, HotkeyIntegration

## Integration Test

- Test location: `src-tauri/src/parakeet/shared.rs` (unit tests section)
- Verification: [ ] Integration test passes
