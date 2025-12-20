---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["query-infrastructure", "zustand-store"]
---

# Spec: Central Tauri event dispatcher

## Description

Create a central event bridge that subscribes to all Tauri backend events and routes them appropriately: server state events trigger Tanstack Query invalidation, UI state events update the Zustand store. This is the key integration point between backend-initiated events and frontend state.

## Acceptance Criteria

- [ ] `src/lib/eventBridge.ts` created
- [ ] `setupEventBridge(queryClient, store)` function exported
- [ ] Function returns cleanup function for unsubscribing all listeners
- [ ] Server state events trigger query invalidation:
  - `recording_started` → invalidate `['tauri', 'get_recording_state']`
  - `recording_stopped` → invalidate `['tauri', 'get_recording_state']`
  - `recording_error` → invalidate `['tauri', 'get_recording_state']`
  - `transcription_completed` → invalidate `['tauri', 'list_recordings']`
  - `listening_started` → invalidate `['tauri', 'get_listening_status']`
  - `listening_stopped` → invalidate `['tauri', 'get_listening_status']`
  - `model_download_completed` → invalidate `['tauri', 'check_parakeet_model_status']`
- [ ] UI state events update Zustand:
  - `overlay-mode` → `store.setOverlayMode(payload)`
- [ ] All `listen()` calls use proper Tauri v2 API
- [ ] Event payloads are typed (no `any`)
- [ ] No duplicate listeners (single subscription per event type)

## Test Cases

- [ ] `setupEventBridge()` returns a cleanup function
- [ ] Cleanup function unsubscribes all listeners when called
- [ ] Recording events trigger correct query invalidation
- [ ] Listening events trigger correct query invalidation
- [ ] UI events update Zustand store

## Dependencies

- `query-infrastructure` - provides QueryClient reference
- `zustand-store` - provides store reference for UI state updates

## Preconditions

- QueryClient created and accessible
- Zustand store created and accessible
- Tauri app context available (for `listen()` API)

## Implementation Notes

```typescript
// src/lib/eventBridge.ts
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { QueryClient } from '@tanstack/react-query';
import { queryKeys } from './queryKeys';
import { useAppStore } from '../stores/appStore';

type AppStore = ReturnType<typeof useAppStore.getState>;

export async function setupEventBridge(
  queryClient: QueryClient,
  store: AppStore
): Promise<() => void> {
  const unlistenFns: UnlistenFn[] = [];

  // Server state events → Query invalidation
  unlistenFns.push(await listen('recording_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  }));

  unlistenFns.push(await listen('recording_stopped', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  }));

  unlistenFns.push(await listen('transcription_completed', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listRecordings });
  }));

  unlistenFns.push(await listen('listening_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getListeningStatus });
  }));

  unlistenFns.push(await listen('listening_stopped', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getListeningStatus });
  }));

  // UI state events → Zustand updates
  unlistenFns.push(await listen<string>('overlay-mode', (event) => {
    store.setOverlayMode(event.payload);
  }));

  // Return cleanup function
  return () => {
    unlistenFns.forEach(unlisten => unlisten());
  };
}
```

## Related Specs

- `query-infrastructure` - provides QueryClient and queryKeys
- `zustand-store` - provides store for UI updates
- `app-providers-wiring` - calls setupEventBridge on mount
- All `*-query-hooks` - benefit from automatic invalidation

## Integration Points

- Production call site: `src/App.tsx` (called in useEffect on mount)
- Connects to: queryClient (invalidation), appStore (UI updates), Tauri backend (events)

## Integration Test

- Test location: `src/lib/__tests__/eventBridge.test.ts`
- Verification: [ ] Mock event emission triggers correct invalidation/store update
