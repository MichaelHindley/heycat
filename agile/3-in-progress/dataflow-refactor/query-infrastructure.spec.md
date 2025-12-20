---
status: in-progress
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Set up Tanstack Query foundation

## Description

Install Tanstack Query and create the foundational infrastructure for wrapping Tauri commands in queries. This establishes the QueryClient configuration and the command-based query key convention that all other specs will use.

## Acceptance Criteria

- [ ] `@tanstack/react-query` package installed and in package.json
- [ ] `@tanstack/react-query-devtools` installed for development debugging
- [ ] `src/lib/queryClient.ts` created and exports configured QueryClient
- [ ] QueryClient configured with sensible defaults:
  - `staleTime`: 60 seconds (data considered fresh)
  - `gcTime`: 5 minutes (cache garbage collection)
  - `retry`: 3 attempts with exponential backoff
  - `refetchOnWindowFocus`: false (desktop app, not browser)
- [ ] `src/lib/queryKeys.ts` created with typed query key factory
- [ ] Query keys follow command-based pattern: `['tauri', 'command_name']`
- [ ] Query key factory exports constants for all known Tauri commands:
  - `listRecordings`, `getRecordingState`, `listAudioDevices`
  - `getListeningStatus`, `checkModelStatus(type)`
- [ ] TypeScript types are strict (no `any`, proper inference)
- [ ] Exports are tree-shakeable (named exports, not default)

## Test Cases

- [ ] QueryClient can be instantiated without errors
- [ ] Query keys are correctly typed and produce expected arrays
- [ ] `queryKeys.tauri.listRecordings` equals `['tauri', 'list_recordings']`
- [ ] `queryKeys.tauri.checkModelStatus('tdt')` equals `['tauri', 'check_parakeet_model_status', 'tdt']`

## Dependencies

None - this is a foundational spec.

## Preconditions

- Node.js and bun available
- Existing React + TypeScript project structure

## Implementation Notes

```typescript
// src/lib/queryClient.ts
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 3,
      refetchOnWindowFocus: false,
    },
  },
});

// src/lib/queryKeys.ts
export const queryKeys = {
  tauri: {
    listRecordings: ['tauri', 'list_recordings'] as const,
    getRecordingState: ['tauri', 'get_recording_state'] as const,
    listAudioDevices: ['tauri', 'list_audio_devices'] as const,
    getListeningStatus: ['tauri', 'get_listening_status'] as const,
    checkModelStatus: (type: string) => ['tauri', 'check_parakeet_model_status', type] as const,
  },
} as const;
```

## Related Specs

- `event-bridge` - Uses queryClient for invalidation
- `app-providers-wiring` - Wraps app with QueryClientProvider
- All `*-query-hooks` specs - Use queryKeys for cache management

## Integration Points

- Production call site: `src/App.tsx` (QueryClientProvider)
- Connects to: event-bridge (queryClient reference), all query hooks (queryKeys)

## Integration Test

- Test location: `src/lib/__tests__/queryClient.test.ts`
- Verification: [ ] Unit tests pass for query key generation
