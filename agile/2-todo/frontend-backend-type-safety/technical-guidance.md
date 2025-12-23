---
last-updated: 2025-12-23
status: approved
---

# Technical Guidance: Full Type Safety Between Frontend and Backend

## Architecture Overview

Use **tauri-specta v2** for compile-time type safety between Rust and TypeScript. This generates TypeScript types from Rust structs, ensuring the frontend and backend stay in sync.

### Data Flow Architecture (where types cross IPC boundary)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FRONTEND (React)                               │
│                                                                          │
│   ┌──────────────┐     ┌─────────────────┐     ┌──────────────────┐     │
│   │ React Hooks  │────▶│  TanStack Query │────▶│  invoke(cmd)     │─────┼──▶ COMMANDS
│   │              │     │  queryFn        │     │                  │     │   (need types)
│   └──────────────┘     └─────────────────┘     └──────────────────┘     │
│         ▲                                                                │
│         │              ┌─────────────────┐     ┌──────────────────┐     │
│         └──────────────│  Event Bridge   │◀────│  listen(event)   │◀────┼── EVENTS
│                        │  (routes to:)   │     │                  │     │   (need types)
│                        │  - Query inval. │     └──────────────────┘     │
│                        │  - Zustand      │                               │
│                        └─────────────────┘                               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ════════════════╪═══════════════  Tauri IPC Boundary
                                    │
┌───────────────────────────────────┼─────────────────────────────────────┐
│                           BACKEND (Rust)                                 │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐  │
│   │ #[tauri::command] handlers                                        │  │
│   │                                                                   │  │
│   │  - Return types: Result<T, String>  ← T needs #[derive(Type)]    │  │
│   │  - Parameters: snake_case (Tauri auto-converts from camelCase)   │  │
│   └──────────────────────────────────────────────────────────────────┘  │
│                         │                                                │
│                         ▼                                                │
│   ┌──────────────────────────────────────────────────────────────────┐  │
│   │ app_handle.emit(event, payload)                                   │  │
│   │                                                                   │  │
│   │  - Payload types need #[serde(rename_all = "camelCase")]         │  │
│   │  - Payload types need #[derive(Type)] for TS generation          │  │
│   └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Type Boundaries Requiring Coverage

1. **Command return types** - `Result<RecordingMetadata, String>`, `Result<Vec<DictionaryEntry>, String>`, etc.
2. **Command input types** - `AddCommandInput`, `UpdateCommandInput`, etc.
3. **Event payloads** - `TranscriptionCompletedPayload`, `CommandMatchedPayload`, etc.

### TanStack Query Integration Points

- `src/hooks/useRecording.ts` - `getRecordingState`, `listRecordings`
- `src/hooks/useListening.ts` - `getListeningStatus`
- `src/hooks/useMultiModelStatus.ts` - `checkParakeetModelStatus`
- `src/lib/queryKeys.ts` - Query key definitions

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| tauri-specta over TauRPC | Minimal code changes, just add attributes. TauRPC requires trait-based API restructuring. | 2025-12-23 |
| Fix serde first | Ensures consistent camelCase before generating types. Breaking change bundled with type safety. | 2025-12-23 |
| Generate to src/lib/bindings.ts | Central location, easy to import across all hooks. | 2025-12-23 |
| Commit generated bindings | CI doesn't need Rust toolchain, reviewable diffs show API changes. | 2025-12-23 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | 16+ structs missing serde rename | Frontend receives snake_case (e.g., `duration_ms` instead of `durationMs`) |
| 2025-12-23 | eventBridge.ts adapted to snake_case | Must update when fixing serde consistency |
| 2025-12-23 | No type generation currently | Types manually duplicated, prone to drift |
| 2025-12-23 | Structs WITH camelCase: WakeWordDetectedPayload, ListeningStartedPayload, ListeningStoppedPayload, ListeningUnavailablePayload, DictionaryUpdatedPayload, ModelDownloadCompletedPayload, ModelFileDownloadProgressPayload, DictionaryEntry, ListeningStatus, AudioDeviceError, RecordingCancelledPayload | These are already correct |
| 2025-12-23 | Structs WITHOUT camelCase: RecordingStartedPayload, RecordingStoppedPayload, RecordingErrorPayload, TranscriptionStartedPayload, TranscriptionCompletedPayload, TranscriptionErrorPayload, CommandMatchedPayload, CommandCandidate, CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload, RecordingMetadata, AudioData, AudioInputDevice, RecordingStateInfo, RecordingInfo, RecordingState, StopReason, CommandDto | These need serde rename added |

## Open Questions

- [x] Best practice for Tauri type safety? → tauri-specta v2
- [x] Commit generated bindings or regenerate on build? → Commit them (ensures CI doesn't need Rust toolchain, reviewable diffs)

## Files to Modify

### Rust (Backend)
- `src-tauri/Cargo.toml` - Add specta dependencies
- `src-tauri/src/lib.rs` - Add specta module
- `src-tauri/src/events.rs` - Add serde + Type derives to 11 structs
- `src-tauri/src/recording/state.rs` - Add serde + Type to 3 types
- `src-tauri/src/audio/device.rs` - Add Type
- `src-tauri/src/audio/mod.rs` - Add serde + Type to StopReason
- `src-tauri/src/audio/error.rs` - Add Type
- `src-tauri/src/commands/mod.rs` - Add specta to ~15 commands
- `src-tauri/src/commands/logic.rs` - Add serde + Type to 2 structs
- `src-tauri/src/commands/dictionary.rs` - Add specta to 4 commands
- `src-tauri/src/dictionary/store.rs` - Add Type
- `src-tauri/src/listening/manager.rs` - Add Type
- `src-tauri/src/voice_commands/mod.rs` - Add serde + Type + specta
- `src-tauri/src/voice_commands/executor.rs` - Add specta to test_command
- `src-tauri/src/model/mod.rs` - Add specta to 2 commands
- `src-tauri/src/model/download.rs` - Add Type

### New Files
- `src-tauri/src/specta.rs` - Builder and export function
- `src/lib/bindings.ts` - Generated TypeScript (auto)

### TypeScript (Frontend)
- `src/lib/eventBridge.ts` - Update to camelCase, import from bindings
- `src/hooks/useRecording.ts` - Import from bindings
- `src/hooks/useListening.ts` - Import from bindings
- `src/hooks/useDisambiguation.ts` - Import from bindings
- `src/types/audio.ts` - Delete (use bindings)
- `src/types/dictionary.ts` - Delete (use bindings)

## References

- [tauri-specta GitHub](https://github.com/specta-rs/tauri-specta)
- [TauRPC](https://github.com/MatsDK/TauRPC)
- [Specta Docs](https://specta.dev/docs/tauri-specta/v2)
