# Segfault Investigation: Combined Tests

**Status:** Investigation Complete
**Date:** 2025-12-29
**Reproduced:** Yes (SIGSEGV signal 11 confirmed during investigation)

## Summary

Intermittent SIGSEGV occurs when running `bun run test && cargo test` due to parallel Rust test execution accessing a global Swift audio engine singleton without test-level synchronization.

## Root Cause

The `SharedAudioEngineManager` in Swift is a **global singleton** (`SharedAudioEngineManager.shared`) that provides unified audio engine functionality for both recording and monitoring. Multiple Rust tests access this singleton concurrently:

### Conflicting Tests

| Test | File | Action | Risk |
|------|------|--------|------|
| `test_engine_running_query` | `audio/monitor.rs:298` | Calls `swift::audio_engine_stop()` | Stops engine another test may be using |
| `test_audio_engine_is_running_query` | `swift.rs:328` | Calls `audio_engine_stop()` | Stops engine another test may be using |
| `test_init_without_device` | `audio/monitor.rs:307` | Starts engine via `init()` | May conflict with stop operations |
| `test_init_is_idempotent` | `audio/monitor.rs:314` | Starts engine twice | Racing with stops |
| `test_start_after_init_works` | `audio/monitor.rs:323` | Full init→start→stop cycle | Full lifecycle conflict |
| `test_spawn_and_drop` | `audio/monitor.rs:278` | Spawns monitor thread | Thread shutdown races |
| `test_shutdown` | `audio/monitor.rs:292` | Explicit shutdown | Shutdown races |

### Race Condition Scenario

```
Thread 1 (test_init_without_device):    Thread 2 (test_engine_running_query):
  |                                       |
  | init() → starts engine                |
  |                                       | audio_engine_stop() ← stops it!
  | ... engine now stopped ...            |
  | subsequent operations fail            |
```

### Why It's Intermittent

1. **Cargo parallel execution:** By default, `cargo test` runs tests in parallel across CPU cores
2. **Timing-dependent:** Race only triggers when specific tests overlap
3. **macOS AVFoundation state:** AVAudioEngine's internal state machine may crash when operations interleave unexpectedly

### Swift Layer Thread Safety

The Swift `SharedAudioEngineManager` uses a serial `DispatchQueue` (`audioQueue`) for thread safety, but this only synchronizes **within a single FFI call**. It does not prevent:
- Test A's FFI call completing
- Test B's FFI call starting and stopping the engine
- Test A making another FFI call to a now-stopped engine

```swift
// audioQueue.sync only protects THIS call, not the entire test
func startEngine(deviceName: String?) -> Bool {
    return audioQueue.sync {
        // ...thread-safe within this call
    }
}
```

## Evidence

1. Segfault only occurs with **combined tests** (vitest then cargo test in same shell)
2. Both test suites pass when run **separately**
3. Reproduction is intermittent (race condition dependent on timing)
4. Multiple tests explicitly call `audio_engine_stop()` to "ensure clean state"

## Technical Details

### Shared Resource: `SharedAudioEngineManager`

Location: `src-tauri/swift-lib/Sources/swift-lib/SharedAudioEngine.swift`

```swift
private class SharedAudioEngineManager {
    static let shared = SharedAudioEngineManager()  // Global singleton
    private var audioEngine: AVAudioEngine?
    private var isRunning = false
    // ...
}
```

### FFI Boundary

Rust calls Swift via `swift-rs` FFI:
- `swift_audio_engine_start()` → `audioEngineStart()` → `shared.startEngine()`
- `swift_audio_engine_stop()` → `audioEngineStop()` → `shared.stopEngine()`
- `swift_audio_engine_is_running()` → `audioEngineIsRunning()` → `shared.getIsRunning()`

### Why Vitest Doesn't Cause This Alone

Vitest (frontend tests) runs in a **separate process** and doesn't touch Swift FFI. The frontend mocks all Tauri invokes, so no real audio engine calls are made.

The issue only manifests when:
1. Vitest finishes (process exits cleanly)
2. Cargo test starts (new process)
3. Cargo test runs Rust tests in parallel
4. Multiple Rust tests race on the Swift singleton

## Recommended Fix

See spec: `add-test-isolation-between-suites` for implementation details.

The fix should:
1. **Add test synchronization** for tests that access the Swift audio engine
2. **Run audio-related tests serially** using `serial_test` crate or cargo's `--test-threads=1` for specific modules
3. **Consider test isolation** via a mutex that all audio-engine tests must acquire

## Files to Modify

- `src-tauri/Cargo.toml` - Add `serial_test` dependency (dev-dependency)
- `src-tauri/src/audio/monitor.rs` - Mark tests with `#[serial]`
- `src-tauri/src/swift.rs` - Mark tests with `#[serial]`
