# Feature: Microphone Recording with Global Hotkey

**Created:** 2025-11-25
**Owner:** [Name]

## Description

Implement microphone recording functionality that allows users to capture audio via a global hotkey. The recording should work system-wide (even when the app is not focused) and provide both file output and in-memory storage for further processing (e.g., transcription).

Key requirements:
- Rust backend audio capture using cpal for native performance
- Global hotkey (Cmd+Shift+R) to toggle recording on/off
- Save recordings as WAV files to disk
- Keep recording in memory for immediate processing
- Minimal UI indicator (red dot) when recording is active
- Prioritize audio quality with no dropped samples

## Acceptance Criteria

> Use Given/When/Then format for testable scenarios

**Scenario 1: Start Recording**
- **Given** the app is running (foreground or background)
- **When** the user presses Cmd+Shift+R
- **Then** audio recording begins from the default microphone
- **And** a red recording indicator appears in the UI

**Scenario 2: Stop Recording**
- **Given** a recording is in progress
- **When** the user presses Cmd+Shift+R again
- **Then** audio recording stops
- **And** the recording is saved as a WAV file
- **And** the recording data is available in memory
- **And** the recording indicator disappears

**Scenario 3: Recording Quality**
- **Given** audio is being recorded
- **When** the system experiences normal load
- **Then** no audio samples are dropped (verified via overflow counter)

**Scenario 4: macOS Permissions**
- **Given** the app has not been granted microphone permissions
- **When** the user attempts to start recording
- **Then** macOS prompts for microphone permission
- **And** recording begins after permission is granted

## Technical Notes

See detailed implementation plan: `.claude/plans/microphone-recording.md`

**Architecture:**
- Lock-free ring buffer between cpal audio callback and writer thread
- Dedicated writer thread for file I/O (prevents blocking audio capture)
- Events emitted from Rust to notify frontend of state changes
- Single `audio.rs` module containing AudioRecorder, AudioBuffer, RecordingResult

**Dependencies:**
- Rust: cpal, ringbuf, hound, parking_lot, tauri-plugin-global-shortcut
- Frontend: @tauri-apps/plugin-global-shortcut

**Audio Config:**
- Sample rate: 48000 Hz
- Channels: 1 (mono)
- Bit depth: 16-bit
- Ring buffer: 2 seconds capacity
- Pre-allocation: 5 minutes

**Files to modify:**
- `src-tauri/Cargo.toml` - Add dependencies
- `src-tauri/src/lib.rs` - Plugin setup, commands, shortcut registration
- `src-tauri/src/audio.rs` - NEW: AudioRecorder implementation
- `src-tauri/capabilities/default.json` - Add global-shortcut permissions
- `src-tauri/tauri.conf.json` - Add macOS microphone entitlement
- `package.json` - Add frontend plugin dependency
- `src/App.tsx` - Add recording indicator and event listeners
- `src/hooks/useRecording.ts` - NEW: Recording state hook
- `src/components/RecordingIndicator.tsx` - NEW: UI indicator

## Definition of Done

- [ ] Global hotkey (Cmd+Shift+R) toggles recording
- [ ] Audio captured via Rust backend (cpal)
- [ ] Recordings saved as WAV files
- [ ] Recordings available in memory after stop
- [ ] Red recording indicator visible during recording
- [ ] No dropped samples under normal load
- [ ] macOS microphone permission handled correctly
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated
