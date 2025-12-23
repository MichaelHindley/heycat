---
status: pending
created: 2025-12-23
completed: null
dependencies: ["bindings-generation"]
---

# Spec: Update frontend to use generated types

## Description

Migrate the frontend to use the generated TypeScript bindings from `src/lib/bindings.ts`. This includes updating imports in hooks, updating eventBridge.ts to use camelCase field names, and deleting manual type definitions that are now generated.

## Acceptance Criteria

- [ ] All hooks import types from `./bindings` instead of manual definitions
- [ ] `eventBridge.ts` updated to use camelCase (e.g., `durationMs` not `duration_ms`)
- [ ] Manual type files deleted (`src/types/audio.ts`, `src/types/dictionary.ts`)
- [ ] TypeScript compilation succeeds with no type errors
- [ ] App runs correctly with generated types
- [ ] E2E tests pass

## Test Cases

- [ ] `bun run typecheck` passes
- [ ] `bun run build` succeeds
- [ ] Recording flow works end-to-end
- [ ] Transcription payload received correctly with camelCase fields
- [ ] Event listeners receive correct payload shapes

## Dependencies

- bindings-generation.spec.md (bindings.ts must exist)

## Preconditions

- `src/lib/bindings.ts` has been generated
- All Rust types use camelCase serialization

## Implementation Notes

### Update `src/lib/eventBridge.ts`

Before:
```typescript
export interface TranscriptionCompletedPayload {
  text: string;
  duration_ms: number;  // snake_case
}

store.transcriptionCompleted(event.payload.text, event.payload.duration_ms);
```

After:
```typescript
import { TranscriptionCompletedPayload } from "./bindings";

store.transcriptionCompleted(event.payload.text, event.payload.durationMs);  // camelCase
```

### Update hooks to import from bindings

**src/hooks/useRecording.ts:**
```typescript
import { RecordingMetadata, RecordingStateInfo } from "../lib/bindings";
```

**src/hooks/useListening.ts:**
```typescript
import { ListeningStatus } from "../lib/bindings";
```

**src/hooks/useDisambiguation.ts:**
```typescript
import { CommandMatchedPayload, CommandAmbiguousPayload } from "../lib/bindings";
```

### Delete manual type files

- Delete `src/types/audio.ts` (AudioInputDevice now from bindings)
- Delete `src/types/dictionary.ts` (DictionaryEntry now from bindings)
- Update any imports from these deleted files

### Update component imports

Search for all files importing from `../types/audio` or `../types/dictionary` and update to import from `../lib/bindings`.

## Related Specs

- bindings-generation.spec.md (dependency)

## Integration Points

- Production call site: All hooks and components that use Tauri types
- Connects to: `src/lib/bindings.ts`

## Integration Test

- Test location: E2E test (recording + transcription flow)
- Verification: [ ] Integration test passes
