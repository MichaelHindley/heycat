---
status: pending
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Create Zustand store for client state

## Description

Install Zustand and create the app store for managing global client-side state. This store holds UI state (overlay visibility, app status) and cached settings. Following the separation of concerns principle: NO server state in this store - that belongs in Tanstack Query.

## Acceptance Criteria

- [ ] `zustand` package installed and in package.json
- [ ] `src/stores/appStore.ts` created with typed store
- [ ] Store contains client state slices:
  - `overlayMode`: string | null (current overlay state)
  - `settingsCache`: Settings object (cached from Tauri Store)
  - `isSettingsLoaded`: boolean (hydration flag)
- [ ] Store actions defined:
  - `setOverlayMode(mode: string | null)`
  - `setSettings(settings: Settings)`
  - `updateSetting(key: string, value: any)`
- [ ] Selectors exported for optimized re-renders:
  - `useOverlayMode()` - returns only overlayMode
  - `useSettingsCache()` - returns only settings
- [ ] TypeScript types are strict with proper inference
- [ ] Store does NOT contain server state (recordings, recording state, etc.)
- [ ] No persist middleware (settings persist via Tauri Store, not localStorage)

## Test Cases

- [ ] Store initializes with default values
- [ ] `setOverlayMode('recording')` updates state correctly
- [ ] `setSettings(mockSettings)` updates settingsCache
- [ ] Selectors return only their slice (not full store)
- [ ] Multiple components using same selector share state

## Dependencies

None - this is a foundational spec.

## Preconditions

- Existing React + TypeScript project structure

## Implementation Notes

```typescript
// src/stores/appStore.ts
import { create } from 'zustand';

interface Settings {
  listening: { enabled: boolean; autoStartOnLaunch: boolean };
  audio: { selectedDevice: string | null };
  shortcuts: { distinguishLeftRight: boolean };
}

interface AppState {
  // Client state only - NO server state here
  overlayMode: string | null;
  settingsCache: Settings | null;
  isSettingsLoaded: boolean;

  // Actions
  setOverlayMode: (mode: string | null) => void;
  setSettings: (settings: Settings) => void;
  updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

export const useAppStore = create<AppState>((set) => ({
  overlayMode: null,
  settingsCache: null,
  isSettingsLoaded: false,

  setOverlayMode: (mode) => set({ overlayMode: mode }),
  setSettings: (settings) => set({ settingsCache: settings, isSettingsLoaded: true }),
  updateSetting: (key, value) => set((state) => ({
    settingsCache: state.settingsCache
      ? { ...state.settingsCache, [key]: value }
      : null
  })),
}));

// Optimized selectors
export const useOverlayMode = () => useAppStore((s) => s.overlayMode);
export const useSettingsCache = () => useAppStore((s) => s.settingsCache);
export const useIsSettingsLoaded = () => useAppStore((s) => s.isSettingsLoaded);
```

## Related Specs

- `event-bridge` - Updates store on UI state events
- `settings-zustand-hooks` - Uses store for settings access
- `app-providers-wiring` - Initializes settings into store

## Integration Points

- Production call site: Components via `useAppStore()` hook
- Connects to: event-bridge (receives updates), settings-zustand-hooks (primary consumer)

## Integration Test

- Test location: `src/stores/__tests__/appStore.test.ts`
- Verification: [ ] Unit tests pass for store actions and selectors
