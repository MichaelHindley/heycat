---
status: in-review
severity: major
origin: manual
owner: Michael
created: 2025-12-17
completed: null
parent_feature: "audio-device-selection"
parent_spec: null
review_round: 1
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

---

## Review

**Review Date:** 2025-12-17
**Review Round:** 1
**Reviewer:** Independent subagent

### Pre-Review Gates (Automated)

#### 1. Build Warning Check
```bash
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
```
**Result:** FAIL - Unused import warning detected:
```
warning: unused imports: `VAD_CHUNK_SIZE_16KHZ` and `VAD_CHUNK_SIZE_8KHZ`
```
While this warning is unrelated to the bug fix, new code should not introduce warnings.

#### 2. Command Registration Check
**Result:** PASS - All commands are properly registered.

#### 3. Event Subscription Check
**Result:** PASS - No new events introduced.

---

### Manual Review

#### 1. Is the code wired up end-to-end?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `UseRecordingOptions.deviceName` | interface | useRecording.ts:28-31 | YES |
| `UseListeningOptions.deviceName` | interface | useListening.ts:28-31 | YES |
| `useRecording({ deviceName })` | hook param | useRecording.ts:48-51 | **NO - NOT WIRED** |
| `useListening({ deviceName })` | hook param | useListening.ts:50-53 | **NO - NOT WIRED** |
| `enable_listening_impl(..., device_name)` | fn param | logic.rs:485-492 | YES |
| `start_recording_impl(..., device_name)` | fn param | logic.rs:53-58 | YES |

**FAIL:** The hooks `useRecording` and `useListening` accept `deviceName` in options but:
- `App.tsx:18` calls `useRecording()` with no device
- `ListeningSettings.tsx:18` calls `useListening()` with no device
- `useAutoStartListening.ts:28` calls `invoke("enable_listening")` with no device
- `useCatOverlay.ts:63` calls `useRecording()` with no device

The settings store has `settings.audio.selectedDevice` but **none of the hook consumers pass it**.

#### 2. What would break if this code was deleted?

The device parameter plumbing exists in the backend (commands/logic.rs, pipeline.rs, cpal_backend.rs) and would function correctly **if** called with the device parameter. However, the frontend UI components do not pass the selected device from settings, so deleting the device_name parameter would not change current behavior - it's dead code at the UI layer.

#### 3. Where does the data flow?

**Expected flow (BROKEN):**
```
[Settings UI] AudioDeviceSelector updates settings.audio.selectedDevice
     |
     v
[Settings Store] persists device name
     |
     v
[Hook Consumer] App.tsx / ListeningSettings.tsx reads settings
     |
     X <--- BROKEN LINK - device not passed to hooks
     v
[Hook] useRecording({ deviceName }) / useListening({ deviceName })
     | invoke("start_recording", { deviceName })
     v
[Command] mod.rs -> logic.rs -> audio thread -> cpal_backend
```

**Actual flow:**
- AudioDeviceSelector correctly stores device to settings
- Settings are correctly read by consumers
- **But consumers don't pass deviceName to hooks**
- Hooks invoke commands with `undefined` deviceName
- Backend falls back to system default

#### 4. Are there any deferrals?

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| "Communication mode issue: Needs separate investigation" | device-not-switching.bug.md:59 | **MISSING** |

The "communication mode" issue is deferred but has no tracking spec.

#### 5. Automated check results
```
Build warnings: 1 (unused imports, unrelated to bug fix)
Command registration: PASS
Event subscription: PASS
```

---

### Verdict: NEEDS_WORK

**What failed:**
1. Question 1: Frontend code not wired up - `deviceName` is not passed from settings to hooks
2. Question 4: Deferred "communication mode" issue has no tracking spec

**Why it failed:**
- `App.tsx`, `ListeningSettings.tsx`, `useCatOverlay.ts`, and `useAutoStartListening.ts` all call recording/listening hooks without passing the selected device from `settings.audio.selectedDevice`
- The UI allows selecting a device but that selection is only used for the audio level monitor preview - the actual recording/listening paths ignore it

**How to fix:**
1. In `ListeningSettings.tsx:18`, change:
   ```typescript
   const { isListening, enableListening, disableListening } = useListening();
   ```
   to:
   ```typescript
   const { isListening, enableListening, disableListening } = useListening({
     deviceName: settings.audio.selectedDevice
   });
   ```

2. In `App.tsx:18`, pass device from settings (requires adding useSettings import):
   ```typescript
   const { settings } = useSettings();
   const { startRecording } = useRecording({ deviceName: settings.audio.selectedDevice });
   ```

3. In `useAutoStartListening.ts:28`, read device from store and pass it:
   ```typescript
   const selectedDevice = await store.get<string | null>("audio.selectedDevice");
   await invoke("enable_listening", { deviceName: selectedDevice ?? undefined });
   ```

4. In `useCatOverlay.ts:63`, pass device (requires context or prop drilling)

5. Create tracking spec for "communication mode" deferral
