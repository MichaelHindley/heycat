---
status: in-progress
created: 2025-12-15
completed: null
dependencies:
  - safe-callback-channel
---

# Spec: Add transcription timeout with graceful recovery

## Description

Add a 60-second timeout to transcription operations to prevent indefinite hangs. Currently, if the Parakeet model hangs on corrupt audio or edge cases, the UI shows "Transcribing..." forever with no recovery path.

## Acceptance Criteria

- [ ] Add 60-second timeout to `HotkeyIntegration.spawn_transcription()`
- [ ] Add timeout to `WakeWordDetector.analyze()` transcription call
- [ ] Emit timeout error event to frontend
- [ ] Reset transcription state to Idle on timeout
- [ ] Subsequent transcriptions work correctly after timeout
- [ ] Timeout duration is configurable (default 60s)

## Test Cases

- [ ] Unit test: Timeout triggers after configured duration
- [ ] Unit test: State resets to Idle after timeout
- [ ] Unit test: Timeout error event contains useful message
- [ ] Integration test: UI shows timeout error (not stuck)
- [ ] Integration test: Recording works after timeout recovery

## Dependencies

- `safe-callback-channel` - Uses same async event pattern

## Preconditions

- Async event channel implemented
- Understanding of Tokio timeout patterns

## Implementation Notes

```rust
// src-tauri/src/hotkey/integration.rs

use tokio::time::{timeout, Duration};

const TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

async fn spawn_transcription(&self, file_path: PathBuf) {
    let transcriber = self.transcription_manager.clone();

    let result = timeout(
        Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            transcriber.transcribe(&file_path)
        })
    ).await;

    match result {
        Ok(Ok(Ok(text))) => {
            // Success - emit transcription complete
            self.emitter.emit_transcription_completed(text);
        }
        Ok(Ok(Err(e))) => {
            // Transcription error
            self.emitter.emit_transcription_error(e.to_string());
        }
        Ok(Err(e)) => {
            // Join error
            self.emitter.emit_transcription_error(format!("Task failed: {}", e));
        }
        Err(_) => {
            // Timeout!
            error!("Transcription timed out after {}s", TRANSCRIPTION_TIMEOUT_SECS);
            self.emitter.emit_transcription_error(
                format!("Transcription timed out after {} seconds", TRANSCRIPTION_TIMEOUT_SECS)
            );
        }
    }

    // Always reset state
    if let Err(e) = self.transcription_manager.reset_to_idle() {
        warn!("Failed to reset state: {}", e);
    }
}
```

Key files:
- `hotkey/integration.rs:441` - Async timeout wrapper using `tokio::time::timeout`
- `listening/detector.rs:381-395` - Post-hoc timeout check

### Known Limitations

**WakeWordDetector timeout is post-hoc, not preemptive:**

The `WakeWordDetector.analyze()` timeout check happens AFTER transcription completes.
If the Parakeet model truly hangs indefinitely, this code path will never execute.
This is acceptable for wake word detection because:
1. The audio window is only ~2 seconds (short)
2. The analysis loop has natural breaks that prevent indefinite hangs
3. True preemptive cancellation would require thread-based timeout (complex, error-prone)

The `HotkeyIntegration.spawn_transcription()` uses proper async timeout which can
preemptively cancel the task.

## Related Specs

- `safe-callback-channel.spec.md` - Prerequisite
- `state-transition-guard.spec.md` - Related (both improve robustness)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: `TranscriptionManager`, Frontend event handlers

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

## Review

### Verdict
**NEEDS_WORK** - Missing unit tests for timeout paths

**Reviewed:** 2025-12-15
**Reviewer:** Claude (Independent Review Agent)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Add 60-second timeout to `HotkeyIntegration.spawn_transcription()` | ✅ | `integration.rs:33` defines `DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60`, and `integration.rs:441` wraps transcription with `tokio::time::timeout(timeout_duration, transcription_future).await` |
| Add timeout to `WakeWordDetector.analyze()` transcription call | ✅ | `detector.rs:12-14` defines `DEFAULT_WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS: u64 = 10` (appropriate for ~2s audio windows), and `detector.rs:381-395` checks duration post-transcription and returns `WakeWordError::TranscriptionTimeout` |
| Emit timeout error event to frontend | ✅ | `integration.rs:467-478` on timeout emits `TranscriptionErrorPayload` with message "Transcription timed out after X seconds..." |
| Reset transcription state to Idle on timeout | ✅ | `integration.rs:473-475` calls `transcription_manager.reset_to_idle()` on timeout, and `detector.rs:391-395` returns error (caller handles state) |
| Subsequent transcriptions work correctly after timeout | ⚠️ | Structurally supported (state reset, semaphore permit released), but no explicit test verifies this |
| Timeout duration is configurable (default 60s) | ✅ | `integration.rs:195-202` provides `with_transcription_timeout(timeout: Duration)` builder method, `detector.rs:43` has `transcription_timeout_secs` in config |

### Test Coverage

| Test Case | Status | Evidence |
|-----------|--------|----------|
| Unit test: Timeout triggers after configured duration | ❌ | No unit test found in `integration_test.rs` or `detector.rs` tests that verifies timeout triggers |
| Unit test: State resets to Idle after timeout | ❌ | No unit test found that verifies state reset after timeout |
| Unit test: Timeout error event contains useful message | ✅ | `detector.rs:835-838` tests `WakeWordError::TranscriptionTimeout` error message formatting |
| Integration test: UI shows timeout error (not stuck) | ❌ | No integration test found; frontend hook (`useTranscription.ts:65-72`) handles errors but no test simulates timeout scenario |
| Integration test: Recording works after timeout recovery | ❌ | No test found that verifies recording continues working after a timeout |

### Issues Found

1. **Missing timeout unit tests for HotkeyIntegration**
   - Location: `src-tauri/src/hotkey/integration_test.rs`
   - Problem: No test verifies that `spawn_transcription()` correctly handles the timeout case. The timeout logic at lines 467-478 is untested.
   - Suggestion: Add a test that uses `with_transcription_timeout(Duration::from_millis(10))` with a mock transcription manager that delays longer than the timeout, then verify:
     1. `transcription_error` event is emitted
     2. Error message contains "timed out"
     3. State is reset to idle

2. **Missing recovery test**
   - Location: `src-tauri/src/hotkey/integration_test.rs`
   - Problem: No test verifies that after a timeout, subsequent transcriptions work correctly.
   - Suggestion: After triggering a timeout, trigger another transcription and verify it succeeds.

3. **WakeWordDetector timeout is post-hoc, not preemptive**
   - Location: `src-tauri/src/listening/detector.rs:381-395`
   - Problem: The timeout check in `WakeWordDetector.analyze()` happens AFTER transcription completes. If the Parakeet model truly hangs indefinitely, this code path will never execute. The comment at line 382-383 acknowledges this: "Since transcription is synchronous, we can only detect this after completion."
   - Suggestion: This is a known limitation and may be acceptable for the ~2s audio windows used in wake word detection. However, the spec acceptance criteria says "Add timeout to WakeWordDetector.analyze() transcription call" which implies preemptive cancellation. Consider:
     - Documenting this as a known limitation in the spec
     - OR wrapping the transcription call in a thread with actual timeout/kill capability (more complex)

4. **No frontend integration test**
   - Location: `src/hooks/useTranscription.test.ts`
   - Problem: While the hook correctly handles `transcription_error` events, there's no test that specifically simulates a timeout error scenario.
   - Suggestion: Add a test that emits a mock `transcription_error` event with a timeout message and verifies the hook updates state correctly.
