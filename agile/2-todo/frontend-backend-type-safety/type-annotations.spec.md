---
status: pending
created: 2025-12-23
completed: null
dependencies: ["tauri-specta-setup"]
---

# Spec: Add serde + specta annotations to all types and commands

## Description

Add `#[serde(rename_all = "camelCase")]` to all structs missing it, `#[derive(specta::Type)]` to all types that cross the IPC boundary, and `#[specta::specta]` to all Tauri commands. This ensures consistent camelCase JSON serialization and enables TypeScript type generation.

## Acceptance Criteria

- [ ] All structs that serialize to JSON have `#[serde(rename_all = "camelCase")]`
- [ ] All types crossing IPC boundary have `#[derive(specta::Type)]`
- [ ] All `#[tauri::command]` handlers have `#[specta::specta]`
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes

## Test Cases

- [ ] Test that event payloads serialize with camelCase field names
- [ ] Test that command return types serialize with camelCase field names

## Dependencies

- tauri-specta-setup.spec.md (specta crate must be available)

## Preconditions

- specta crate is added to Cargo.toml

## Implementation Notes

### Structs Needing `#[serde(rename_all = "camelCase")]`

**src-tauri/src/events.rs:**
- `RecordingStartedPayload` (line 148)
- `RecordingStoppedPayload` (line 155)
- `RecordingErrorPayload` (line 162)
- `TranscriptionStartedPayload` (line 179)
- `TranscriptionCompletedPayload` (line 186)
- `TranscriptionErrorPayload` (line 195)
- `CommandMatchedPayload` (line 202)
- `CommandCandidate` (line 215)
- `CommandAmbiguousPayload` (line 226)
- `CommandExecutedPayload` (line 235)
- `CommandFailedPayload` (line 246)

**src-tauri/src/recording/state.rs:**
- `RecordingState` enum (line 7)
- `AudioData` (line 53)
- `RecordingMetadata` (line 64)

**src-tauri/src/audio/device.rs:**
- `AudioInputDevice` (line 9)

**src-tauri/src/audio/mod.rs:**
- `StopReason` enum

**src-tauri/src/commands/logic.rs:**
- `RecordingInfo`
- `RecordingStateInfo`

**src-tauri/src/voice_commands/mod.rs:**
- `CommandDto`
- `AddCommandInput`
- `UpdateCommandInput`

### Types Needing `#[derive(specta::Type)]`

All of the above, plus:
- `src-tauri/src/audio/error.rs` - `AudioDeviceError`
- `src-tauri/src/dictionary/store.rs` - `DictionaryEntry`
- `src-tauri/src/listening/manager.rs` - `ListeningStatus`
- `src-tauri/src/model/download.rs` - `ModelType`

### Commands Needing `#[specta::specta]`

Add `#[specta::specta]` below `#[tauri::command]` for all commands in:
- `src-tauri/src/commands/mod.rs` (~15 commands)
- `src-tauri/src/commands/dictionary.rs` (4 commands)
- `src-tauri/src/voice_commands/mod.rs` (4 commands)
- `src-tauri/src/voice_commands/executor.rs` (test_command)
- `src-tauri/src/model/mod.rs` (2 commands)

## Related Specs

- tauri-specta-setup.spec.md (dependency)
- bindings-generation.spec.md (depends on this spec)

## Integration Points

- Production call site: All command handlers and event emission points
- Connects to: Frontend via generated bindings

## Integration Test

- Test location: Existing test in events.rs:test_serde_camel_case_rename can be extended
- Verification: [ ] Integration test passes
