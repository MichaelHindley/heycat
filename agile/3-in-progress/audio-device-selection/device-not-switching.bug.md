---
status: in-progress
severity: major
origin: manual
owner: Michael
created: 2025-12-17
completed: null
parent_feature: "audio-device-selection"
parent_spec: null
---

# Bug: Device Not Switching

**Created:** 2025-12-17
**Severity:** Major

## Problem Description

**What's happening:**
- When switching microphone (audio input device) in the app, the change is not being applied
- Backend logs show it always uses the default device, despite UI showing a different selection
- Some logs indicate the selected device is being used for monitoring, but actual audio capture uses the default
- The headset only works correctly when set as the system default device
- Additionally, the device appears stuck in "communication mode" (different audio profile used for calls vs music playback)

**What should happen:**
- Selecting a different microphone should immediately switch the audio input to that device
- The selected device should be used for both monitoring and recording
- Device should not be forced into communication mode

## Steps to Reproduce

1. Open the app
2. Go to settings
3. Select a different microphone from the dropdown (not the system default)
4. Start recording
5. Check backend logs - observe it still uses default device
6. Audio from the selected device is not captured unless it's set as system default

## Root Cause

**Status:** Identified

The selected device only affects the **audio level meter preview** but doesn't affect actual recording or listening. The root cause is that device selection is not being passed through the full pipeline:

| Component | File | Issue |
|-----------|------|-------|
| Recording Start | `useRecording.ts:53` | Device not passed to `start_recording` command |
| Listening Enable | `useListening.ts:54` | Device not passed to `enable_listening` command |
| Listening Command | `commands/mod.rs:356` | `enable_listening` has no `device_name` parameter |
| Pipeline Start | `listening/pipeline.rs:293` | Uses `.start()` not `.start_with_device()` |

**Why monitoring works but recording/listening don't:**
- `useAudioLevelMonitor.ts:51-53` correctly passes `deviceName` to `start_audio_monitor`
- But `useRecording.ts:53` calls `start_recording` with no device parameter
- And `enable_listening` command has no device parameter at all

**Communication mode issue:** Needs separate investigation - may be OS-level behavior when app accesses microphone.

## Fix Approach

### 1. Recording Path (useRecording.ts)
- Read selected device from settings store
- Pass `deviceName` parameter to `invoke("start_recording", { deviceName })`

### 2. Listening Path (multiple files)
- `useListening.ts`: Read selected device and pass to `enable_listening`
- `commands/mod.rs`: Add `device_name: Option<String>` parameter to `enable_listening` command
- `commands/logic.rs`: Pass device to `enable_listening_impl`
- `listening/pipeline.rs`: Change `.start()` to `.start_with_device(device_name)`

### 3. Pipeline Plumbing
- Update `ListeningPipeline::start()` signature to accept `device_name: Option<String>`
- Pass device through to audio thread

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken
- [ ] Selected device is actually used for audio capture (not just monitoring)
- [ ] Device is not forced into communication mode

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Switch to non-default microphone and record | Audio captured from selected device | [ ] |
| Check backend logs after device switch | Logs show correct device ID being used | [ ] |
| Switch device while monitoring is active | Both monitoring and recording use new device | [ ] |
| Verify device audio mode | Device not stuck in communication mode | [ ] |

## Integration Points

- Frontend device selection (React) → Tauri command → Rust backend
- Audio monitoring stream
- Audio recording stream
- cpal device enumeration and selection

## Integration Test

Manual verification:
1. Select non-default microphone
2. Start recording and verify audio is from correct device
3. Check logs confirm device ID matches selection
