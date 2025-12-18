---
status: in-progress
created: 2025-12-18
completed: null
dependencies: []
---

# Spec: Extract TranscriptionService from HotkeyIntegration

## Description

Extract the transcription flow logic from `HotkeyIntegration` into a standalone `TranscriptionService` that can be called from any recording trigger (hotkey, UI button, wake word).

Currently, `HotkeyIntegration::spawn_transcription()` (integration.rs:494-789) contains ~300 lines of transcription logic that is tightly coupled to the hotkey module. This causes:
- Button-initiated recordings don't get transcribed (they call `stop_recording` command which lacks transcription)
- Wake word flow has to call `HotkeyIntegration` directly for transcription
- Testing transcription requires mocking the entire hotkey infrastructure

## Acceptance Criteria

- [ ] New `TranscriptionService` struct in `src-tauri/src/transcription/` module
- [ ] Service handles: WAV transcription → command matching → clipboard fallback
- [ ] Service is managed as Tauri state (accessible from commands)
- [ ] `stop_recording` command calls `TranscriptionService` after successful stop
- [ ] `HotkeyIntegration` delegates to `TranscriptionService` (no duplicate logic)
- [ ] Wake word flow uses `TranscriptionService`
- [ ] Button-initiated recordings now trigger transcription

## Test Cases

- [ ] Button-initiated recording produces transcription in log
- [ ] Hotkey-initiated recording still produces transcription (no regression)
- [ ] Transcription triggers command matching when commands are configured
- [ ] Transcription falls back to clipboard when no command matches

## Dependencies

None

## Preconditions

- Existing transcription logic works correctly via hotkey

## Implementation Notes

**Current flow (hotkey only):**
```
Hotkey → HotkeyIntegration::handle_toggle()
  → stop_recording_impl()
  → HotkeyIntegration::spawn_transcription(file_path)  ← TRANSCRIPTION HERE
    → TranscriptionManager::transcribe()
    → CommandMatcher::find_matches()
    → Clipboard + auto-paste
```

**Target flow (both triggers):**
```
[Hotkey OR Button] → stop_recording
  → TranscriptionService::process_recording(file_path)  ← NEW
    → TranscriptionManager::transcribe()
    → CommandMatcher::find_matches()
    → Clipboard + auto-paste
```

**Key files:**
- `src-tauri/src/hotkey/integration.rs` - Has spawn_transcription (lines 494-789)
- `src-tauri/src/commands/mod.rs` - stop_recording command (lines 228-268)
- New: `src-tauri/src/transcription/service.rs`

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs:stop_recording`
- Connects to: TranscriptionManager, CommandMatcher, clipboard module

## Integration Test

- Test location: Manual E2E - click Start Recording, speak, click Stop Recording
- Verification: [ ] Transcription appears in log and clipboard
