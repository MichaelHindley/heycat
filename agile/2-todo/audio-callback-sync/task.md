---
status: todo
type: task
created: 2025-12-23
owner: null
---

# Task: Replace Thread Sleep with Proper Audio Callback Synchronization

**Created:** 2025-12-23

## Problem

The current audio recording stop sequence uses a hacky `thread::sleep(10ms)` to wait for in-flight audio callbacks to complete. This is unreliable and wastes time.

**Current code in `cpal_backend.rs:stop()`:**
```rust
drop(stream);
// Brief wait for any in-flight callbacks to check the flag and exit
std::thread::sleep(std::time::Duration::from_millis(10));
```

## Proposed Solution

Replace the sleep with proper synchronization using one of:

1. **Condition Variable** - Callback signals completion, stop() waits on condition
2. **Barrier** - Both callback and stop() synchronize at barrier point
3. **Atomic counter** - Track in-flight callbacks, wait until counter reaches 0

## Implementation Notes

- The `stop_flag` atomic already prevents new sample processing
- Need to ensure ALL callbacks have exited before flushing
- Consider using `std::sync::Barrier` or `parking_lot::Condvar`

## Context

This issue was created during the fix for the "audio degradation on subsequent recordings" bug. The sleep was added as a quick fix to prevent race conditions where callbacks pushed stale samples to the SharedDenoiser after flush.

## Files to Modify

- `src-tauri/src/audio/cpal_backend.rs`

## Acceptance Criteria

- [ ] Thread sleep removed from stop()
- [ ] Proper synchronization mechanism in place
- [ ] No regression in audio quality
- [ ] Tests pass
