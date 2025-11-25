# Microphone Recording Feature - Implementation Plan

## Overview

Implement microphone recording with global hotkey control (Cmd+Shift+R) for the heycat Tauri v2 desktop app.

**Requirements:**
- Rust backend audio capture (cpal)
- Save to file AND keep in memory
- Minimal UI indicator (red dot)
- Global hotkey: Cmd+Shift+R
- Prioritize audio quality (no dropped samples)

---

## Architecture

```
Frontend (React)              Backend (Rust)
┌──────────────────┐         ┌─────────────────────────────────────┐
│ RecordingIndicator│←─events─│  Global Shortcut Handler            │
│ (red dot UI)     │         │         │                           │
└──────────────────┘         │         ▼                           │
                             │  ┌─────────────┐                    │
                             │  │AudioRecorder│                    │
                             │  └──────┬──────┘                    │
                             │         │                           │
                             │  ┌──────▼──────┐   ┌─────────────┐  │
                             │  │ cpal audio  │──▶│ Ring Buffer │  │
                             │  │   thread    │   │ (lock-free) │  │
                             │  └─────────────┘   └──────┬──────┘  │
                             │                           │         │
                             │                    ┌──────▼──────┐  │
                             │                    │Writer Thread│  │
                             │                    │ (file + mem)│  │
                             │                    └─────────────┘  │
                             └─────────────────────────────────────┘
```

**Key design decisions:**
1. **Lock-free ring buffer** between audio callback and writer thread (no dropped samples)
2. **Dedicated writer thread** handles file I/O without blocking audio capture
3. **Events for state changes** - hotkey triggers in Rust, emits events to frontend
4. **Single audio module file** - keep it simple, split into modules only if >300 lines

---

## Dependencies

### Rust (Cargo.toml)
```toml
cpal = "0.16"                      # Audio capture
ringbuf = "0.4"                    # Lock-free SPSC ring buffer
hound = "3.5"                      # WAV file writing
parking_lot = "0.12"               # Fast mutexes
tauri-plugin-global-shortcut = "2" # Global hotkeys
```

### Frontend (package.json)
```json
"@tauri-apps/plugin-global-shortcut": "^2"
```

---

## Implementation Steps

### Phase 1: Dependencies & Permissions

1. **Update `src-tauri/Cargo.toml`** - Add cpal, ringbuf, hound, parking_lot, tauri-plugin-global-shortcut

2. **Update `package.json`** - Add @tauri-apps/plugin-global-shortcut

3. **Update `src-tauri/capabilities/default.json`** - Add global-shortcut permissions:
   ```json
   "global-shortcut:allow-register",
   "global-shortcut:allow-unregister"
   ```

4. **Update `src-tauri/tauri.conf.json`** - Add macOS microphone entitlement:
   ```json
   "bundle": {
     "macOS": {
       "infoPlist": {
         "NSMicrophoneUsageDescription": "This app requires microphone access to record audio."
       }
     }
   }
   ```

### Phase 2: Rust Audio Module

Create `src-tauri/src/audio.rs` with:

1. **AudioConfig constants**
   - Sample rate: 48000 Hz
   - Channels: 1 (mono)
   - Bits per sample: 16
   - Ring buffer: 2 seconds capacity
   - Memory pre-allocation: 5 minutes

2. **AudioBuffer struct** - Wrapper around ringbuf SPSC
   - `push_samples()` - called from audio callback (lock-free)
   - `drain_to()` - called from writer thread
   - Track overflow count for diagnostics

3. **AudioRecorder struct**
   - State: `is_recording` (AtomicBool)
   - cpal Stream (Option)
   - Ring buffer (Arc)
   - Memory buffer (Arc<Mutex<Vec<i16>>>)
   - Writer thread handle

4. **Methods**
   - `new()` - Initialize with pre-allocated buffers
   - `start_recording(file_path)` - Start cpal stream + writer thread
   - `stop_recording()` - Stop stream, join writer, return RecordingResult
   - `is_recording()` - Query state

5. **RecordingResult struct**
   - samples: Vec<i16> (in-memory copy)
   - duration_secs: f32
   - file_path: Option<String>
   - overflow_count: usize (for diagnostics)

### Phase 3: Tauri Integration

Update `src-tauri/src/lib.rs`:

1. **App state**
   ```rust
   struct AppState {
       recorder: Mutex<AudioRecorder>,
   }
   ```

2. **Commands**
   - `start_recording()` - Start recording, emit "recording:started" event
   - `stop_recording()` - Stop recording, emit "recording:stopped" event
   - `get_recording_status()` - Return bool

3. **Global shortcut setup** (in `setup` hook)
   - Register Cmd+Shift+R (or Ctrl+Shift+R on Windows/Linux)
   - On trigger: toggle recording state, emit appropriate event

4. **Plugin registration**
   ```rust
   .plugin(tauri_plugin_global_shortcut::Builder::new().build())
   ```

### Phase 4: Frontend UI

1. **Create `src/hooks/useRecording.ts`**
   - Listen for "recording:started" and "recording:stopped" events
   - Expose `isRecording` state
   - Expose `toggleRecording()` function

2. **Create `src/components/RecordingIndicator.tsx`**
   - Simple red dot with pulse animation
   - Fixed position top-right
   - Only visible when recording

3. **Update `src/App.tsx`**
   - Import and use `useRecording` hook
   - Render `<RecordingIndicator />` when recording
   - Optionally add manual toggle button

4. **Update `src/App.css`**
   ```css
   .recording-indicator {
     position: fixed;
     top: 16px;
     right: 16px;
     width: 12px;
     height: 12px;
     background: #ff4444;
     border-radius: 50%;
     animation: pulse 1s infinite;
   }
   ```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add dependencies |
| `src-tauri/src/lib.rs` | Plugin setup, state, commands, shortcut registration |
| `src-tauri/src/audio.rs` | **NEW** - AudioRecorder, AudioBuffer, RecordingResult |
| `src-tauri/capabilities/default.json` | Add global-shortcut permissions |
| `src-tauri/tauri.conf.json` | Add macOS microphone entitlement |
| `package.json` | Add @tauri-apps/plugin-global-shortcut |
| `src/App.tsx` | Add RecordingIndicator, useRecording hook |
| `src/App.css` | Add recording indicator styles |
| `src/hooks/useRecording.ts` | **NEW** - Recording state hook |
| `src/components/RecordingIndicator.tsx` | **NEW** - UI indicator component |

---

## Audio Configuration

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Sample Rate | 48000 Hz | Standard, well-supported |
| Channels | 1 (mono) | Voice recording, 50% less data |
| Bit Depth | 16-bit | CD quality, sufficient for voice |
| Ring Buffer | 2 seconds | Handles disk I/O latency spikes |
| Pre-allocation | 5 minutes | Avoids reallocation during recording |

---

## Potential Risks & Mitigations

1. **macOS microphone permissions** - Must add NSMicrophoneUsageDescription or app will crash silently

2. **cpal Stream lifetime** - Stream must be kept alive while recording; store in Option and take/replace

3. **Thread cleanup on crash** - Implement Drop trait on AudioRecorder to ensure cleanup

4. **Long recordings** - Pre-allocate 5 minutes; for longer recordings, memory grows (~10MB/minute mono 48kHz 16-bit)

5. **Hotkey conflict** - Cmd+Shift+R may conflict with browser hard refresh; accept for now, make configurable later

---

## Testing Notes

Given the TCR workflow with 80% coverage requirement:
- Audio capture is hard to unit test (hardware dependency)
- Focus tests on: buffer management, state transitions, error handling
- Consider mock AudioRecorder trait for testing commands
